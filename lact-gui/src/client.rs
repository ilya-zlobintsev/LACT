use anyhow::{anyhow, Context};
use lact_schema::{request::Request, response::Response, DeviceInfo, DeviceListEntry, DeviceStats};
use nix::unistd::getuid;
use serde::de::DeserializeOwned;
use std::{
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
    path::PathBuf,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct DaemonClient {
    stream: Arc<Mutex<(BufReader<UnixStream>, UnixStream)>>,
}

impl DaemonClient {
    pub fn connect() -> anyhow::Result<Self> {
        let path =
            get_socket_path().context("Could not connect to daemon: socket file not found")?;
        let stream = UnixStream::connect(path).context("Could not connect to daemon")?;
        let reader = BufReader::new(stream.try_clone()?);

        Ok(Self {
            stream: Arc::new(Mutex::new((reader, stream))),
        })
    }

    fn make_request<T: DeserializeOwned>(&self, request: Request) -> anyhow::Result<T> {
        let (reader, writer) = *self.stream.lock().map_err(|err| anyhow!("{err}"))?;

        if !reader.buffer().is_empty() {
            return Err(anyhow!("Another request was not processed properly"));
        }

        let request_payload = serde_json::to_string(&request)?;
        writer.write_all(request_payload.as_bytes())?;
        writer.write_all(b"\n")?;

        let mut response_payload = String::new();
        reader.read_line(&mut response_payload);

        let response: Response<T> = serde_json::from_str(&response_payload)
            .context("Could not deserialize response from daemon")?;

        match response {
            Response::Ok(data) => Ok(data),
            Response::Error(error) => Err(anyhow!("Error from daemon: {error}")),
        }
    }

    pub fn list_devices<'a>(&self) -> anyhow::Result<Vec<DeviceListEntry<'a>>> {
        self.make_request(Request::ListDevices)
    }

    pub fn set_fan_control(&self, id: &str, enabled: bool) -> anyhow::Result<()> {
        self.make_request(Request::SetFanControl { id, enabled })
    }

    pub fn get_device_info(&self, id: &str) -> anyhow::Result<DeviceInfo> {
        self.make_request(Request::DeviceInfo { id })
    }

    pub fn get_device_stats(&self, id: &str) -> anyhow::Result<DeviceStats> {
        self.make_request(Request::DeviceStats { id })
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
