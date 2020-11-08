use crate::{
    gpu_controller::GpuInfo,
    gpu_controller::{FanControlInfo, GpuStats},
    hw_mon::HWMonError,
    Action, SOCK_PATH,
};
use std::{collections::HashMap, os::unix::net::UnixStream};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
};

#[derive(Debug)]
pub enum DaemonConnectionError {
    ConnectionFailed,
    PermissionDenied,
}

#[derive(Clone, Copy)]
pub struct DaemonConnection {}

impl DaemonConnection {
    pub fn new() -> Result<Self, DaemonConnectionError> {
        match UnixStream::connect(SOCK_PATH) {
            Ok(mut stream) => {
                stream
                    .write(&bincode::serialize(&Action::CheckAlive).unwrap())
                    .unwrap();

                stream
                    .shutdown(std::net::Shutdown::Write)
                    .expect("Could not shut down");

                let mut buffer: [u8; 1] = [0; 1];
                stream.read(&mut buffer).unwrap();

                if buffer[0] == 1 {
                    Ok(DaemonConnection {})
                } else {
                    Err(DaemonConnectionError::ConnectionFailed)
                }
            }
            Err(_) => Err(DaemonConnectionError::ConnectionFailed),
        }
    }

    pub fn get_gpu_stats(&self, gpu_id: u32) -> GpuStats {
        let mut s = UnixStream::connect(SOCK_PATH).expect("Failed to connect to daemon");
        s
            .write(&bincode::serialize(&Action::GetStats(gpu_id)).unwrap())
            .unwrap();
        s.shutdown(std::net::Shutdown::Write).expect("Could not shut down");

        let mut buffer = Vec::<u8>::new();
        s.read_to_end(&mut buffer).unwrap();

        bincode::deserialize(&buffer).unwrap()
    }

    pub fn get_gpu_info(&self, gpu_id: u32) -> GpuInfo {
        let mut s = UnixStream::connect(SOCK_PATH).unwrap();
        s.write_all(&bincode::serialize(&Action::GetInfo(gpu_id)).unwrap())
            .unwrap();
        s.shutdown(std::net::Shutdown::Write)
            .expect("Could not shut down");
        log::trace!("Sent action, receiving response");

        let mut buffer = Vec::<u8>::new();
        s.read_to_end(&mut buffer).unwrap();
        log::trace!("Response recieved");

        bincode::deserialize(&buffer).unwrap()
    }

    pub fn start_fan_control(&self, gpu_id: u32) -> Result<(), DaemonConnectionError> {
        let mut s = UnixStream::connect(SOCK_PATH).unwrap();
        s.write_all(&bincode::serialize(&Action::StartFanControl(gpu_id)).unwrap())
            .unwrap();
        s.shutdown(std::net::Shutdown::Write).expect("Could not shut down");
        log::trace!("Sent action, receiving response");

        let mut buffer = Vec::<u8>::new();
        s.read_to_end(&mut buffer).unwrap();
        log::trace!("Response recieved");

        let result: Result<(), HWMonError> = bincode::deserialize(&buffer).unwrap();

        match result {
            Ok(_) => Ok(()),
            Err(_) => Err(DaemonConnectionError::PermissionDenied),
        }
    }

    pub fn stop_fan_control(&self, gpu_id: u32) -> Result<(), DaemonConnectionError> {
        let mut s = UnixStream::connect(SOCK_PATH).unwrap();
        s.write_all(&bincode::serialize(&Action::StopFanControl(gpu_id)).unwrap())
            .unwrap();

        s.shutdown(std::net::Shutdown::Write).expect("Could not shut down");
        let mut buffer = Vec::<u8>::new();
        s.read_to_end(&mut buffer).unwrap();

        let result: Result<(), HWMonError> = bincode::deserialize(&buffer).unwrap();

        match result {
            Ok(_) => Ok(()),
            Err(_) => Err(DaemonConnectionError::PermissionDenied),
        }
    }

    pub fn get_fan_control(&self, gpu_id: u32) -> FanControlInfo {
        let mut s = UnixStream::connect(SOCK_PATH).unwrap();
        s.write_all(&bincode::serialize(&Action::GetFanControl(gpu_id)).unwrap())
            .unwrap();

        s.shutdown(std::net::Shutdown::Write).expect("Could not shut down");
        let mut buffer = Vec::<u8>::new();
        s.read_to_end(&mut buffer).unwrap();

        bincode::deserialize(&buffer).unwrap()
    }

    pub fn set_fan_curve(&self, gpu_id: u32, curve: BTreeMap<i32, f64>) {
        let mut s = UnixStream::connect(SOCK_PATH).unwrap();
        s.write_all(&bincode::serialize(&Action::SetFanCurve(gpu_id)).unwrap())
            .unwrap();
        s.shutdown(std::net::Shutdown::Write).expect("Could not shut down");
        s.write_all(&bincode::serialize(&curve).unwrap()).unwrap();
    }

    pub fn get_gpus(&self) -> HashMap<u32, String> {
        log::trace!("sending request");
        let mut s = UnixStream::connect(SOCK_PATH).unwrap();
        s.write_all(&bincode::serialize(&Action::GetGpus).unwrap())
            .unwrap();

        s.shutdown(std::net::Shutdown::Write).expect("Could not shut down");
        
        log::trace!("sent request");

        let mut buffer = Vec::<u8>::new();
        s.read_to_end(&mut buffer).unwrap();
        log::trace!("read response");

        bincode::deserialize(&buffer).unwrap()
    }

    pub fn shutdown(&self) {
        let mut s = UnixStream::connect(SOCK_PATH).unwrap();
        s.write_all(&bincode::serialize(&Action::Shutdown).unwrap())
            .unwrap();
    }
}
