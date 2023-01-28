mod args;

use anyhow::{Context, Result};
use args::{Args, Command};
use clap::Parser;
use lact_client::DaemonClient;

fn main() -> Result<()> {
    let args = Args::parse();
    let client = DaemonClient::connect()?;

    let f = match args.subcommand {
        Command::ListGpus => list_gpus,
        Command::Info => info,
    };
    f(&args, &client)
}

fn list_gpus(_: &Args, client: &DaemonClient) -> Result<()> {
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

fn info(args: &Args, client: &DaemonClient) -> Result<()> {
    for id in args.gpu_ids(client) {
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
