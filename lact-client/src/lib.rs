mod connection;
#[macro_use]
mod macros;

pub use lact_schema as schema;
use lact_schema::ProfileRule;

use amdgpu_sysfs::gpu_handle::{
    power_profile_mode::PowerProfileModesTable, PerformanceLevel, PowerLevelKind,
};
use anyhow::Context;
use connection::{tcp::TcpConnection, unix::UnixConnection, DaemonConnection};
use nix::unistd::getuid;
use schema::{
    request::{ConfirmCommand, ProfileBase, SetClocksCommand},
    ClocksInfo, DeviceInfo, DeviceListEntry, DeviceStats, FanOptions, PowerStates, ProfilesInfo,
    Request, Response, SystemInfo,
};
use serde::Deserialize;
use std::{
    future::Future, marker::PhantomData, os::unix::net::UnixStream, path::PathBuf, pin::Pin,
    rc::Rc, time::Duration,
};
use tokio::{
    net::ToSocketAddrs,
    sync::{broadcast, Mutex},
};
use tracing::{error, info};

const STATUS_MSG_CHANNEL_SIZE: usize = 16;
const RECONNECT_INTERVAL_MS: u64 = 250;

#[derive(Clone)]
pub struct DaemonClient {
    stream: Rc<Mutex<Box<dyn DaemonConnection>>>,
    status_tx: broadcast::Sender<ConnectionStatusMsg>,
    pub embedded: bool,
}

impl DaemonClient {
    pub async fn connect() -> anyhow::Result<Self> {
        let path =
            get_socket_path().context("Could not connect to daemon: socket file not found")?;
        let stream = UnixConnection::connect(&path).await?;

        Ok(Self {
            stream: Rc::new(Mutex::new(stream)),
            embedded: false,
            status_tx: broadcast::Sender::new(STATUS_MSG_CHANNEL_SIZE),
        })
    }

    pub async fn connect_tcp(addr: impl ToSocketAddrs) -> anyhow::Result<Self> {
        let stream = TcpConnection::connect(addr).await?;

        Ok(Self {
            stream: Rc::new(Mutex::new(stream)),
            embedded: false,
            status_tx: broadcast::Sender::new(STATUS_MSG_CHANNEL_SIZE),
        })
    }

    pub fn from_stream(stream: UnixStream, embedded: bool) -> anyhow::Result<Self> {
        let connection = UnixConnection::try_from(stream)?;
        Ok(Self {
            stream: Rc::new(Mutex::new(Box::new(connection))),
            embedded,
            status_tx: broadcast::Sender::new(STATUS_MSG_CHANNEL_SIZE),
        })
    }

    pub fn status_receiver(&self) -> broadcast::Receiver<ConnectionStatusMsg> {
        self.status_tx.subscribe()
    }

    fn make_request<'a, 'r, T: Deserialize<'r>>(
        &'a self,
        request: Request<'a>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<ResponseBuffer<T>>> + 'a>> {
        Box::pin(async {
            let mut stream = self.stream.lock().await;

            let request_payload = serde_json::to_string(&request)?;
            match stream.request(&request_payload).await {
                Ok(response_payload) => Ok(ResponseBuffer {
                    buf: response_payload,
                    _phantom: PhantomData,
                }),
                Err(err) => {
                    error!("Could not make request: {err}, reconnecting to socket");
                    let _ = self.status_tx.send(ConnectionStatusMsg::Disconnected);

                    loop {
                        match stream.new_connection().await {
                            Ok(new_connection) => {
                                info!("Established new socket connection");
                                *stream = new_connection;
                                drop(stream);

                                let _ = self.status_tx.send(ConnectionStatusMsg::Reconnected);

                                return self.make_request(request).await;
                            }
                            Err(err) => {
                                error!("Could not reconnect: {err:#}, retrying in {RECONNECT_INTERVAL_MS}ms");
                                tokio::time::sleep(Duration::from_millis(RECONNECT_INTERVAL_MS))
                                    .await;
                            }
                        }
                    }
                }
            }
        })
    }

    pub async fn list_devices(&self) -> anyhow::Result<ResponseBuffer<Vec<DeviceListEntry>>> {
        self.make_request(Request::ListDevices).await
    }

    pub async fn set_fan_control(&self, cmd: FanOptions<'_>) -> anyhow::Result<u64> {
        self.make_request(Request::SetFanControl(cmd))
            .await?
            .inner()
    }

    pub async fn set_power_cap(&self, id: &str, cap: Option<f64>) -> anyhow::Result<u64> {
        self.make_request(Request::SetPowerCap { id, cap })
            .await?
            .inner()
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

    pub async fn list_profiles(&self, include_state: bool) -> anyhow::Result<ProfilesInfo> {
        self.make_request(Request::ListProfiles { include_state })
            .await?
            .inner()
    }

    pub async fn set_profile(&self, name: Option<String>, auto_switch: bool) -> anyhow::Result<()> {
        self.make_request(Request::SetProfile { name, auto_switch })
            .await?
            .inner()
    }

    pub async fn create_profile(&self, name: String, base: ProfileBase) -> anyhow::Result<()> {
        self.make_request(Request::CreateProfile { name, base })
            .await?
            .inner()
    }

    pub async fn delete_profile(&self, name: String) -> anyhow::Result<()> {
        self.make_request(Request::DeleteProfile { name })
            .await?
            .inner()
    }

    pub async fn move_profile(&self, name: String, new_position: usize) -> anyhow::Result<()> {
        self.make_request(Request::MoveProfile { name, new_position })
            .await?
            .inner()
    }

    pub async fn evaluate_profile_rule(&self, rule: ProfileRule) -> anyhow::Result<bool> {
        self.make_request(Request::EvaluateProfileRule { rule })
            .await?
            .inner()
    }

    pub async fn set_profile_rule(
        &self,
        name: String,
        rule: Option<ProfileRule>,
    ) -> anyhow::Result<()> {
        self.make_request(Request::SetProfileRule { name, rule })
            .await?
            .inner()
    }

    pub async fn set_performance_level(
        &self,
        id: &str,
        performance_level: PerformanceLevel,
    ) -> anyhow::Result<u64> {
        self.make_request(Request::SetPerformanceLevel {
            id,
            performance_level,
        })
        .await?
        .inner()
    }

    pub async fn set_clocks_value(
        &self,
        id: &str,
        command: SetClocksCommand,
    ) -> anyhow::Result<u64> {
        self.make_request(Request::SetClocksValue { id, command })
            .await?
            .inner()
    }

    pub async fn batch_set_clocks_value(
        &self,
        id: &str,
        commands: Vec<SetClocksCommand>,
    ) -> anyhow::Result<u64> {
        self.make_request(Request::BatchSetClocksValue { id, commands })
            .await?
            .inner()
    }

    pub async fn set_enabled_power_states(
        &self,
        id: &str,
        kind: PowerLevelKind,
        states: Vec<u8>,
    ) -> anyhow::Result<u64> {
        self.make_request(Request::SetEnabledPowerStates { id, kind, states })
            .await?
            .inner()
    }

    pub async fn set_power_profile_mode(
        &self,
        id: &str,
        index: Option<u16>,
        custom_heuristics: Vec<Vec<Option<i32>>>,
    ) -> anyhow::Result<u64> {
        self.make_request(Request::SetPowerProfileMode {
            id,
            index,
            custom_heuristics,
        })
        .await?
        .inner()
    }

    pub async fn confirm_pending_config(&self, command: ConfirmCommand) -> anyhow::Result<()> {
        self.make_request(Request::ConfirmPendingConfig(command))
            .await?
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

#[derive(Debug, Clone, Copy)]
pub enum ConnectionStatusMsg {
    Disconnected,
    Reconnected,
}
