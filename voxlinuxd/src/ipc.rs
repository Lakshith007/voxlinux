use std::os::unix::net::{UnixListener, UnixStream};
use std::io::{Read, Write};
use std::fs;
use std::thread;
use std::os::unix::fs::PermissionsExt;

use crate::repair_executor;
use voxlinux::repair_plan::RepairPlan;

const SOCKET_PATH: &str = "/run/voxlinux/voxlinux.sock";

pub fn start_ipc_server() {

    if let Some(parent) = std::path::Path::new(SOCKET_PATH).parent() {
        fs::create_dir_all(parent).ok();
        fs::set_permissions(parent, fs::Permissions::from_mode(0o755)).ok();
    }

    let _ = fs::remove_file(SOCKET_PATH);

    let listener = UnixListener::bind(SOCKET_PATH)
    .expect("Failed to bind socket");

    fs::set_permissions(SOCKET_PATH, fs::Permissions::from_mode(0o666)).ok();

    println!("[IPC] Listening on {}", SOCKET_PATH);

    for stream in listener.incoming() {

        match stream {

            Ok(stream) => {
                thread::spawn(|| handle_client(stream));
            }

            Err(e) => {
                eprintln!("[IPC] connection error: {}", e);
            }
        }
    }
}

fn handle_client(mut stream: UnixStream) {

    let mut buffer = [0; 512];

    match stream.read(&mut buffer) {

        Ok(size) => {

            let request =
            String::from_utf8_lossy(&buffer[..size])
            .trim()
            .to_string();

            println!("[IPC] Received: {}", request);

            if let Some(plan_id) = request.strip_prefix("APPLY:") {

                let plan_id = plan_id.trim();

                match load_plan(plan_id) {

                    Some(plan) => {

                        repair_executor::apply_plan(plan);

                        let _ = stream.write_all(b"OK");
                    }

                    None => {
                        let _ = stream.write_all(b"PLAN_NOT_FOUND");
                    }
                }

            } else {

                let _ = stream.write_all(b"UNKNOWN_COMMAND");
            }
        }

        Err(e) => {
            eprintln!("[IPC] read error: {}", e);
        }
    }
}

fn load_plan(plan_id: &str) -> Option<RepairPlan> {

    let path = format!("/run/voxlinux/plans/{}.json", plan_id);

    match fs::read_to_string(&path) {

        Ok(data) => {
            serde_json::from_str(&data).ok()
        }

        Err(e) => {

            println!("[IPC] Failed to read plan {}: {}", plan_id, e);

            None
        }
    }
}
