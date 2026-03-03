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

const CORE_UNITS: &[&str] = &[
    "basic.target",
"sysinit.target",
"multi-user.target",
"graphical.target",
"systemd-journald.service",
"systemd-logind.service",
"systemd-udevd.service",
"dbus.service",
];

pub fn assess() -> Opinion {
    let output = Command::new("systemctl")
    .args(["list-units", "--failed", "--no-legend"])
    .output();

    let output = match output {
        Ok(o) if o.status.success() => o,
        _ => return Opinion::Broken {
            reason: "Unable to query systemd state".into(),
        },
    };

    let failed_units: Vec<String> = String::from_utf8_lossy(&output.stdout)
    .lines()
    .filter_map(|line| {
        let mut parts = line.split_whitespace();
        let first = parts.next()?;

        if first == "●" {
            parts.next().map(|s| s.to_string())
        } else {
            Some(first.to_string())
        }
    })
    .collect();

    if failed_units.is_empty() {
        return Opinion::Ok;
    }

    // Separate core vs non-core failures
    let core_failed: Vec<&String> = failed_units
    .iter()
    .filter(|u| CORE_UNITS.contains(&u.as_str()))
    .collect();

    if !core_failed.is_empty() {
        return Opinion::Broken {
            reason: format!(
                "Critical systemd units failed: {}",
                core_failed.len()
            ),
        };
    }

    // Non-core services failed → degraded only
    Opinion::Degraded {
        reason: format!(
            "Non-critical services failed: {}",
            failed_units.len()
        ),
    }
}
