use super::{request, DaemonConnection};
use anyhow::Context;
use std::{
    io::{self, BufReader},
    net::{TcpStream, ToSocketAddrs},
};
use tracing::info;

pub struct TcpConnection {
    reader: BufReader<TcpStream>,
    writer: TcpStream,
}

impl TcpConnection {
    pub fn connect(addr: impl ToSocketAddrs) -> anyhow::Result<Box<Self>> {
        info!("connecting to remote TCP service");
        let stream = TcpStream::connect(addr)?;
        Ok(Box::new(stream.try_into()?))
    }
}

impl TryFrom<TcpStream> for TcpConnection {
    type Error = io::Error;

    fn try_from(writer: TcpStream) -> Result<Self, Self::Error> {
        let reader = BufReader::new(writer.try_clone()?);
        Ok(Self { reader, writer })
    }
}

impl DaemonConnection for TcpConnection {
    fn request(&mut self, payload: &str) -> anyhow::Result<String> {
        request(&mut self.reader, &mut self.writer, payload)
    }

    fn new_connection(&self) -> anyhow::Result<Box<dyn DaemonConnection>> {
        let peer_addr = self
            .writer
            .peer_addr()
            .context("Could not read peer address")?;

        Ok(Self::connect(peer_addr)?)
    }
}
