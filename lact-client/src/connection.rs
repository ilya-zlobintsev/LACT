pub mod tcp;
pub mod unix;

use anyhow::anyhow;
use futures::future::BoxFuture;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};

pub trait DaemonConnection {
    fn request<'a>(&'a mut self, payload: &'a str) -> BoxFuture<'a, anyhow::Result<String>>;

    /// Establish a new connection to the same service
    fn new_connection(&self) -> BoxFuture<'_, anyhow::Result<Box<dyn DaemonConnection>>>;
}

async fn request(
    socket: &mut BufReader<impl AsyncRead + AsyncWrite + Unpin>,
    payload: &str,
) -> anyhow::Result<String> {
    if !socket.buffer().is_empty() {
        return Err(anyhow!("Another request was not processed properly"));
    }

    socket.write_all(payload.as_bytes()).await?;
    socket.write_all(b"\n").await?;

    let mut response_payload = String::new();
    socket.read_line(&mut response_payload).await?;

    Ok(response_payload)
}
