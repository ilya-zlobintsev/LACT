use std::os::unix::net::{UnixStream, UnixListener};
use std::io::{Read, Write};
use std::process::Command;
use std::thread;
use std::fs;
use serde::{Serialize, Deserialize};

use crate::SOCK_PATH;
use crate::fan_controller::FanController;

#[derive(Clone)]
pub struct Daemon {
    fan_controller: FanController,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Action {
    GetInfo,
    GetStats,
}

impl Daemon {
    pub fn new(fan_controller: FanController) -> Daemon {
        Daemon {
            fan_controller,
        }
    }

    pub fn run(self) {
        if fs::metadata(SOCK_PATH).is_ok() {
            fs::remove_file(SOCK_PATH).expect("Failed to take control over socket");
        }

        let listener = UnixListener::bind(SOCK_PATH).unwrap();
        
        Command::new("chmod").arg("666").arg(SOCK_PATH).output().expect("Failed to chmod");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    thread::spawn(|| Daemon::handle_connection(stream));
                }
                Err(err) => {
                    println!("Error: {}", err);
                    break;
                }
            }
        }
    }

    fn handle_connection(mut stream: UnixStream) {
        let mut buffer = Vec::<u8>::new();
        stream.read_to_end(&mut buffer).unwrap();
        println!("finished reading");
        let action: Action = bincode::deserialize(&buffer).unwrap();
        
        let response = match action {
            Action::GetInfo => "gpu information",
            Action::GetStats => {
                "gpu stats"
            },
        };
        stream.write_all(response.as_bytes()).unwrap();
    
    }

}
