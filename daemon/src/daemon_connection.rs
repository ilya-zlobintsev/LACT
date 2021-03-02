use crate::gpu_controller::{GpuInfo, PowerProfile};
use crate::DaemonError;
use crate::{
    config::Config,
    gpu_controller::{FanControlInfo, GpuStats},
};
use crate::{Action, DaemonResponse, SOCK_PATH};
use std::collections::{BTreeMap, HashMap};
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

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

                let result: Result<DaemonResponse, DaemonResponse> =
                    bincode::deserialize(&buffer).unwrap();
                match result {
                    Ok(_) => Ok(DaemonConnection {}),
                    Err(_) => Err(DaemonError::ConnectionFailed),
                }
            }
            Err(_) => Err(DaemonError::ConnectionFailed),
        }
    }

    fn send_action(&self, action: Action) -> Result<DaemonResponse, DaemonError> {
        let mut s = UnixStream::connect(SOCK_PATH).expect("Connection to daemon dropped");

        s.write(&bincode::serialize(&action).unwrap())
            .expect("Failed to write");
        s.shutdown(std::net::Shutdown::Write)
            .expect("Could nto shut down");

        let mut buffer = Vec::<u8>::new();
        s.read_to_end(&mut buffer).unwrap();

        bincode::deserialize(&buffer).expect("failed to deserialize message")
    }

    pub fn get_gpu_stats(&self, gpu_id: u32) -> Result<GpuStats, DaemonError> {
        match self.send_action(Action::GetStats(gpu_id))? {
            DaemonResponse::GpuStats(stats) => Ok(stats),
            _ => unreachable!(),
        }
    }

    pub fn get_gpu_info(&self, gpu_id: u32) -> Result<GpuInfo, DaemonError> {
        match self.send_action(Action::GetInfo(gpu_id))? {
            DaemonResponse::GpuInfo(info) => Ok(info),
            _ => unreachable!("impossible enum variant"),
        }
    }

    pub fn start_fan_control(&self, gpu_id: u32) -> Result<(), DaemonError> {
        match self.send_action(Action::StartFanControl(gpu_id))? {
            DaemonResponse::OK => Ok(()),
            _ => Err(DaemonError::HWMonError),
        }
    }

    pub fn stop_fan_control(&self, gpu_id: u32) -> Result<(), DaemonError> {
        match self.send_action(Action::StopFanControl(gpu_id))? {
            DaemonResponse::OK => Ok(()),
            _ => Err(DaemonError::HWMonError),
        }
    }

    pub fn get_fan_control(&self, gpu_id: u32) -> Result<FanControlInfo, DaemonError> {
        match self.send_action(Action::GetFanControl(gpu_id))? {
            DaemonResponse::FanControlInfo(info) => Ok(info),
            _ => unreachable!(),
        }
    }

    pub fn set_fan_curve(&self, gpu_id: u32, curve: BTreeMap<i64, f64>) -> Result<(), DaemonError> {
        match self.send_action(Action::SetFanCurve(gpu_id, curve))? {
            DaemonResponse::OK => Ok(()),
            _ => unreachable!(),
        }
    }

    pub fn set_power_cap(&self, gpu_id: u32, cap: i64) -> Result<(), DaemonError> {
        match self.send_action(Action::SetPowerCap(gpu_id, cap))? {
            DaemonResponse::OK => Ok(()),
            _ => unreachable!(),
        }
    }

    pub fn set_power_profile(&self, gpu_id: u32, profile: PowerProfile) -> Result<(), DaemonError> {
        match self.send_action(Action::SetPowerProfile(gpu_id, profile))? {
            DaemonResponse::OK => Ok(()),
            _ => unreachable!(),
        }
    }

    /*pub fn set_gpu_power_state(&self, gpu_id: u32, num: u32, clockspeed: i64, voltage: Option<i64>) -> Result<(), DaemonError> {
        match self.send_action(Action::SetGPUPowerState(gpu_id, num, clockspeed, voltage))? {
            DaemonResponse::OK => Ok(()),
            _ => unreachable!(),
        }
    }*/

    pub fn set_gpu_max_power_state(
        &self,
        gpu_id: u32,
        clockspeed: i64,
        voltage: Option<i64>,
    ) -> Result<(), DaemonError> {
        match self.send_action(Action::SetGPUMaxPowerState(gpu_id, clockspeed, voltage))? {
            DaemonResponse::OK => Ok(()),
            _ => unreachable!(),
        }
    }

    pub fn set_vram_max_clock(&self, gpu_id: u32, clockspeed: i64) -> Result<(), DaemonError> {
        match self.send_action(Action::SetVRAMMaxClock(gpu_id, clockspeed))? {
            DaemonResponse::OK => Ok(()),
            _ => unreachable!(),
        }
    }

    pub fn commit_gpu_power_states(&self, gpu_id: u32) -> Result<(), DaemonError> {
        match self.send_action(Action::CommitGPUPowerStates(gpu_id))? {
            DaemonResponse::OK => Ok(()),
            _ => unreachable!(),
        }
    }

    pub fn reset_gpu_power_states(&self, gpu_id: u32) -> Result<(), DaemonError> {
        match self.send_action(Action::ResetGPUPowerStates(gpu_id))? {
            DaemonResponse::OK => Ok(()),
            _ => unreachable!(),
        }
    }

    pub fn get_gpus(&self) -> Result<HashMap<u32, Option<String>>, DaemonError> {
        match self.send_action(Action::GetGpus)? {
            DaemonResponse::Gpus(gpus) => Ok(gpus),
            _ => unreachable!(),
        }
    }

    pub fn shutdown(&self) {
        let mut s = UnixStream::connect(SOCK_PATH).unwrap();
        s.write_all(&bincode::serialize(&Action::Shutdown).unwrap())
            .unwrap();
    }

    pub fn get_config(&self) -> Result<Config, DaemonError> {
        match self.send_action(Action::GetConfig)? {
            DaemonResponse::Config(config) => Ok(config),
            _ => unreachable!(),
        }
    }

    pub fn set_config(&self, config: Config) -> Result<(), DaemonError> {
        match self.send_action(Action::SetConfig(config))? {
            DaemonResponse::OK => Ok(()),
            _ => unreachable!(),
        }
    }
}
