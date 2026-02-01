use super::{DaemonConnection, request};
use anyhow::Context;
use futures::future::BoxFuture;
use std::os::unix::net::UnixStream as StdUnixStream;
use std::path::Path;
use tokio::{io::BufReader, net::UnixStream};
use tracing::info;

pub struct UnixConnection {
    inner: BufReader<UnixStream>,
}

impl UnixConnection {
    pub async fn connect(path: &Path) -> anyhow::Result<Box<Self>> {
        info!("connecting to service at {path:?}");
        let inner = UnixStream::connect(path).await?;
        Ok(Box::new(Self {
            inner: BufReader::new(inner),
        }))
    }
}

impl From<UnixStream> for UnixConnection {
    fn from(inner: UnixStream) -> Self {
        Self {
            inner: BufReader::new(inner),
        }
    }
}

impl TryFrom<StdUnixStream> for UnixConnection {
    type Error = anyhow::Error;

    fn try_from(stream: StdUnixStream) -> Result<Self, Self::Error> {
        Ok(UnixStream::from_std(stream)?.into())
    }
}

impl DaemonConnection for UnixConnection {
    fn request<'a>(&'a mut self, payload: &'a str) -> BoxFuture<'a, anyhow::Result<String>> {
        Box::pin(async { request(&mut self.inner, payload).await })
    }

    fn new_connection(&self) -> BoxFuture<'_, anyhow::Result<Box<dyn DaemonConnection>>> {
        Box::pin(async {
            let peer_addr = self
                .inner
                .get_ref()
                .peer_addr()
                .context("Could not read peer address")?;
            let path = peer_addr
                .as_pathname()
                .context("Connected socket addr is not a path")?;

            Ok(Self::connect(path).await? as Box<dyn DaemonConnection>)
        })
    }
}
