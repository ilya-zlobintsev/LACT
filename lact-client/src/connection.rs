pub mod tcp;
pub mod unix;

use anyhow::anyhow;
use std::io::{BufRead, BufReader, Read, Write};

pub trait DaemonConnection {
    fn request(&mut self, payload: &str) -> anyhow::Result<String>;

    /// Establish a new connection to the same service
    fn new_connection(&self) -> anyhow::Result<Box<dyn DaemonConnection>>;
}

fn request(
    reader: &mut BufReader<impl Read>,
    writer: &mut impl Write,
    payload: &str,
) -> anyhow::Result<String> {
    if !reader.buffer().is_empty() {
        return Err(anyhow!("Another request was not processed properly"));
    }

    writer.write_all(payload.as_bytes())?;
    writer.write_all(b"\n")?;

    let mut response_payload = String::new();
    reader.read_line(&mut response_payload)?;

    Ok(response_payload)
}
