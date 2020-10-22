pub mod gpu_controller;
pub mod hw_mon;
pub mod daemon_connection;

use std::{path::PathBuf, io::{Read, Write}};
use serde::{Deserialize, Serialize};
use std::fs;
use std::os::unix::net::{UnixListener, UnixStream};
use std::process::Command;

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
    StartFanControl,
    StopFanControl,
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

    pub fn listen(mut self) {
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    //let controller = self.gpu_controller.clone();
                    //thread::spawn(move || Daemon::handle_connection(controller, stream));
                    Daemon::handle_connection(&mut self.gpu_controller, stream);
                }
                Err(err) => {
                    println!("Error: {}", err);
                    break;
                }
            }
        }
    }

    fn handle_connection(gpu_controller: &mut GpuController, mut stream: UnixStream) {
        let mut buffer: [u8; 4] = [0; 4];
        stream.read(&mut buffer).unwrap();
        println!("finished reading, buffer size {}", buffer.len());
        let action: Action = bincode::deserialize(&buffer).expect("Failed to deserialize buffer");
        println!("{:?}", action);

        let response: Option<Vec<u8>> = match action {
            Action::GetStats => Some(bincode::serialize(&gpu_controller.get_stats()).unwrap()),
            Action::GetInfo => Some(bincode::serialize(&gpu_controller.gpu_info).unwrap()),
            Action::StartFanControl => Some(bincode::serialize(&gpu_controller.start_fan_control()).unwrap()),
            Action::StopFanControl => Some(bincode::serialize(&gpu_controller.stop_fan_control()).unwrap()),
            Action::CheckAlive => Some(vec![1]),
        };

        if let Some(r) = &response {
            stream
                .write_all(&r)
                .expect("Failed writing response");
        }
    }
}



#[derive(Debug)]
pub enum DaemonError {
    ConnectionFailed,
}