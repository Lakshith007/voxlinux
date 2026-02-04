use super::classifier::{Detection, Severity};

const IGNORE_PATTERNS: &[&str] = &[
    "getty@",
    "serial-getty@",
    "bluetooth.service",
    "cups.service",
    "modemmanager.service",
    "avahi-daemon.service",
    "power-profiles-daemon.service",
];

const ALLOW_LIST: &[&str] = &[
    "NetworkManager.service",
    "pipewire.service",
    "wireplumber.service",
    "sddm.service",
    "systemd-logind.service",
    "dbus.service",
];

fn matches_pattern(unit: &str, pattern: &str) -> bool {
    unit == pattern || unit.starts_with(pattern)
}

pub fn apply_policy(mut d: Detection) -> Detection {
    // 1. Ignore list → force INFO
    for pattern in IGNORE_PATTERNS {
        if matches_pattern(&d.unit, pattern) {
            d.severity = Severity::Info;
            d.reason.push_str(" (ignored by policy)");
            return d;
        }
    }

    // 2. Allow list → allow severity
    for allowed in ALLOW_LIST {
        if d.unit == *allowed {
            return d;
        }
    }

    // 3. Default → INFO only
    d.severity = Severity::Info;
    d.reason.push_str(" (not in allow list)");
    d
}
