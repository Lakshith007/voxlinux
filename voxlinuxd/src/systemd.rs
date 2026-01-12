use std::process::Command;

const DENYLIST: &[&str] = &[
    "systemd-logind.service",
    "sddm.service",
    "getty@",
];

pub fn heal(service: &str) -> String {
    if DENYLIST.iter().any(|d| service.starts_with(d)) {
        return format!("skipped critical service '{}'", service);
    }

    let status = Command::new("systemctl")
        .arg("restart")
        .arg(service)
        .status();

    match status {
        Ok(s) if s.success() => format!("restarted service '{}'", service),
        _ => format!("failed to restart service '{}'", service),
    }
}
