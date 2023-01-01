use clap::{Parser, Subcommand};
use lact_client::DaemonClient;

#[derive(Parser)]
#[command(author, version, about)]
pub struct Args {
    pub gpu_id: Option<String>,
    #[command(subcommand)]
    pub subcommand: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// List GPUs
    ListGpus,
    /// Show GPU info
    Info,
}

impl Args {
    pub fn gpu_ids(&self, client: &DaemonClient) -> Vec<String> {
        match self.gpu_id {
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
}
