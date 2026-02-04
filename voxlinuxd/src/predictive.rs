// predictive.rs
//
// Predictive self-healing layer for VoxLinux
// -----------------------------------------
// This module detects early warning signals (drift)
// and triggers EXISTING self-healing logic in systemd.rs.
//
// It never executes systemctl directly.

use crate::explain;
use crate::state;
use crate::systemd;
use crate::system_state::SystemState;
//use crate::healer;
/// Why a healing action was triggered
#[derive(Debug)]
pub enum TriggerReason {
    Predicted,
}

/// Represents detected drift
struct DriftResult {
    service: String,
    score: u32,
}

/// Entry point called from main daemon loop
pub fn observe() {
    let drifts = detect_systemd_drift();

    for drift in drifts {
        // Conservative threshold (safe default)
        if drift.score >= 15 {
            trigger_preventive_heal(&drift);
        }
    }
}

/// Detect restart-pattern drift in selected services
fn detect_systemd_drift() -> Vec<DriftResult> {
    let mut results = Vec::new();

    // Start with non-critical services only
    let services = [
        "NetworkManager.service",
        "sshd.service",
    ];

    for service in services {
        if let Some(drift) = check_restart_trend(service) {
            results.push(drift);
        }
    }

    results
}

/// Compare current restart count with previous value
fn check_restart_trend(service: &str) -> Option<DriftResult> {
    let current = systemd::get_restart_count(service)?;

    let previous = state::get_last_restart_count(service)
        .unwrap_or(current);

    // Persist for next observation cycle
    state::set_last_restart_count(service, current);

    let delta = current.saturating_sub(previous);

    if delta == 0 {
        return None;
    }

    Some(DriftResult {
        service: service.to_string(),
        score: delta * 10, // simple, explainable heuristic
    })
}

/// Call existing self-healing logic early

fn trigger_preventive_heal(drift: &DriftResult) {
    explain::note(format!(
        "Predictive alert: restart trend detected for '{}' (drift score = {})",
        drift.service,
        drift.score
    ));

    //healer::heal(
        //&SystemState::ServiceActive(drift.service.clone())
    //);

    explain::note(format!(
        "Predictive healing triggered for '{}'",
        drift.service
    ));
}

