pub mod config;
pub mod daemon_connection;
pub mod gpu_controller;
pub mod hw_mon;

use config::Config;
use serde::{Deserialize, Serialize};
use std::{collections::{BTreeMap, HashMap}, fs};
use std::os::unix::net::{UnixListener, UnixStream};
use std::process::Command;
use std::{
    io::{Read, Write},
    path::PathBuf,
    thread,
};
use rand::prelude::*;

use crate::gpu_controller::GpuController;

pub const SOCK_PATH: &str = "/tmp/amdgpu-configurator.sock";

pub struct Daemon {
    gpu_controllers: HashMap<u32, GpuController>,
    listener: UnixListener,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Action {
    CheckAlive,
    GetGpus,
    GetInfo(u32),
    GetStats(u32),
    StartFanControl(u32),
    StopFanControl(u32),
    GetFanControl(u32),
    SetFanCurve(u32, BTreeMap<i32, f64>),
    Shutdown,
}

impl Daemon {
    pub fn new(unpriveleged: bool) -> Daemon {
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
        let config = if unpriveleged {
            Config::new()
        } else {
            match Config::read_from_file(&config_path) {
                Ok(c) => c,
                Err(_) => {
                    let c = Config::new();
                    c.save(&config_path).expect("Failed to save config");
                    c
                }
            }
        };

        log::trace!("Using config {:?}", config);

        let mut gpu_controllers: HashMap<u32, GpuController> = HashMap::new();

        for entry in fs::read_dir("/sys/class/drm").expect("Could not open /sys/class/drm") {
            let entry = entry.unwrap();
            if entry.file_name().len() == 5 {
                if entry.file_name().to_str().unwrap().split_at(4).0 == "card" {
                    log::info!("Initializing {:?}", entry.path());
                    loop {
                        let id: u32 = random();
                        if !gpu_controllers.contains_key(&id) {
                            gpu_controllers.insert(id, GpuController::new(entry.path().join("device"), config.clone(), config_path.clone()));
                            break;
                        }
                    }
                }
            }
        }

        Daemon {
            listener,
            gpu_controllers,
        }
    }

    pub fn listen(mut self) {
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    //let mut controller = self.gpu_controller.clone();
                    //thread::spawn(move || Daemon::handle_connection(&mut controller, stream));
                    Daemon::handle_connection(&mut self.gpu_controllers, stream);
                }
                Err(err) => {
                    log::error!("Error: {}", err);
                    break;
                }
            }
        }
    }

    fn handle_connection(gpu_controllers: &mut HashMap<u32, GpuController>, mut stream: UnixStream) {
        log::trace!("Reading buffer");
        let mut buffer = Vec::<u8>::new();
        stream.read_to_end(&mut buffer).unwrap();
        //log::trace!("finished reading, buffer size {}", buffer.len());
        log::trace!("Attempting to deserialize {:?}", &buffer);
        //log::trace!("{:?}", action);

        match bincode::deserialize::<Action>(&buffer) {
            Ok(action) => {
                log::trace!("Executing action {:?}", action);
                let response: Result<DaemonResponse, DaemonError> = match action {
                    Action::CheckAlive => Ok(DaemonResponse::OK),
                    Action::GetGpus => {
                        let mut gpus: HashMap<u32, String> = HashMap::new();
                        for controller in gpu_controllers {
                            gpus.insert(*controller.0, controller.1.gpu_info.gpu_model.clone());
                        }
                        Ok(DaemonResponse::Gpus(gpus))
                    },
                    Action::GetStats(i) => match gpu_controllers.get(&i) {
                        Some(controller) => Ok(DaemonResponse::GpuStats(controller.get_stats())),
                        None => Err(DaemonError::InvalidID),
                    },
                    Action::GetInfo(i) => match gpu_controllers.get(&i) {
                        Some(controller) => Ok(DaemonResponse::GpuInfo(controller.gpu_info.clone())),
                        None => Err(DaemonError::InvalidID),
                    },
                    Action::StartFanControl(i) => match gpu_controllers.get_mut(&i) {
                        Some(controller) => match controller.start_fan_control() {
                            Ok(_) => Ok(DaemonResponse::OK),
                            Err(_) => Err(DaemonError::HWMonError),
                        }
                        None => Err(DaemonError::InvalidID),
                    },
                    Action::StopFanControl(i) => match gpu_controllers.get_mut(&i) {
                        Some(controller) => match controller.stop_fan_control() {
                            Ok(_) => Ok(DaemonResponse::OK),
                            Err(_) => Err(DaemonError::HWMonError),
                        },
                        None => Err(DaemonError::InvalidID),
                    },
                    Action::GetFanControl(i) => match gpu_controllers.get(&i) {
                        Some(controller) => Ok(DaemonResponse::FanControlInfo(controller.get_fan_control())),
                        None => Err(DaemonError::InvalidID),
                    }
                    Action::SetFanCurve(i, curve) => match gpu_controllers.get_mut(&i) {
                        Some(controller) => {

                            let mut buffer = Vec::new();
                            stream.read_to_end(&mut buffer).unwrap();
                            
                            controller.set_fan_curve(curve);
                            
                            Ok(DaemonResponse::OK)
                        },
                        None => Err(DaemonError::InvalidID),
                    }
                    Action::Shutdown => std::process::exit(0),
                };

                log::trace!("Responding");
                stream.write_all(&bincode::serialize(&response).unwrap()).expect("Failed writing response");
                //stream
                //    .shutdown(std::net::Shutdown::Write)
                //    .expect("Could not shut down");
                log::trace!("Finished responding");
            },
            Err(_) => {
                println!("Failed deserializing action");
            }
        }

    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DaemonResponse {
    OK,
    GpuInfo(gpu_controller::GpuInfo),
    GpuStats(gpu_controller::GpuStats),
    Gpus(HashMap<u32, String>),
    FanControlInfo(gpu_controller::FanControlInfo),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DaemonError {
    ConnectionFailed,
    InvalidID,
    HWMonError,
}
