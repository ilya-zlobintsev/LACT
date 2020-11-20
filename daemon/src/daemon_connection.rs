use crate::{Action, DaemonError, DaemonResponse, gpu_controller::PowerProfile, SOCK_PATH, gpu_controller::GpuInfo, gpu_controller::{FanControlInfo, GpuStats}};
use std::{collections::HashMap, os::unix::net::UnixStream}; use std::{ collections::BTreeMap, io::{Read, Write}, };

#[derive(Clone, Copy)]
pub struct DaemonConnection {}

impl DaemonConnection {
    pub fn new() -> Result<Self, DaemonError> {
        match UnixStream::connect(SOCK_PATH) {
            Ok(mut stream) => {
                stream
                    .write(&bincode::serialize(&Action::CheckAlive).unwrap())
                    .unwrap();

                stream
                    .shutdown(std::net::Shutdown::Write)
                    .expect("Could not shut down");

                let mut buffer = Vec::<u8>::new();
                stream.read_to_end(&mut buffer).unwrap();

                let result: Result<DaemonResponse, DaemonResponse> = bincode::deserialize(&buffer).unwrap();
                match result {
                    Ok(_) => Ok(DaemonConnection {}),
                    Err(_) => Err(DaemonError::ConnectionFailed),
                }
            }
            Err(_) => Err(DaemonError::ConnectionFailed),
        }
    }

    pub fn get_gpu_stats(&self, gpu_id: u32) -> Result<GpuStats, DaemonError> {
        let mut s = UnixStream::connect(SOCK_PATH).expect("Failed to connect to daemon");
        s
            .write(&bincode::serialize(&Action::GetStats(gpu_id)).unwrap())
            .unwrap();
        s.shutdown(std::net::Shutdown::Write).expect("Could not shut down");

        let mut buffer = Vec::<u8>::new();
        s.read_to_end(&mut buffer).unwrap();

        let result: Result<DaemonResponse, DaemonError> = bincode::deserialize(&buffer).unwrap();
        match result {
            Ok(r) => match r {
                DaemonResponse::GpuStats(stats) => Ok(stats),
                _ => unreachable!("impossible enum variant"),
            },
            Err(e) => Err(e),
        }
    }

    pub fn get_gpu_info(&self, gpu_id: u32) -> Result<GpuInfo, DaemonError> {
        let mut s = UnixStream::connect(SOCK_PATH).unwrap();
        s.write_all(&bincode::serialize(&Action::GetInfo(gpu_id)).unwrap())
            .unwrap();
        s.shutdown(std::net::Shutdown::Write)
            .expect("Could not shut down");
        log::trace!("Sent action, receiving response");

        let mut buffer = Vec::<u8>::new();
        s.read_to_end(&mut buffer).unwrap();
        log::trace!("Response recieved");

        let result: Result<DaemonResponse, DaemonError> = bincode::deserialize(&buffer).unwrap();
        match result {
            Ok(r) => match r {
                DaemonResponse::GpuInfo(info) => Ok(info),
                _ => unreachable!("impossible enum variant"),
            },
            Err(e) => Err(e),
        }
    }

    pub fn start_fan_control(&self, gpu_id: u32) -> Result<(), DaemonError> {
        let mut s = UnixStream::connect(SOCK_PATH).unwrap();
        s.write_all(&bincode::serialize(&Action::StartFanControl(gpu_id)).unwrap())
            .unwrap();
        s.shutdown(std::net::Shutdown::Write).expect("Could not shut down");
        log::trace!("Sent action, receiving response");

        let mut buffer = Vec::<u8>::new();
        s.read_to_end(&mut buffer).unwrap();
        log::trace!("Response recieved");

        let result: Result<DaemonResponse, DaemonError> = bincode::deserialize(&buffer).unwrap();

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
        
    }

    pub fn stop_fan_control(&self, gpu_id: u32) -> Result<(), DaemonError> {
        let mut s = UnixStream::connect(SOCK_PATH).unwrap();
        s.write_all(&bincode::serialize(&Action::StopFanControl(gpu_id)).unwrap())
            .unwrap();

        s.shutdown(std::net::Shutdown::Write).expect("Could not shut down");
        let mut buffer = Vec::<u8>::new();
        s.read_to_end(&mut buffer).unwrap();

        let result: Result<DaemonResponse, DaemonError> = bincode::deserialize(&buffer).unwrap();

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub fn get_fan_control(&self, gpu_id: u32) -> Result<FanControlInfo, DaemonError> {
        let mut s = UnixStream::connect(SOCK_PATH).unwrap();
        s.write_all(&bincode::serialize(&Action::GetFanControl(gpu_id)).unwrap())
            .unwrap();

        s.shutdown(std::net::Shutdown::Write).expect("Could not shut down");
        let mut buffer = Vec::<u8>::new();
        s.read_to_end(&mut buffer).unwrap();

        let result: Result<DaemonResponse, DaemonError> = bincode::deserialize(&buffer).unwrap();

        match result {
            Ok(r) => match r {
                DaemonResponse::FanControlInfo(info) => Ok(info),
                _ => unreachable!("impossible enum"),
            }, 
            Err(e) => Err(e),
        }
    }

    pub fn set_fan_curve(&self, gpu_id: u32, curve: BTreeMap<i32, f64>) -> Result<(), DaemonError> {
        let mut s = UnixStream::connect(SOCK_PATH).unwrap();
        s.write_all(&bincode::serialize(&Action::SetFanCurve(gpu_id, curve)).unwrap())
            .unwrap();
        s.shutdown(std::net::Shutdown::Write).expect("Could not shut down");
        let mut buffer = Vec::<u8>::new();
        s.read_to_end(&mut buffer).unwrap();

        let result: Result<DaemonResponse, DaemonError> = bincode::deserialize(&buffer).unwrap();

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub fn get_power_cap(&self, gpu_id: u32) -> Result<(i32, i32), DaemonError> {
        let mut s = UnixStream::connect(SOCK_PATH).unwrap();
        s.write_all(&bincode::serialize(&Action::GetPowerCap(gpu_id)).unwrap())
            .unwrap();
        s.shutdown(std::net::Shutdown::Write).expect("Could not shut down");
        let mut buffer = Vec::<u8>::new();
        s.read_to_end(&mut buffer).unwrap();

        let result: Result<DaemonResponse, DaemonError> = bincode::deserialize(&buffer).unwrap();

        match result {
            Ok(response) => {
                match response {
                    DaemonResponse::PowerCap(cap) => Ok(cap),
                    _ => unreachable!("invalid response"),
                }
            },
            Err(e) => Err(e),
        }
    }

    pub fn set_power_cap(&self, gpu_id: u32, cap: i32) -> Result<(), DaemonError> {
        let mut s = UnixStream::connect(SOCK_PATH).unwrap();
        s.write_all(&bincode::serialize(&Action::SetPowerCap(gpu_id, cap)).unwrap())
            .unwrap();
        s.shutdown(std::net::Shutdown::Write).expect("Could not shut down");
        let mut buffer = Vec::<u8>::new();
        s.read_to_end(&mut buffer).unwrap();

        let result: Result<DaemonResponse, DaemonError> = bincode::deserialize(&buffer).unwrap();

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub fn set_power_profile(&self, gpu_id: u32, profile: PowerProfile) -> Result<(), DaemonError> {
        let mut s = UnixStream::connect(SOCK_PATH).unwrap();
        s.write_all(&bincode::serialize(&Action::SetPowerProfile(gpu_id, profile)).unwrap())
            .unwrap();
        s.shutdown(std::net::Shutdown::Write).expect("Could not shut down");
        let mut buffer = Vec::<u8>::new();
        s.read_to_end(&mut buffer).unwrap();

        let result: Result<DaemonResponse, DaemonError> = bincode::deserialize(&buffer).unwrap();

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub fn get_gpus(&self) -> Result<HashMap<u32, String>, DaemonError> {
        log::trace!("sending request");
        let mut s = UnixStream::connect(SOCK_PATH).unwrap();
        s.write_all(&bincode::serialize(&Action::GetGpus).unwrap())
            .unwrap();

        s.shutdown(std::net::Shutdown::Write).expect("Could not shut down");
        
        log::trace!("sent request");

        let mut buffer = Vec::<u8>::new();
        s.read_to_end(&mut buffer).unwrap();
        log::trace!("read response");

        let result: Result<DaemonResponse, DaemonError> = bincode::deserialize(&buffer).unwrap();
        match result { 
            Ok(r) => match r {
                DaemonResponse::Gpus(gpus) => Ok(gpus),
                _ => unreachable!("impossible enum variant"),
            },
            Err(e) => Err(e),
        }
    }

    pub fn shutdown(&self) {
        let mut s = UnixStream::connect(SOCK_PATH).unwrap();
        s.write_all(&bincode::serialize(&Action::Shutdown).unwrap())
            .unwrap();
    }
}
