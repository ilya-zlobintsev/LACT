use crate::{Action, SOCK_PATH, gpu_controller::GpuInfo, gpu_controller::{FanControlInfo, GpuStats}, hw_mon::HWMonError};
use std::{collections::BTreeMap, io::{Read, Write}};
use std::os::unix::net::UnixStream;

#[derive(Debug)]
pub enum DaemonConnectionError {
    ConnectionFailed,
    PermissionDenied,
}

#[derive(Clone, Copy)]
pub struct DaemonConnection {
}

impl DaemonConnection {
    pub fn new() -> Result<Self, DaemonConnectionError> {
        match UnixStream::connect(SOCK_PATH) {
            Ok(mut stream) => { 
                stream.write(&bincode::serialize(&Action::CheckAlive).unwrap()).unwrap();
                let mut buffer = Vec::<u8>::new();
                stream.read_to_end(&mut buffer).unwrap();
                
                if buffer[0] == 1 {
                    Ok(DaemonConnection { })
                }
                else {
                    Err(DaemonConnectionError::ConnectionFailed)
                }
            }
            Err(_) => Err(DaemonConnectionError::ConnectionFailed),
        }
    }

    pub fn get_gpu_stats(&self) -> GpuStats {
        let mut stream = UnixStream::connect(SOCK_PATH).expect("Failed to connect to daemon");
        stream
            .write(&bincode::serialize(&Action::GetStats).unwrap())
            .unwrap();
        /*stream
            .shutdown(std::net::Shutdown::Write)
            .expect("Could not shut down");*/
    
        let mut buffer = Vec::<u8>::new();
        stream.read_to_end(&mut buffer).unwrap();
    
        bincode::deserialize(&buffer).unwrap()
    }
    
    pub fn get_gpu_info(&self) -> GpuInfo {
        let mut s = UnixStream::connect(SOCK_PATH).unwrap();
        s.write_all(&bincode::serialize(&Action::GetInfo).unwrap())
            .unwrap();
        /*s.shutdown(std::net::Shutdown::Write)
            .expect("Could not shut down");*/
    
        let mut buffer = Vec::<u8>::new();
        s.read_to_end(&mut buffer).unwrap();
    
        bincode::deserialize(&buffer).unwrap()
    }

    pub fn start_fan_control(&self) -> Result<(), DaemonConnectionError> {
        let mut s = UnixStream::connect(SOCK_PATH).unwrap();
        s.write_all(&bincode::serialize(&Action::StartFanControl).unwrap())
            .unwrap();

        let mut buffer = Vec::<u8>::new();
        s.read_to_end(&mut buffer).unwrap();

        let result: Result<(), HWMonError> = bincode::deserialize(&buffer).unwrap();
        
        match result {
            Ok(_) => Ok(()),
            Err(_) => Err(DaemonConnectionError::PermissionDenied),
        }
    }

    pub fn stop_fan_control(&self) -> Result<(), DaemonConnectionError> {
        let mut s = UnixStream::connect(SOCK_PATH).unwrap();
        s.write_all(&bincode::serialize(&Action::StopFanControl).unwrap()).unwrap();
        
        let mut buffer = Vec::<u8>::new();
        s.read_to_end(&mut buffer).unwrap();

        let result: Result<(), HWMonError> = bincode::deserialize(&buffer).unwrap();
        
        match result {
            Ok(_) => Ok(()),
            Err(_) => Err(DaemonConnectionError::PermissionDenied),
        }
    }   

    pub fn get_fan_control(&self)-> FanControlInfo {
        let mut s = UnixStream::connect(SOCK_PATH).unwrap();
        s.write_all(&bincode::serialize(&Action::GetFanControl).unwrap()).unwrap();

        let mut buffer = Vec::<u8>::new();
        s.read_to_end(&mut buffer).unwrap();

        bincode::deserialize(&buffer).unwrap()
    }

    pub fn set_fan_curve(&self, curve: BTreeMap<i32, f64>) {
        let mut s = UnixStream::connect(SOCK_PATH).unwrap();
        s.write_all(&bincode::serialize(&Action::SetFanCurve).unwrap()).unwrap();
        s.write_all(&bincode::serialize(&curve).unwrap()).unwrap();
    }

    pub fn shutdown(&self) {
        let mut s = UnixStream::connect(SOCK_PATH).unwrap();
        s.write_all(&bincode::serialize(&Action::Shutdown).unwrap()).unwrap();
    }
}