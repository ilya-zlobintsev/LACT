pub mod config;
pub mod daemon_connection;
pub mod gpu_controller;
pub mod hw_mon;

use config::{Config, GpuConfig};
use gpu_controller::PowerProfile;
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
    config: Config,
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
    SetPowerCap(u32, i32),
    GetPowerCap(u32),
    SetPowerProfile(u32, PowerProfile),
    Shutdown,
}

impl Daemon {
    pub fn new(unprivileged: bool) -> Daemon {
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
        let mut config = if unprivileged {
            Config::new(&config_path)
        } else {
            match Config::read_from_file(&config_path) {
                Ok(c) => c,
                Err(_) => {
                    let c = Config::new(&config_path);
                    //c.save().unwrap();
                    c
                }
            }
        };

        log::trace!("Using config {:?}", config);

        let mut gpu_controllers: HashMap<u32, GpuController> = HashMap::new();

        /*for (gpu_identifier, gpu_config) in &config.gpu_configs {
            let mut controller = GpuController::new(gpu_identifier.path.clone(), GpuConfig::new());
            if controller.gpu_info.pci_slot == gpu_identifier.pci_id && controller.gpu_info.card_model == gpu_identifier.card_model && controller.gpu_info.gpu_model == gpu_identifier.gpu_model {
                controller.load_config(gpu_config.clone());
                gpu_controllers.insert(gpu_identifier.id, controller);
            }
        }*/

        'entries: for entry in fs::read_dir("/sys/class/drm").expect("Could not open /sys/class/drm") {
            let entry = entry.unwrap();
            if entry.file_name().len() == 5 {
                if entry.file_name().to_str().unwrap().split_at(4).0 == "card" {
                    log::info!("Initializing {:?}", entry.path());

                    let mut controller = GpuController::new(entry.path().join("device"), GpuConfig::new());
                    let gpu_info = controller.get_info();

                    for (id, (gpu_identifier, gpu_config)) in &config.gpu_configs {
                        if gpu_info.pci_slot == gpu_identifier.pci_id && gpu_info.card_model == gpu_identifier.card_model && gpu_info.gpu_model == gpu_identifier.gpu_model {
                            controller.load_config(gpu_config.clone());
                            gpu_controllers.insert(id.clone(), controller);
                            log::info!("already known");
                            continue 'entries;
                        }
                    }

                    log::info!("initializing for the first time");

                    let id: u32 = random();

                    config.gpu_configs.insert(id, (controller.get_identifier(), controller.get_config()));
                    gpu_controllers.insert(id, controller);
                }
            }
        }
        if !unprivileged {
            config.save().unwrap();
        }

        Daemon {
            listener,
            gpu_controllers,
            config,
        }
    }

    pub fn listen(mut self) {
        let listener = self.listener.try_clone().expect("couldn't try_clone");
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    //let mut controller = self.gpu_controller.clone();
                    //thread::spawn(move || Daemon::handle_connection(&mut controller, stream));
                    //Daemon::handle_connection(&mut self.gpu_controllers, stream);
                    Daemon::handle_connection(&mut self, stream);
                }
                Err(err) => {
                    log::error!("Error: {}", err);
                    break;
                }
            }
        }
    }

    //fn handle_connection(gpu_controllers: &mut HashMap<u32, GpuController>, mut stream: UnixStream) {
    fn handle_connection(&mut self, mut stream: UnixStream) {
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
                        for (id, controller) in &self.gpu_controllers {
                            gpus.insert(*id, controller.get_info().gpu_model.clone());
                        }
                        Ok(DaemonResponse::Gpus(gpus))
                    },
                    Action::GetStats(i) => match self.gpu_controllers.get(&i) {
                        Some(controller) => Ok(DaemonResponse::GpuStats(controller.get_stats())),
                        None => Err(DaemonError::InvalidID),
                    },
                    Action::GetInfo(i) => match self.gpu_controllers.get(&i) {
                        Some(controller) => Ok(DaemonResponse::GpuInfo(controller.get_info())),
                        None => Err(DaemonError::InvalidID),
                    },
                    Action::StartFanControl(i) => match self.gpu_controllers.get_mut(&i) {
                        Some(controller) => match controller.start_fan_control() {
                            Ok(_) => {
                                self.config.gpu_configs.insert(i, (controller.get_identifier(), controller.get_config()));
                                self.config.save().unwrap();
                                Ok(DaemonResponse::OK)
                            },
                            Err(_) => Err(DaemonError::HWMonError),
                        }
                        None => Err(DaemonError::InvalidID),
                    },
                    Action::StopFanControl(i) => match self.gpu_controllers.get_mut(&i) {
                        Some(controller) => match controller.stop_fan_control() {
                            Ok(_) => {
                                self.config.gpu_configs.insert(i, (controller.get_identifier(), controller.get_config()));
                                self.config.save().unwrap();
                                Ok(DaemonResponse::OK)
                            },
                            Err(_) => Err(DaemonError::HWMonError),
                        },
                        None => Err(DaemonError::InvalidID),
                    },
                    Action::GetFanControl(i) => match self.gpu_controllers.get(&i) {
                        Some(controller) => match controller.get_fan_control() {
                            Ok(info) => Ok(DaemonResponse::FanControlInfo(info)),
                            Err(_) => Err(DaemonError::HWMonError),
                        }
                        None => Err(DaemonError::InvalidID),
                    }
                    Action::SetFanCurve(i, curve) => match self.gpu_controllers.get_mut(&i) {
                        Some(controller) => {
                            
                            match controller.set_fan_curve(curve) {
                                Ok(_) => {
                                    self.config.gpu_configs.insert(i, (controller.get_identifier(), controller.get_config()));
                                    self.config.save().unwrap();
                                    Ok(DaemonResponse::OK)
                                },
                                Err(_) => Err(DaemonError::HWMonError),
                            }
                            
                        },
                        None => Err(DaemonError::InvalidID),
                    }
                    Action::SetPowerCap(i, cap) => match self.gpu_controllers.get_mut(&i) {
                        Some(controller) => {
                            match controller.set_power_cap(cap) {
                                Ok(_) => {
                                    self.config.gpu_configs.insert(i, (controller.get_identifier(), controller.get_config()));
                                    self.config.save().unwrap();
                                    Ok(DaemonResponse::OK)
                                },
                                Err(_) => Err(DaemonError::HWMonError),
                            }
                        },
                        None => Err(DaemonError::InvalidID),
                    }
                    Action::GetPowerCap(i) => match self.gpu_controllers.get(&i) {
                        Some(controller) => match controller.get_power_cap() {
                            Ok(cap) => Ok(DaemonResponse::PowerCap(cap)),
                            Err(_) => Err(DaemonError::HWMonError),
                        }
                        None => Err(DaemonError::InvalidID),
                    }
                    Action::SetPowerProfile(i, profile) => match self.gpu_controllers.get_mut(&i) {
                        Some(controller) => {
                            match controller.set_power_profile(profile) {
                                Ok(_) => {
                                    self.config.gpu_configs.insert(i, (controller.get_identifier(), controller.get_config()));
                                    self.config.save().unwrap();
                                    Ok(DaemonResponse::OK)
                                },
                                Err(_) => Err(DaemonError::ControllerError)
                            }
                        },
                        None => Err(DaemonError::InvalidID),
                    }
                    Action::Shutdown => {
                        for (_, controller) in &mut self.gpu_controllers {
                            controller.stop_fan_control().expect("Failed to stop fan control");
                        }
                        std::process::exit(0);
                    }
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
    PowerCap((i32, i32)),
    FanControlInfo(gpu_controller::FanControlInfo),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DaemonError {
    ConnectionFailed,
    InvalidID,
    HWMonError,
    ControllerError,
}
