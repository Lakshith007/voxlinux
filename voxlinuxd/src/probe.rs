use crate::{pacman, systemd};
use crate::system_state::SystemState;
use std::process::{Command, Stdio};
use crate::state::BootContext;


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



pub fn detect_boot_context() -> BootContext {
    // Check active systemd target
    let output = Command::new("systemctl")
    .arg("get-default")
    .output();

    if let Ok(out) = output {
        let target = String::from_utf8_lossy(&out.stdout);

        if target.contains("graphical.target") {
            return BootContext::Graphical;
        }
        if target.contains("multi-user.target") {
            return BootContext::MultiUser;
        }
        if target.contains("rescue.target") {
            return BootContext::Rescue;
        }
    }

    // Fallback: check if system is fully up
    let state_output = Command::new("systemctl")
    .arg("is-system-running")
    .output();

    if let Ok(out) = state_output {
        let status = String::from_utf8_lossy(&out.stdout);

        if status.contains("running") {
            return BootContext::MultiUser;
        }
        if status.contains("starting") {
            return BootContext::EarlyBoot;
        }
    }

    BootContext::Unknown
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
