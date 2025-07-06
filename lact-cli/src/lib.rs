use anyhow::Result;
use lact_client::DaemonClient;
use lact_schema::args::{CliArgs, CliCommand};

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
