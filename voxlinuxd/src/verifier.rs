use crate::{systemd, pacman};
use crate::system_state::SystemState;
use std::process::{Command, Stdio};

/// Verify whether a system state is currently healthy
pub fn verify(state: &SystemState) -> bool {
    match state {
        // Package is healthy if pacman is not broken
        SystemState::PackageConsistent => {
            !pacman::pacman_broken()
        }

        // Service is healthy if active
        SystemState::ServiceActive(service) => {
            systemd::is_active(service)
        }

        // Network is healthy if ping works
        SystemState::NetworkReachable => {
            let ping = Command::new("ping")
            .args(["-c", "1", "-W", "1", "1.1.1.1"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

            matches!(ping, Ok(p) if p.success())
        }

        // Filesystem is healthy if mounted rw
        SystemState::FilesystemWritable(path) => {
            let output = Command::new("mount")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .output();

            if let Ok(o) = output {
                let stdout = String::from_utf8_lossy(&o.stdout);
                stdout.contains(path) && stdout.contains("(rw")
            } else {
                false
            }
        }
    }
}
