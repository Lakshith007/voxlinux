use super::detector::RawDetection;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Warn,
    Critical,
}

#[derive(Debug)]
pub struct Detection {
    pub unit: String,
    pub severity: Severity,
    pub reason: String,
}

pub fn classify(raw: RawDetection) -> Detection {
    let severity = if raw.unit.contains("systemd-logind")
        || raw.unit.contains("dbus")
    {
        Severity::Critical
    } else {
        Severity::Info
    };

    Detection {
        unit: raw.unit,
        severity,
        reason: format!("systemd reported status: {}", raw.status),
    }
}
