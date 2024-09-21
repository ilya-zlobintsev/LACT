use super::DaemonConnection;
use anyhow::{anyhow, Context};
use std::{
    io::{self, BufRead, BufReader, Write},
    os::unix::net::UnixStream,
    path::Path,
};
use tracing::info;

pub struct UnixConnection {
    reader: BufReader<UnixStream>,
    writer: UnixStream,
}

impl UnixConnection {
    pub fn connect(path: &Path) -> anyhow::Result<Box<Self>> {
        info!("connecting to service at {path:?}");
        let stream = UnixStream::connect(path).context("Could not connect to daemon")?;
        Ok(Box::new(stream.try_into()?))
    }
}

impl TryFrom<UnixStream> for UnixConnection {
    type Error = io::Error;

    fn try_from(writer: UnixStream) -> Result<Self, Self::Error> {
        let reader = BufReader::new(writer.try_clone()?);
        Ok(Self { reader, writer })
    }
}

impl DaemonConnection for UnixConnection {
    fn request(&mut self, payload: &str) -> anyhow::Result<String> {
        if !self.reader.buffer().is_empty() {
            return Err(anyhow!("Another request was not processed properly"));
        }

        self.writer.write_all(payload.as_bytes())?;
        self.writer.write_all(b"\n")?;

        let mut response_payload = String::new();
        self.reader.read_line(&mut response_payload)?;

        Ok(response_payload)
    }

    fn new_connection(&self) -> anyhow::Result<Box<dyn DaemonConnection>> {
        let peer_addr = self
            .writer
            .peer_addr()
            .context("Could not read peer address")?;
        let path = peer_addr
            .as_pathname()
            .context("Connected socket addr is not a path")?;

        Ok(Self::connect(path)?)
    }
}
