use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

use gpu_controller::{GpuInfo, GpuStats};

pub mod daemon;
pub mod fan_controller;
pub mod gpu_controller;

pub const SOCK_PATH: &str = "/tmp/amdgpu-configurator.sock";

#[derive(Debug)]
pub enum DaemonError {
    ConnectionFailed,
}

pub fn get_gpu_stats() -> GpuStats {
    let mut stream = UnixStream::connect(SOCK_PATH).expect("Failed to connect to daemon");
    stream
        .write_all(&bincode::serialize(&daemon::Action::GetStats).unwrap())
        .unwrap();
    stream
        .shutdown(std::net::Shutdown::Write)
        .expect("Could not shut down");

    let mut buffer = Vec::<u8>::new();
    stream.read_to_end(&mut buffer).unwrap();

    bincode::deserialize(&buffer).unwrap()
}

pub fn get_gpu_info() -> Result<GpuInfo, DaemonError> {
    match UnixStream::connect(SOCK_PATH) {
        Ok(mut s) => {
            s.write_all(&bincode::serialize(&daemon::Action::GetInfo).unwrap())
                .unwrap();
            s.shutdown(std::net::Shutdown::Write)
                .expect("Could not shut down");

            let mut buffer = Vec::<u8>::new();
            s.read_to_end(&mut buffer).unwrap();

            Ok(bincode::deserialize(&buffer).unwrap())
        }
        Err(_) => Err(DaemonError::ConnectionFailed),
    }

    
}
