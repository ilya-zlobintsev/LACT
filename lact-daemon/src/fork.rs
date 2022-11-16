use anyhow::{anyhow, Context};
use nix::{
    sys::wait::waitpid,
    unistd::{fork, ForkResult},
};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    fmt::Debug,
    io::{BufReader, Read, Write},
    mem::size_of,
    os::unix::net::UnixStream,
};
use tracing::trace;

pub unsafe fn run_forked<T, F>(f: F) -> anyhow::Result<T>
where
    T: Serialize + DeserializeOwned + Debug,
    F: FnOnce() -> Result<T, String>,
{
    let (rx, mut tx) = UnixStream::pair()?;
    let mut rx = BufReader::new(rx);

    match fork()? {
        ForkResult::Parent { child } => {
            trace!("Waiting for message from child");

            let mut size_buf = [0u8; size_of::<usize>()];
            rx.read_exact(&mut size_buf)?;
            let size = usize::from_ne_bytes(size_buf);

            let mut data_buf = vec![0u8; size];
            rx.read_exact(&mut data_buf)?;

            trace!("Received {} data bytes from child", data_buf.len());

            waitpid(child, None)?;

            let data: Result<T, String> = bincode::deserialize(&data_buf)
                .context("Could not deserialize response from child")?;

            data.map_err(|err| anyhow!("{err}"))
        }
        ForkResult::Child => {
            let response = f();
            trace!("Sending response to parent: {response:?}");

            let send_result = (|| {
                let data = bincode::serialize(&response)?;
                tx.write_all(&data.len().to_ne_bytes())?;
                tx.write_all(&data)?;
                Ok::<_, anyhow::Error>(())
            })();

            let exit_code = match send_result {
                Ok(()) => 0,
                Err(_) => 1,
            };
            trace!("Exiting child with code {exit_code}");
            std::process::exit(exit_code);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::run_forked;

    #[test]
    fn basic() {
        let response = unsafe { run_forked(|| Ok(String::from("hello"))).unwrap() };
        assert_eq!(response, "hello");
    }

    #[test]
    fn error() {
        let response =
            unsafe { run_forked::<(), _>(|| Err("something went wrong".to_owned())) }.unwrap_err();
        assert_eq!(response.to_string(), "something went wrong");
    }

    #[test]
    fn vec() {
        let response = unsafe {
            run_forked(|| {
                let mut data = Vec::new();
                data.push("hello".to_owned());
                data.push("world".to_owned());
                data.push("123".to_owned());
                Ok(data)
            })
            .unwrap()
        };
        assert_eq!(response, vec!["hello", "world", "123"])
    }

    #[test]
    fn pci_db() {
        let db = unsafe {
            run_forked(|| pciid_parser::Database::read().map_err(|err| err.to_string())).unwrap()
        };
        assert_ne!(db.classes.len(), 0);
    }
}
