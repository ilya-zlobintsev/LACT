use std::os::unix::net::UnixStream;
use std::io::{Read, Write};

pub mod daemon;
pub mod fan_controller;

pub const SOCK_PATH: &str = "/tmp/amdgpu-configurator.sock";

pub fn get_info() {
    let mut stream = UnixStream::connect(SOCK_PATH).expect("Failed to connect to daemon");
    stream.write_all(&bincode::serialize(&daemon::Action::GetInfo).unwrap()).unwrap();
    stream.shutdown(std::net::Shutdown::Write).expect("Could not shut down");
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    println!("{}", response);
}

pub struct DaemonConnection {

}

impl DaemonConnection {

}