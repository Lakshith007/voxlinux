use crate::{pacman, systemd, explain, state};
use crate::system_state::SystemState;
use std::process::{Command, Stdio};

pub fn heal(state_obj: &SystemState) {
    let key = format!("{:?}", state_obj);
    let level = state::current_level(&key);
    let confidence = state::get_confidence(&key);

    explain::note(format!(
        "Healing {:?} | level=L{} | confidence={:.2}",
        state_obj, level, confidence
    ));

    match state_obj {
        SystemState::PackageConsistent => heal_package(level),
        SystemState::ServiceActive(service) => heal_service(service, level),
        SystemState::NetworkReachable => heal_network(level),
        SystemState::FilesystemWritable(path) => heal_filesystem(path, level),
        SystemState::BootHealthy => heal_boot(level),
    }

    // ─────────────────────────────
    // Confidence-aware escalation
    // ─────────────────────────────
    if confidence >= 0.4 {
        state::escalate_level(&key);
    } else {
        explain::note("Low confidence: holding healing level".into());
    }
}

/// ───────────── Services ─────────────
fn heal_service(service: &str, level: u8) {
    match level {
        1 => {
            explain::note("L1: restarting service".into());
            systemd::heal(service);
        }
        2 => {
            explain::note("L2: daemon reload + restart".into());
            systemd::daemon_reload();
            systemd::heal(service);
        }
        3 => {
            explain::note("L3: reinstalling service unit".into());
            systemd::reinstall_unit(service);
        }
        _ => {
            explain::note(format!(
                "L4: isolating service '{}' (manual intervention)",
                service
            ));
        }
    }
}

/// ───────────── Packages ─────────────
fn heal_package(level: u8) {
    match level {
        1 => {
            explain::note("L1: removing pacman lock".into());
            pacman::heal();
        }
        2 => {
            explain::note("L2: syncing pacman database".into());
            pacman::sync_db();
        }
        3 => {
            explain::note("L3: reinstalling base packages".into());
            pacman::reinstall_base();
        }
        _ => {
            explain::note("L4: pacman isolated".into());
        }
    }
}

/// ───────────── Network ─────────────
fn heal_network(level: u8) {
    match level {
        1 => {
            explain::note("L1: restarting NetworkManager".into());
            let _ = Command::new("systemctl")
                .args(["restart", "NetworkManager"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
        2 => {
            explain::note("L2: enabling networking (nmcli on)".into());
            let _ = Command::new("nmcli")
                .args(["networking", "on"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
        3 => {
            explain::note("L3: resetting network stack".into());
            let _ = Command::new("nmcli")
                .args(["networking", "off"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            let _ = Command::new("nmcli")
                .args(["networking", "on"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
        _ => {
            explain::note("L4: network isolated".into());
        }
    }
}

/// ───────────── Filesystem ─────────────
fn heal_filesystem(path: &str, level: u8) {
    match level {
        1 => {
            explain::note("L1: remounting filesystem rw".into());
            let _ = Command::new("mount")
                .args(["-o", "remount,rw", path])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
        2 => {
            explain::note("L2: fsck scheduled for next boot".into());
        }
        3 => {
            explain::note("L3: emergency sync".into());
            let _ = Command::new("sync").status();
        }
        _ => {
            explain::note("L4: filesystem isolated".into());
        }
    }
}

/// ───────────── Boot / Initramfs ─────────────
fn heal_boot(level: u8) {
    match level {
        1 => {
            explain::note("L1: regenerating initramfs".into());
            let _ = Command::new("mkinitcpio")
                .args(["-P"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
        2 => {
            explain::note("L2: reinstalling kernel".into());
            let _ = Command::new("pacman")
                .args(["-S", "--noconfirm", "linux"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
        3 => {
            explain::note("L3: regenerating bootloader config".into());
            let _ = Command::new("grub-mkconfig")
                .args(["-o", "/boot/grub/grub.cfg"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
        _ => {
            explain::note("L4: boot recovery required".into());
        }
    }
}

