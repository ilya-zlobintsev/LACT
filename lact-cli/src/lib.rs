mod subcommands;

use crate::subcommands::{
    current_auto_switch, current_profile, info, list_gpus, list_profiles, power_limit,
    set_auto_switch, set_profile, snapshot, stats,
};
use anyhow::{bail, Context, Result};
use lact_client::DaemonClient;
use lact_schema::{
    args::cli::{CliArgs, CliCommand, ProfileAutoSwitchCommand, ProfileCommand},
    config::GpuConfig,
    request::ConfirmCommand,
};

pub fn run(args: CliArgs) -> Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async move {
        let client = DaemonClient::connect().await?;

        let ctx = CliContext {
            client,
            args: &args,
        };

        match &args.subcommand {
            CliCommand::List => list_gpus(ctx).await,
            CliCommand::Info => info(ctx).await,
            CliCommand::Stats => stats(ctx).await,
            CliCommand::Snapshot => snapshot(ctx).await,
            CliCommand::PowerLimit { cmd } => power_limit(ctx, cmd.as_ref()).await,
            CliCommand::Profile(profile_args) => match &profile_args.subcommand {
                None => current_profile(profile_args, ctx).await,
                Some(profile_subcommand) => match profile_subcommand {
                    ProfileCommand::List => list_profiles(profile_args, ctx).await,
                    ProfileCommand::Get => current_profile(profile_args, ctx).await,
                    ProfileCommand::Set(set_profile_args) => {
                        set_profile(set_profile_args, ctx).await
                    }
                    ProfileCommand::AutoSwitch(auto_switch_args) => {
                        match &auto_switch_args.subcommand {
                            None => current_auto_switch(auto_switch_args, ctx).await,
                            Some(auto_switch_command) => match auto_switch_command {
                                ProfileAutoSwitchCommand::Get => {
                                    current_auto_switch(auto_switch_args, ctx).await
                                }
                                ProfileAutoSwitchCommand::Enable => {
                                    set_auto_switch(auto_switch_args, ctx, true).await
                                }
                                ProfileAutoSwitchCommand::Disable => {
                                    set_auto_switch(auto_switch_args, ctx, false).await
                                }
                            },
                        }
                    }
                },
            },
        }
    })
}

struct CliContext<'a> {
    args: &'a CliArgs,
    client: DaemonClient,
}

impl CliContext<'_> {
    async fn current_gpu_id(&self) -> anyhow::Result<String> {
        let entries = self
            .client
            .list_devices()
            .await
            .context("Could not list GPUs")?;
        if entries.is_empty() {
            bail!("No GPUs detected");
        }

        match self.args.gpu_id {
            Some(ref id) => {
                let id = if let Ok(index) = id.parse::<usize>() {
                    entries
                        .get(index)
                        .with_context(|| format!("Could not get GPU {id}"))?
                        .id
                        .clone()
                } else if entries.iter().any(|entry| entry.id == *id) {
                    id.clone()
                } else {
                    bail!("GPU with id {id} not found")
                };

                Ok(id)
            }
            None => {
                let first_entry = entries.first().unwrap();
                if entries.len() > 1 {
                    eprintln!(
                        "GPU id not specified, selecting {}",
                        first_entry.name.as_deref().unwrap_or("<Unknown>")
                    );
                }
                Ok(first_entry.id.clone())
            }
        }
    }

    async fn edit_gpu_config(
        &self,
        gpu_id: &str,
        f: impl FnOnce(&mut GpuConfig),
    ) -> anyhow::Result<()> {
        let mut config = self
            .client
            .get_gpu_config(gpu_id)
            .await?
            .unwrap_or_default();

        f(&mut config);

        self.client
            .set_gpu_config(gpu_id, config)
            .await
            .context("Failed to apply config")?;

        self.client
            .confirm_pending_config(ConfirmCommand::Confirm)
            .await
            .context("Failed to confirm config")?;

        Ok(())
    }
}
