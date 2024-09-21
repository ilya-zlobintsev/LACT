pub mod unix;

pub trait DaemonConnection {
    fn request(&mut self, payload: &str) -> anyhow::Result<String>;

    /// Establish a new connection to the same service
    fn new_connection(&self) -> anyhow::Result<Box<dyn DaemonConnection>>;
}
