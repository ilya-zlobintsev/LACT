use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::process::Command;
use std::thread;

use crate::gpu_controller::GpuController;
use crate::SOCK_PATH;

#[derive(Clone)]
pub struct Daemon {
    gpu_controller: GpuController,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Action {
    GetInfo,
    GetStats,
}

impl Daemon {
    pub fn new(gpu_controller: GpuController) -> Daemon {
        Daemon { gpu_controller }
    }

    pub fn run(self) {
        if fs::metadata(SOCK_PATH).is_ok() {
            fs::remove_file(SOCK_PATH).expect("Failed to take control over socket");
        }

        let listener = UnixListener::bind(SOCK_PATH).unwrap();

        Command::new("chmod")
            .arg("666")
            .arg(SOCK_PATH)
            .output()
            .expect("Failed to chmod");

        for stream in listener.incoming() {
            let d = self.clone();
            match stream {
                Ok(stream) => {
                    thread::spawn(move || d.handle_connection(stream));
                }
                Err(err) => {
                    println!("Error: {}", err);
                    break;
                }
            }
        }
    }

    fn handle_connection(self, mut stream: UnixStream) {
        let mut buffer = Vec::<u8>::new();
        stream.read_to_end(&mut buffer).unwrap();
        println!("finished reading, buffer size {}", buffer.len());
        let action: Action = bincode::deserialize(&buffer).unwrap();

        let response: Vec<u8> = match action {
            Action::GetStats => bincode::serialize(&self.gpu_controller.get_stats()).unwrap(),
            Action::GetInfo => bincode::serialize(&self.gpu_controller.gpu_info).unwrap(),
        };
        stream
            .write_all(&response)
            .expect("Failed writing response");
    }
}
