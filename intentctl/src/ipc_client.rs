use std::os::unix::net::UnixStream;
use std::io::{Write, Read};

const SOCKET_PATH: &str = "/run/voxlinux/voxlinux.sock";

pub fn send_apply(plan_id: &str) {

    match UnixStream::connect(SOCKET_PATH) {

        Ok(mut stream) => {

            let message = format!("APPLY:{}", plan_id);

            if let Err(e) = stream.write_all(message.as_bytes()) {
                println!("[IPC CLIENT] Failed to send request: {}", e);
                return;
            }

            let mut response = [0; 128];

            match stream.read(&mut response) {

                Ok(size) => {

                    let reply =
                    String::from_utf8_lossy(&response[..size]);

                    println!("[IPC CLIENT] daemon reply: {}", reply);
                }

                Err(e) => {
                    println!("[IPC CLIENT] Failed reading response: {}", e);
                }
            }
        }

        Err(e) => {
            println!("[IPC CLIENT] Failed to connect to daemon: {}", e);
        }
    }
}
