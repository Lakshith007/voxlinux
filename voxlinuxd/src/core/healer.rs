use std::collections::HashSet;
use std::process::Command;

use crate::core::confidence::Confidence;
use crate::state::BootContext;

/// Units that must NEVER be restarted automatically
const DENYLIST: &[&str] = &[
    "basic.target",
"sysinit.target",
"multi-user.target",
"graphical.target",
"systemd-journald.service",
"systemd-logind.service",
"systemd-udevd.service",
"dbus.service",
];

#[derive(Default)]
pub struct HealingSession {
    attempted_units: HashSet<String>,
}

impl HealingSession {
    /// Attempt a safe runtime restart of a systemd unit
    pub fn restart_service(
        &mut self,
        unit: &str,
        boot_context: BootContext,
        confidence: Confidence,
    ) -> Result<(), String> {
        // ─────────────────────────────
        // HARD SAFETY GATES
        // ─────────────────────────────
        if boot_context != BootContext::Graphical {
            return Err("not in graphical session".into());
        }

        if confidence != Confidence::High {
            return Err("confidence not high".into());
        }

        if DENYLIST.contains(&unit) {
            return Err("unit is denylisted".into());
        }

        if self.attempted_units.contains(unit) {
            return Err("unit already attempted this session".into());
        }

        // ─────────────────────────────
        // Execute (one-shot)
        // ─────────────────────────────
        self.attempted_units.insert(unit.to_string());

        let status = Command::new("systemctl")
        .args(["restart", unit])
        .status()
        .map_err(|e| format!("failed to invoke systemctl: {}", e))?;

        if !status.success() {
            return Err("systemctl restart failed".into());
        }

        Ok(())
    }
}
