use std::process::Command;
use crate::core::opinion::Opinion;

pub fn get_restart_count(_service: &str) -> Option<u32> {
    None
}

pub fn check() -> Option<String> {
    None
}

pub fn is_active(_service: &str) -> bool {
    true
}


pub fn assess() -> Opinion {
    let output = Command::new("systemctl")
    .arg("--failed")
    .arg("--no-legend")
    .output();

    let output = match output {
        Ok(o) => o,
        Err(_) => {
            return Opinion::Broken {
                reason: "Unable to query systemd state".to_string(),
            };
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);

    // No failed units at all
    if stdout.trim().is_empty() {
        return Opinion::Ok;
    }

    // Analyze failures
    let mut critical = Vec::new();
    let mut ignorable = Vec::new();

    for line in stdout.lines() {
        if line.contains("getty@")
            || line.contains("serial-getty@")
            || line.contains("bluetooth.service")
            || line.contains("cups.service")
            {
                ignorable.push(line);
            } else {
                critical.push(line);
            }
    }

    if !critical.is_empty() {
        return Opinion::Broken {
            reason: format!(
                "Critical systemd units failed: {}",
                critical.len()
            ),
        };
    }

    Opinion::Degraded {
        reason: format!(
            "Non-critical systemd units failed: {}",
            ignorable.len()
        ),
    }
}
