pub mod gpu_controller;
pub mod handler;
// mod pci;
mod vulkan;

use self::handler::Handler;
use crate::{config::Config, socket};
use anyhow::Context;
use lact_schema::{Pong, Request, Response, SystemInfo};
use serde::Serialize;
use std::{fs, process::Command};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{UnixListener, UnixStream},
};
use tracing::{debug, error, instrument};

pub struct Server {
    pub handler: Handler,
    listener: UnixListener,
}

impl Server {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let handler = Handler::new(config).await?;
        let listener = socket::listen()?;

        Ok(Self { handler, listener })
    }

    pub async fn run(self) {
        loop {
            match self.listener.accept().await {
                Ok((stream, _)) => {
                    let handler = self.handler.clone();
                    tokio::spawn(async move {
                        if let Err(error) = handle_stream(stream, handler).await {
                            error!("{error}");
                        }
                    });
                }
                Err(error) => {
                    error!("failed to handle connection: {error}");
                }
            }
        }
    }
}

#[instrument(level = "debug", skip(stream, handler))]
pub async fn handle_stream(stream: UnixStream, handler: Handler) -> anyhow::Result<()> {
    let mut stream = BufReader::new(stream);

    let mut buf = String::new();
    while stream.read_line(&mut buf).await? != 0 {
        debug!("handling request: {}", buf.trim_end());

        let maybe_request = serde_json::from_str(&buf);
        let response = match maybe_request {
            Ok(request) => match handle_request(request, &handler).await {
                Ok(response) => response,
                Err(error) => serde_json::to_vec(&Response::<()>::Error(format!("{error:#}")))?,
            },
            Err(error) => serde_json::to_vec(&Response::<()>::Error(format!(
                "Failed to deserialize request: {error}"
            )))?,
        };

        stream.write_all(&response).await?;
        stream.write_all(b"\n").await?;

        buf.clear();
    }

    Ok(())
}

#[instrument(level = "debug", skip(handler))]
async fn handle_request<'a>(request: Request<'a>, handler: &'a Handler) -> anyhow::Result<Vec<u8>> {
    match request {
        Request::Ping => ok_response(ping()),
        Request::SystemInfo => ok_response(system_info()?),
        Request::ListDevices => ok_response(handler.list_devices()),
        Request::DeviceInfo { id } => ok_response(handler.get_device_info(id)?),
        Request::DeviceStats { id } => ok_response(handler.get_gpu_stats(id)?),
        Request::DeviceClocksInfo { id } => ok_response(handler.get_clocks_info(id)?),
        Request::SetFanControl { id, enabled, curve } => {
            ok_response(handler.set_fan_control(id, enabled, curve).await?)
        }
        Request::SetPowerCap { id, cap } => ok_response(handler.set_power_cap(id, cap).await?),
        Request::SetPerformanceLevel {
            id,
            performance_level,
        } => ok_response(handler.set_performance_level(id, performance_level).await?),
    }
}

fn ok_response<T: Serialize>(data: T) -> anyhow::Result<Vec<u8>> {
    Ok(serde_json::to_vec(&Response::Ok(data))?)
}

fn ping() -> Pong {
    Pong
}

fn system_info() -> anyhow::Result<SystemInfo<'static>> {
    let version = env!("CARGO_PKG_VERSION");
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    let kernel_output = Command::new("uname")
        .arg("-r")
        .output()
        .context("Could not read kernel version")?;
    let kernel_version = String::from_utf8(kernel_output.stdout)
        .context("Invalid kernel version output")?
        .trim()
        .to_owned();

    let amdgpu_overdrive_enabled = if let Ok(ppfeaturemask) =
        fs::read_to_string("/sys/module/amdgpu/parameters/ppfeaturemask")
    {
        const PP_OVERDRIVE_MASK: i32 = 0x4000;

        let ppfeaturemask = ppfeaturemask
            .trim()
            .strip_prefix("0x")
            .context("Invalid ppfeaturemask")?;
        let ppfeaturemask: u64 =
            u64::from_str_radix(ppfeaturemask, 16).context("Invalid ppfeaturemask")?;

        Some((ppfeaturemask & PP_OVERDRIVE_MASK as u64) > 0)
    } else {
        None
    };

    Ok(SystemInfo {
        version,
        profile,
        kernel_version,
        amdgpu_overdrive_enabled,
    })
}
