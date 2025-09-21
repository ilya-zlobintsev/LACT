mod subcommands;

use crate::subcommands::{info, list_gpus, power_limit, snapshot};
use anyhow::{bail, Context, Result};
use lact_client::DaemonClient;
use lact_schema::{
    args::cli::{CliArgs, CliCommand},
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

        match args.subcommand {
            CliCommand::List => list_gpus(ctx).await,
            CliCommand::Info => info(ctx).await,
            CliCommand::Snapshot => snapshot(ctx).await,
            CliCommand::PowerLimit { cmd } => power_limit(ctx, cmd).await,
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
