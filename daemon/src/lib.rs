pub mod gpu_controller;
pub mod hw_mon;
pub mod daemon_connection;

use std::{path::PathBuf, io::{Read, Write}};
use serde::{Deserialize, Serialize};
use std::fs;
use std::os::unix::net::{UnixListener, UnixStream};
use std::process::Command;
use std::thread;

use crate::gpu_controller::GpuController;

pub const SOCK_PATH: &str = "/tmp/amdgpu-configurator.sock";

pub struct Daemon {
    gpu_controller: GpuController,
    listener: UnixListener,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Action {
    CheckAlive,
    GetInfo,
    GetStats,
}

impl Daemon {
    pub fn new() -> Daemon {
        if fs::metadata(SOCK_PATH).is_ok() {
            fs::remove_file(SOCK_PATH).expect("Failed to take control over socket");
        }

        let listener = UnixListener::bind(SOCK_PATH).unwrap();

        Command::new("chmod")
            .arg("666")
            .arg(SOCK_PATH)
            .output()
            .expect("Failed to chmod");

        Daemon { listener, gpu_controller: GpuController::new(PathBuf::from("/sys/class/drm/card0/device"))}
    }

    pub fn listen(self) {
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    let controller = self.gpu_controller.clone();
                    thread::spawn(move || Daemon::handle_connection(controller, stream));
                }
                Err(err) => {
                    println!("Error: {}", err);
                    break;
                }
            }
        }
    }

    fn handle_connection(gpu_controller: GpuController, mut stream: UnixStream) {
        let mut buffer: [u8; 4] = [0; 4];
        stream.read(&mut buffer).unwrap();
        println!("finished reading, buffer size {}", buffer.len());
        let action: Action = bincode::deserialize(&buffer).expect("Failed to deserialize buffer");

        let response: Vec<u8> = match action {
            Action::GetStats => bincode::serialize(&gpu_controller.get_stats()).unwrap(),
            Action::GetInfo => bincode::serialize(&gpu_controller.gpu_info).unwrap(),
            Action::CheckAlive => vec![1],
        };
        println!("responding with {} bytes", response.len());

        stream
            .write_all(&response)
            .expect("Failed writing response");
    }
}



#[derive(Debug)]
pub enum DaemonError {
    ConnectionFailed,
}

/*pub fn get_gpu_stats() -> GpuStats {
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
}*/