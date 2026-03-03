use super::detector::RawDetection;
use crate::system_state::SystemState;
use crate::core::opinion::Opinion;
use crate::core::reporter::ObserverReport;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Warn,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureClass {
    RuntimeFailure,
    CoreIntegrityFailure,
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

/// System-wide classification

pub fn classify_system(
    report: &ObserverReport,
    health_op: &Opinion,
    systemd_op: &Opinion,
) -> FailureClass {

    // 1️⃣ Pacman integrity issues
    if report.pacman.locked && report.pacman.no_active_process {
        return FailureClass::CoreIntegrityFailure;
    }

    // 2️⃣ If systemd says Broken (core units failed)
    if matches!(systemd_op, Opinion::Broken { .. }) {
        return FailureClass::CoreIntegrityFailure;
    }

    // 3️⃣ If health subsystem says Broken (disk full, etc.)
    if matches!(health_op, Opinion::Broken { .. }) {
        return FailureClass::CoreIntegrityFailure;
    }

    // Otherwise → runtime-level failure
    FailureClass::RuntimeFailure
}
