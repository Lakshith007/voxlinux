use std::os::unix::net::{UnixListener, UnixStream};
use std::io::{Read, Write};
use std::fs;
use std::thread;
use std::os::unix::fs::PermissionsExt;
use crate::repair_executor;
use crate::repair_plan::RepairPlan;

const SOCKET_PATH: &str = "/run/voxlinux/voxlinux.sock";

pub fn start_ipc_server() {
    // Ensure directory exists
    if let Some(parent) = std::path::Path::new(SOCKET_PATH).parent() {
        fs::create_dir_all(parent).ok();
    }



    // Remove stale socket
    let _ = fs::remove_file(SOCKET_PATH);

    let listener = UnixListener::bind(SOCKET_PATH)
    .expect("Failed to bind socket");
    fs::set_permissions("/run/voxlinux", fs::Permissions::from_mode(0o755)).ok();
    fs::set_permissions(SOCKET_PATH, fs::Permissions::from_mode(0o666)).ok();


    println!("[IPC] Listening on {}", SOCKET_PATH);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle_client(stream));
            }
            Err(e) => eprintln!("IPC error: {}", e),
        }
    }
}

fn handle_client(mut stream: UnixStream) {
    let mut buffer = [0; 2048];

    if let Ok(size) = stream.read(&mut buffer) {
        let request = String::from_utf8_lossy(&buffer[..size]).trim().to_string();

        println!("[IPC] Received: {}", request);

        // Expected format: APPLY:<plan_id>
        if request.starts_with("APPLY:") {
            let plan_id = request.replace("APPLY:", "").trim().to_string();

            match load_plan(&plan_id) {
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
}

fn load_plan(plan_id: &str) -> Option<RepairPlan> {
    let path = format!("/run/voxlinux/plans/{}.json", plan_id);

    if let Ok(data) = fs::read_to_string(path) {
        serde_json::from_str(&data).ok()
    } else {
        None
    }
}
