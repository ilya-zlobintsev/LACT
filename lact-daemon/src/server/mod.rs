pub mod gpu_controller;
mod handler;
// mod pci;
mod vulkan;

use self::handler::Handler;
use crate::{config::Config, socket};
use lact_schema::{
    request::Request,
    response::{Pong, Response},
};
use serde::Serialize;
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
        let listener = socket::listen().await?;

        Ok(Self { listener, handler })
    }

    pub async fn run(self) {
        loop {
            match self.listener.accept().await {
                Ok((stream, _)) => {
                    let handler = self.handler.clone();
                    tokio::spawn(async move {
                        if let Err(error) = handle_stream(stream, handler).await {
                            error!("{error}")
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
async fn handle_stream(stream: UnixStream, handler: Handler) -> anyhow::Result<()> {
    let mut stream = BufReader::new(stream);

    let mut buf = String::new();
    while stream.read_line(&mut buf).await? != 0 {
        debug!("Handling request: {}", buf.trim_end());

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
        Request::Ping => ok_response(Pong),
        Request::ListDevices => ok_response(handler.list_devices()),
        Request::DeviceInfo { id } => ok_response(handler.get_device_info(id)?),
        Request::DeviceStats { id } => ok_response(handler.get_gpu_stats(id)?),
        Request::SetFanControl { id, enabled } => {
            ok_response(handler.set_fan_control(id, enabled).await?)
        }
        Request::SetPowerCap { id, cap } => ok_response(handler.set_power_cap(id, cap)?),
    }
}

fn ok_response<T: Serialize>(data: T) -> anyhow::Result<Vec<u8>> {
    Ok(serde_json::to_vec(&Response::Ok(data))?)
}
