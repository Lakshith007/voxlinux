use std::os::unix::net::UnixStream;
use std::io::Write;

const SOCKET_PATH: &str = "/run/voxlinux/voxlinux.sock";

pub fn send_apply(plan_id: &str) {
    match UnixStream::connect(SOCKET_PATH) {
        Ok(mut stream) => {
            let message = format!("APPLY:{}", plan_id);
            let _ = stream.write_all(message.as_bytes());
        }
        Err(_) => {
            println!("Failed to connect to VoxLinux daemon.");
        }
    }
}
