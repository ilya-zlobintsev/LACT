mod connection;
#[macro_use]
mod macros;

pub use lact_schema as schema;

use amdgpu_sysfs::gpu_handle::{
    power_profile_mode::PowerProfileModesTable, PerformanceLevel, PowerLevelKind,
};
use anyhow::{anyhow, Context};
use connection::{tcp::TcpConnection, unix::UnixConnection, DaemonConnection};
use nix::unistd::getuid;
use schema::{
    request::{ConfirmCommand, SetClocksCommand},
    ClocksInfo, DeviceInfo, DeviceListEntry, DeviceStats, FanOptions, PowerStates, Request,
    Response, SystemInfo,
};
use serde::Deserialize;
use std::{
    cell::RefCell, marker::PhantomData, net::ToSocketAddrs, os::unix::net::UnixStream, path::PathBuf, rc::Rc, time::Duration
};
use tracing::{error, info};

const RECONNECT_INTERVAL_MS: u64 = 250;

#[derive(Clone)]
pub struct DaemonClient {
    stream: Rc<RefCell<Box<dyn DaemonConnection>>>,
    pub embedded: bool,
}

impl DaemonClient {
    pub fn connect() -> anyhow::Result<Self> {
        let path =
            get_socket_path().context("Could not connect to daemon: socket file not found")?;
        let stream = UnixConnection::connect(&path)?;

        Ok(Self {
            stream: Rc::new(RefCell::new(stream)),
            embedded: false,
        })
    }

    pub fn connect_tcp(addr: impl ToSocketAddrs) -> anyhow::Result<Self> {
        let stream = TcpConnection::connect(addr)?;

        Ok(Self {
            stream: Rc::new(RefCell::new(stream)),
            embedded: false,
        })
    }

    pub fn from_stream(stream: UnixStream, embedded: bool) -> anyhow::Result<Self> {
        let connection = UnixConnection::try_from(stream)?;
        Ok(Self {
            stream: Rc::new(RefCell::new(Box::new(connection))),
            embedded,
        })
    }

    fn make_request<'a, T: Deserialize<'a>>(
        &self,
        request: Request,
    ) -> anyhow::Result<ResponseBuffer<T>> {
        let mut stream = self
            .stream
            .try_borrow_mut()
            .map_err(|err| anyhow!("{err}"))?;

        let request_payload = serde_json::to_string(&request)?;
        match stream.request(&request_payload) {
            Ok(response_payload) => Ok(ResponseBuffer {
                buf: response_payload,
                _phantom: PhantomData,
            }),
            Err(err) => {
                error!("Could not make request: {err}, reconnecting to socket");

                loop {
                    match stream.new_connection() {
                        Ok(new_connection) => {
                            info!("Established new socket connection");
                            *stream = new_connection;
                            drop(stream);
                            return self.make_request(request);
                        }
                        Err(err) => {
                            error!("Could not reconnect: {err:#}, retrying in {RECONNECT_INTERVAL_MS}ms");
                            std::thread::sleep(Duration::from_millis(RECONNECT_INTERVAL_MS));
                        }
                    }
                }
            }
        }
    }

    pub fn list_devices(&self) -> anyhow::Result<ResponseBuffer<Vec<DeviceListEntry>>> {
        self.make_request(Request::ListDevices)
    }

    pub fn set_fan_control(&self, cmd: FanOptions) -> anyhow::Result<u64> {
        self.make_request(Request::SetFanControl(cmd))?.inner()
    }

    pub fn set_power_cap(&self, id: &str, cap: Option<f64>) -> anyhow::Result<u64> {
        self.make_request(Request::SetPowerCap { id, cap })?.inner()
    }

    request_plain!(get_system_info, SystemInfo, SystemInfo);
    request_plain!(enable_overdrive, EnableOverdrive, String);
    request_plain!(disable_overdrive, DisableOverdrive, String);
    request_plain!(generate_debug_snapshot, GenerateSnapshot, String);
    request_plain!(reset_config, RestConfig, ());
    request_with_id!(get_device_info, DeviceInfo, DeviceInfo);
    request_with_id!(get_device_stats, DeviceStats, DeviceStats);
    request_with_id!(get_device_clocks_info, DeviceClocksInfo, ClocksInfo);
    request_with_id!(
        get_device_power_profile_modes,
        DevicePowerProfileModes,
        PowerProfileModesTable
    );
    request_with_id!(get_power_states, GetPowerStates, PowerStates);
    request_with_id!(reset_pmfw, ResetPmfw, u64);
    request_with_id!(dump_vbios, VbiosDump, Vec<u8>);

    pub fn set_performance_level(
        &self,
        id: &str,
        performance_level: PerformanceLevel,
    ) -> anyhow::Result<u64> {
        self.make_request(Request::SetPerformanceLevel {
            id,
            performance_level,
        })?
        .inner()
    }

    pub fn set_clocks_value(&self, id: &str, command: SetClocksCommand) -> anyhow::Result<u64> {
        self.make_request(Request::SetClocksValue { id, command })?
            .inner()
    }

    pub fn batch_set_clocks_value(
        &self,
        id: &str,
        commands: Vec<SetClocksCommand>,
    ) -> anyhow::Result<u64> {
        self.make_request(Request::BatchSetClocksValue { id, commands })?
            .inner()
    }

    pub fn set_enabled_power_states(
        &self,
        id: &str,
        kind: PowerLevelKind,
        states: Vec<u8>,
    ) -> anyhow::Result<u64> {
        self.make_request(Request::SetEnabledPowerStates { id, kind, states })?
            .inner()
    }

    pub fn set_power_profile_mode(
        &self,
        id: &str,
        index: Option<u16>,
        custom_heuristics: Vec<Vec<Option<i32>>>,
    ) -> anyhow::Result<u64> {
        self.make_request(Request::SetPowerProfileMode {
            id,
            index,
            custom_heuristics,
        })?
        .inner()
    }

    pub fn confirm_pending_config(&self, command: ConfirmCommand) -> anyhow::Result<()> {
        self.make_request(Request::ConfirmPendingConfig(command))?
            .inner()
    }
}

fn get_socket_path() -> Option<PathBuf> {
    let root_path = PathBuf::from("/var/run/lactd.sock");

    if root_path.exists() {
        return Some(root_path);
    }

    let uid = getuid();
    let user_path = PathBuf::from(format!("/var/run/user/{}/lactd.sock", uid));

    if user_path.exists() {
        Some(user_path)
    } else {
        None
    }
}

pub struct ResponseBuffer<T> {
    buf: String,
    _phantom: PhantomData<T>,
}

impl<'a, T: Deserialize<'a>> ResponseBuffer<T> {
    pub fn inner(&'a self) -> anyhow::Result<T> {
        let response: Response<T> = serde_json::from_str(&self.buf)
            .context("Could not deserialize response from daemon")?;
        match response {
            Response::Ok(data) => Ok(data),
            Response::Error(err) => {
                Err(anyhow::Error::new(err)
                    .context("Got error from daemon, end of client boundary"))
            }
        }
    }
}
