use anyhow::{Context, Result};
use lact_client::DaemonClient;
use lact_schema::args::{CliArgs, CliCommand};

pub fn run(args: CliArgs) -> Result<()> {
    let client = DaemonClient::connect()?;

    let f = match args.subcommand {
        CliCommand::ListGpus => list_gpus,
        CliCommand::Info => info,
    };
    f(&args, &client)
}

fn list_gpus(_: &CliArgs, client: &DaemonClient) -> Result<()> {
    let buffer = client.list_devices()?;
    for entry in buffer.inner()? {
        let id = entry.id;
        if let Some(name) = entry.name {
            println!("{id} ({name})");
        } else {
            println!("{id}");
        }
    }
    Ok(())
}

fn info(args: &CliArgs, client: &DaemonClient) -> Result<()> {
    for id in extract_gpu_ids(args, client) {
        let info_buffer = client.get_device_info(&id)?;
        let info = info_buffer.inner()?;
        let pci_info = info.pci_info.context("GPU reports no pci info")?;

        if let Some(ref vendor) = pci_info.device_pci_info.vendor {
            println!("GPU Vendor: {vendor}");
        }
        if let Some(ref model) = pci_info.device_pci_info.model {
            println!("GPU Model: {model}");
        }
        println!("Driver in use: {}", info.driver);
        if let Some(ref vbios_version) = info.vbios_version {
            println!("VBIOS version: {vbios_version}");
        }
        println!("Link: {:?}", info.link_info);
    }
    Ok(())
}

fn extract_gpu_ids(args: &CliArgs, client: &DaemonClient) -> Vec<String> {
    match args.gpu_id {
        Some(ref id) => vec![id.clone()],
        None => {
            let buffer = client.list_devices().expect("Could not list GPUs");
            buffer
                .inner()
                .expect("Could not deserialize GPUs response")
                .into_iter()
                .map(|entry| entry.id.to_owned())
                .collect()
        }
    }
}
