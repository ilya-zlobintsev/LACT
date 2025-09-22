use anyhow::Result;
use lact_client::DaemonClient;
use lact_schema::args::{
    CliArgs, CliCommand, ProfileArgs, ProfileAutoSwitchArgs, ProfileAutoSwitchCommand,
    ProfileCommand, SetProfileArgs,
};

const PROFILE_DEFAULT: &str = "Default";

pub fn run(args: CliArgs) -> Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async move {
        let client = DaemonClient::connect().await?;

        match args.subcommand {
            CliCommand::ListGpus => list_gpus(&args, &client).await,
            CliCommand::Info => info(&args, &client).await,
            CliCommand::Snapshot => snapshot(&client).await,
            CliCommand::Profile(profile_args) => match &profile_args.subcommand {
                None => current_profile(&profile_args, &client).await,
                Some(profile_subcommand) => match profile_subcommand {
                    ProfileCommand::List => list_profiles(&profile_args, &client).await,
                    ProfileCommand::Get => current_profile(&profile_args, &client).await,
                    ProfileCommand::Set(set_profile_args) => {
                        set_profile(&set_profile_args, &client).await
                    }
                    ProfileCommand::AutoSwitch(auto_switch_args) => {
                        match &auto_switch_args.subcommand {
                            None => current_auto_switch(auto_switch_args, &client).await,
                            Some(auto_switch_command) => match auto_switch_command {
                                ProfileAutoSwitchCommand::Get => {
                                    current_auto_switch(auto_switch_args, &client).await
                                }
                                ProfileAutoSwitchCommand::Enable => {
                                    set_auto_switch(auto_switch_args, &client, true).await
                                }
                                ProfileAutoSwitchCommand::Disable => {
                                    set_auto_switch(auto_switch_args, &client, false).await
                                }
                            },
                        }
                    }
                },
            },
        }
    })
}

async fn list_gpus(_: &CliArgs, client: &DaemonClient) -> Result<()> {
    let entries = client.list_devices().await?;
    for entry in entries {
        let id = entry.id;
        let device_type = entry.device_type;

        if let Some(name) = entry.name {
            println!("{id} ({name}) [{device_type}]");
        } else {
            println!("{id} [{device_type}]");
        }
    }
    Ok(())
}

async fn info(args: &CliArgs, client: &DaemonClient) -> Result<()> {
    for id in extract_gpu_ids(args, client).await {
        let gpu_line = format!("GPU {id}:");
        println!("{gpu_line}");
        println!("{}", "=".repeat(gpu_line.len()));

        let info = client.get_device_info(&id).await?;
        let stats = client.get_device_stats(&id).await?;

        let elements = info.info_elements(Some(&stats));
        for (name, value) in elements {
            if let Some(value) = value {
                println!("{name}: {value}");
            }
        }
    }
    Ok(())
}

async fn extract_gpu_ids(args: &CliArgs, client: &DaemonClient) -> Vec<String> {
    match args.gpu_id {
        Some(ref id) => vec![id.clone()],
        None => {
            let entries = client.list_devices().await.expect("Could not list GPUs");
            entries
                .into_iter()
                .map(|entry| entry.id.to_owned())
                .collect()
        }
    }
}

async fn snapshot(client: &DaemonClient) -> Result<()> {
    let path = client.generate_debug_snapshot().await?;
    println!("Generated debug snapshot in {path}");
    Ok(())
}

async fn list_profiles(_: &ProfileArgs, client: &DaemonClient) -> Result<()> {
    let profiles_info = client.list_profiles(false).await?;
    println!("{}", PROFILE_DEFAULT);
    for (name, _rule) in profiles_info.profiles {
        println!("{}", name);
    }
    Ok(())
}

async fn current_profile(_: &ProfileArgs, client: &DaemonClient) -> Result<()> {
    let profiles_info = client.list_profiles(false).await?;
    if let Some(current_profile) = profiles_info.current_profile {
        println!("{}", current_profile);
    } else {
        println!("{}", PROFILE_DEFAULT);
    }
    Ok(())
}

async fn set_profile(args: &SetProfileArgs, client: &DaemonClient) -> Result<()> {
    let new_profile = args.name.trim();

    if new_profile.to_lowercase() == PROFILE_DEFAULT.to_lowercase() {
        client.set_profile(None, false).await?;
        println!("{}", PROFILE_DEFAULT);
    } else {
        // Ugly hack to workaround a bug:
        // When setting a profile while auto-switch is enabled, the new profile will not
        // be set. Adding a little delay to allow the auto-switcher to shutdown fixes the issue.
        client.set_profile(None, false).await?;
        loop {
            if let None = client.list_profiles(true).await?.watcher_state {
                break;
            } else {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        }
        // Remove above when fixed.

        client
            .set_profile(Some(new_profile.to_string()), false)
            .await?;
        println!("{}", new_profile);
    }
    Ok(())
}

async fn current_auto_switch(_: &ProfileAutoSwitchArgs, client: &DaemonClient) -> Result<()> {
    let auto_switch = client.list_profiles(false).await?.auto_switch;
    println!("{}", auto_switch);
    Ok(())
}

async fn set_auto_switch(
    _: &ProfileAutoSwitchArgs,
    client: &DaemonClient,
    enable: bool,
) -> Result<()> {
    client.set_profile(None, enable).await?;
    println!("{}", enable);
    Ok(())
}
