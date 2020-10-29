pub mod gpu_controller;
pub mod hw_mon;
pub mod daemon_connection;
pub mod config;

use std::{io::{Read, Write}, path::PathBuf, thread};
use config::Config;
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
    GetFanControl,
    SetFanCurve,
    Shutdown,
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

        let config_path = PathBuf::from("/etc/lact.json");
        let config = match Config::read_from_file(&config_path) {
            Ok(c) => c,
            Err(_) => {
                let c = Config::new();
                c.save(&config_path).expect("Failed to save config");
                c
            }
        };
        log::trace!("Using config {:?}", config);

        let gpu_controller = GpuController::new(PathBuf::from("/sys/class/drm/card0/device"), config, config_path);

        Daemon { listener, gpu_controller }
    }

    pub fn listen(mut self) {
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    //let mut controller = self.gpu_controller.clone();
                    //thread::spawn(move || Daemon::handle_connection(&mut controller, stream));
                    Daemon::handle_connection(&mut self.gpu_controller, stream);
                }
                Err(err) => {
                    log::error!("Error: {}", err);
                    break;
                }
            }
        }
    }

    fn handle_connection(gpu_controller: &mut GpuController, mut stream: UnixStream) {
        let mut buffer: [u8; 4] = [0; 4];
        stream.read(&mut buffer).unwrap();
        //log::trace!("finished reading, buffer size {}", buffer.len());
        let action: Action = bincode::deserialize(&buffer).expect("Failed to deserialize buffer");
        //log::trace!("{:?}", action);

        let response: Option<Vec<u8>> = match action {
            Action::GetStats => Some(bincode::serialize(&gpu_controller.get_stats()).unwrap()),
            Action::GetInfo => Some(bincode::serialize(&gpu_controller.gpu_info).unwrap()),
            Action::StartFanControl => Some(bincode::serialize(&gpu_controller.start_fan_control()).unwrap()),
            Action::StopFanControl => Some(bincode::serialize(&gpu_controller.stop_fan_control()).unwrap()),
            Action::GetFanControl => Some(bincode::serialize(&gpu_controller.get_fan_control()).unwrap()),
            Action::SetFanCurve => {
                let mut buffer = Vec::new();
                stream.read_to_end(&mut buffer).unwrap();
                gpu_controller.set_fan_curve(bincode::deserialize(&buffer).expect("Failed to deserialize curve"));
                None
            },
            Action::CheckAlive => Some(vec![1]),
            Action::Shutdown => std::process::exit(0),
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