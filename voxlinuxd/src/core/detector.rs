use std::process::Command;
use std::fs;
use crate::state::BootContext;



#[derive(Debug)]
pub struct RawDetection {
    pub unit: String,
    pub status: String,
}

pub fn detect_boot_context() -> BootContext {
    if in_initramfs() {
        return BootContext::EarlyBoot;
    }

    if !pid1_is_systemd() {
        return BootContext::Unknown;
    }

    if rescue_target_active() {
        return BootContext::Rescue;
    }

    match systemd_state() {
        Some(state) => match state.as_str() {
            "initializing" | "starting" => BootContext::EarlyUserspace,
            "running" | "degraded" => {
                if graphical_target_active() {
                    BootContext::Graphical
                } else {
                    BootContext::MultiUser
                }
            }
            _ => BootContext::Unknown,
        },
        None => BootContext::Unknown,
    }
}

/* --- helpers --- */

fn in_initramfs() -> bool {
    fs::metadata("/run/initramfs").is_ok()
}

fn pid1_is_systemd() -> bool {
    fs::read_to_string("/proc/1/comm")
    .map(|s| s.trim() == "systemd")
    .unwrap_or(false)
}

fn systemd_state() -> Option<String> {
    let out = Command::new("systemctl")
    .arg("is-system-running")
    .output()
    .ok()?;

    if !out.status.success() {
        return None;
    }

    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn rescue_target_active() -> bool {
    Command::new("systemctl")
    .args(["is-active", "rescue.target"])
    .output()
    .map(|o| o.status.success())
    .unwrap_or(false)
}

fn graphical_target_active() -> bool {
    Command::new("systemctl")
    .args(["is-active", "graphical.target"])
    .output()
    .map(|o| o.status.success())
    .unwrap_or(false)
}

pub fn scan() -> Vec<RawDetection> {
    let output = Command::new("systemctl")
    .args(["list-units", "--failed", "--no-legend"])
    .output()
    .expect("failed to run systemctl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    stdout
    .lines()
    .filter_map(|line| {
        // Expected format:
        // ● UNIT LOAD ACTIVE SUB DESCRIPTION
        let mut parts = line.split_whitespace();

        let marker = parts.next()?; // ●
        let unit = parts.next()?;   // getty@tty1.service
        let load = parts.next()?;   // loaded
        let active = parts.next()?; // failed
        let sub = parts.next()?;    // failed

        Some(RawDetection {
            unit: unit.to_string(),
             status: format!("{}/{}/{}", load, active, sub),
        })
    })
    .collect()
}

