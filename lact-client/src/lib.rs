mod connection;
#[macro_use]
mod macros;

pub use lact_schema as schema;
use lact_schema::{
    config::{GpuConfig, Profile},
    ProfileRule,
};

use amdgpu_sysfs::gpu_handle::power_profile_mode::PowerProfileModesTable;
use anyhow::Context;
use connection::{tcp::TcpConnection, unix::UnixConnection, DaemonConnection};
use nix::unistd::getuid;
use schema::{
    request::{ConfirmCommand, ProfileBase, SetClocksCommand},
    ClocksInfo, DeviceInfo, DeviceListEntry, DeviceStats, PowerStates, ProfilesInfo, Request,
    Response, SystemInfo,
};
use serde::de::DeserializeOwned;
use std::{
    future::Future, os::unix::net::UnixStream, path::PathBuf, pin::Pin, rc::Rc, time::Duration,
};
use tokio::{
    net::ToSocketAddrs,
    sync::{broadcast, Mutex},
};
use tracing::{error, info, trace};

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

    fn make_request<'a, T: DeserializeOwned>(
        &'a self,
        request: Request<'a>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<T>> + 'a>> {
        Box::pin(async {
            let mut stream = self.stream.lock().await;

            let request_payload = serde_json::to_string(&request)?;
            trace!("sending request {request_payload}");

            match stream.request(&request_payload).await {
                Ok(response_payload) => {
                    let response: Response<T> = serde_json::from_str(&response_payload)
                        .context("Could not deserialize response from daemon")?;
                    match response {
                        Response::Ok(data) => Ok(data),
                        Response::Error(err) => Err(anyhow::Error::new(err)
                            .context("Got error from daemon, end of client boundary")),
                    }
                }
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

    pub async fn list_devices(&self) -> anyhow::Result<Vec<DeviceListEntry>> {
        self.make_request(Request::ListDevices).await
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
            .await
    }

    pub async fn get_profile(&self, name: Option<String>) -> anyhow::Result<Option<Profile>> {
        self.make_request(Request::GetProfile { name }).await
    }

    pub async fn set_profile(&self, name: Option<String>, auto_switch: bool) -> anyhow::Result<()> {
        self.make_request(Request::SetProfile { name, auto_switch })
            .await
    }

    pub async fn create_profile(&self, name: String, base: ProfileBase) -> anyhow::Result<()> {
        self.make_request(Request::CreateProfile { name, base })
            .await
    }

    pub async fn delete_profile(&self, name: String) -> anyhow::Result<()> {
        self.make_request(Request::DeleteProfile { name }).await
    }

    pub async fn move_profile(&self, name: String, new_position: usize) -> anyhow::Result<()> {
        self.make_request(Request::MoveProfile { name, new_position })
            .await
    }

    pub async fn evaluate_profile_rule(&self, rule: ProfileRule) -> anyhow::Result<bool> {
        self.make_request(Request::EvaluateProfileRule { rule })
            .await
    }

    pub async fn get_gpu_config(&self, id: &str) -> anyhow::Result<Option<GpuConfig>> {
        self.make_request(Request::GetGpuConfig { id }).await
    }

    pub async fn set_gpu_config(&self, id: &str, config: GpuConfig) -> anyhow::Result<u64> {
        self.make_request(Request::SetGpuConfig { id, config })
            .await
    }

    pub async fn set_clocks_value(
        &self,
        id: &str,
        command: SetClocksCommand,
    ) -> anyhow::Result<u64> {
        self.make_request(Request::SetClocksValue { id, command })
            .await
    }

    pub async fn set_profile_rule(
        &self,
        name: String,
        rule: Option<ProfileRule>,
    ) -> anyhow::Result<()> {
        self.make_request(Request::SetProfileRule { name, rule })
            .await
    }

    pub async fn confirm_pending_config(&self, command: ConfirmCommand) -> anyhow::Result<()> {
        self.make_request(Request::ConfirmPendingConfig(command))
            .await
    }
}

fn get_socket_path() -> Option<PathBuf> {
    let root_path = PathBuf::from("/run/lactd.sock");

    if root_path.exists() {
        return Some(root_path);
    }

    let uid = getuid();
    let user_path = PathBuf::from(format!("/run/user/{}/lactd.sock", uid));

    if user_path.exists() {
        Some(user_path)
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ConnectionStatusMsg {
    Disconnected,
    Reconnected,
}
