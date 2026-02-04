use crate::{pacman, systemd};
use crate::system_state::SystemState;
use std::process::{Command, Stdio};

/// Detect which system states are currently broken
pub fn detect_broken_states() -> Vec<SystemState> {
    let mut broken = Vec::new();

    // ─────────────────────────────
    // Package consistency
    // ─────────────────────────────
    if pacman::pacman_broken() {
        broken.push(SystemState::PackageConsistent);
    }

    // ─────────────────────────────
    // systemd services
    // ─────────────────────────────
    if let Some(service) = systemd::check() {
        broken.push(SystemState::ServiceActive(service));
    }

    // ─────────────────────────────
    // Network reachability
    // ─────────────────────────────
    if !network_ok() {
        broken.push(SystemState::NetworkReachable);
    }

    // ─────────────────────────────
    // Filesystem writable (root)
    // ─────────────────────────────
    if !filesystem_rw("/") {
        broken.push(SystemState::FilesystemWritable("/".into()));
    }

    broken
}

/// Check if network is usable (ping + DNS)
fn network_ok() -> bool {
    let ping = Command::new("ping")
    .args(["-c", "1", "-W", "1", "1.1.1.1"])
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status();

    let dns = Command::new("getent")
    .args(["hosts", "archlinux.org"])
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status();

    matches!(ping, Ok(p) if p.success())
    && matches!(dns, Ok(d) if d.success())
}

/// Check if filesystem is mounted read-write
fn filesystem_rw(path: &str) -> bool {
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


pub fn disk_full() -> bool {
    false
}

pub fn boot_degraded() -> bool {
    false
}
