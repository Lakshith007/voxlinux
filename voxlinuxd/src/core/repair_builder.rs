use crate::core::reporter::ObserverReport;
use crate::core::opinion::Opinion;
use crate::core::confidence::Confidence;
use voxlinux::repair_plan::{RepairPlan, RiskLevel};
use voxlinux::explain::{ExplainBlock, ExplainCategory};

use std::time::{SystemTime, UNIX_EPOCH};

const DENYLIST: &[&str] = &[
    "basic.target",
"sysinit.target",
"multi-user.target",
"graphical.target",
"systemd-journald.service",
"systemd-logind.service",
"systemd-udevd.service",
"dbus.service",
"getty@tt1.service",
];

fn generate_plan_id(prefix: &str) -> String {
    let ts = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs();

    format!("{}-{}", prefix, ts)
}

pub fn build_repair_plans(
    report: &ObserverReport,
    health: &Opinion,
    systemd: &Opinion,
) -> Vec<RepairPlan> {
    let mut plans = Vec::new();

    // ─────────────────────────────
    // 1️⃣ PACMAN LOCK
    // ─────────────────────────────
    if report.pacman.locked && report.pacman.no_active_process {
        let mut plan = RepairPlan {
            id: generate_plan_id("pacman-lock"),
            issue: "pacman database is locked".into(),
            risk: RiskLevel::Medium,
            confidence_high: report.confidence == Confidence::High,
            reversible: true,
            requires_reboot: false,
            actions: vec![
                "rm -f /var/lib/pacman/db.lck".into(),
                "pacman -Sy".into(),
            ],
            explain: vec![
                ExplainBlock {
                    level: 1,
                    category: ExplainCategory::WhatHappened,
                    content: "A pacman database lock file exists, preventing package operations.".into(),
                },
                ExplainBlock {
                    level: 2,
                    category: ExplainCategory::WhyDetected,
                    content: "VoxLinux verified that the lock file exists and no pacman process is currently running.".into(),
                },
                ExplainBlock {
                    level: 3,
                    category: ExplainCategory::WhySafe,
                    content: "Since no package transaction is active, removing the stale lock file is considered safe.".into(),
                },
                ExplainBlock {
                    level: 4,
                    category: ExplainCategory::WhatWillExecute,
                    content: "The system will remove the lock file and re-synchronize package databases.".into(),
                },
            ],
            integrity_hash: String::new(),
        };

        plan.integrity_hash = plan.compute_hash();
        plans.push(plan);
    }

    // ─────────────────────────────
    // 2️⃣ FAILED SYSTEMD UNITS
    // ─────────────────────────────
    for unit in &report.failed_units {
        if DENYLIST.contains(&unit.as_str()) {
            continue;
        }

        let mut plan = RepairPlan {
            id: generate_plan_id(&format!("restart-{}", unit)),
            issue: format!("systemd unit '{}' failed", unit),
            risk: RiskLevel::Low,
            confidence_high: report.confidence == Confidence::High,
            reversible: true,
            requires_reboot: false,
            actions: vec![
                format!("systemctl restart {}", unit),
            ],
            explain: vec![
                ExplainBlock {
                    level: 1,
                    category: ExplainCategory::WhatHappened,
                    content: "Systemd reports that this service is currently in a failed state.".into(),
                },
                ExplainBlock {
                    level: 2,
                    category: ExplainCategory::WhyDetected,
                    content: "The service appears in the output of 'systemctl list-units --failed'.".into(),
                },
                ExplainBlock {
                    level: 3,
                    category: ExplainCategory::WhySafe,
                    content: "Restarting a failed service is generally safe when confidence is high and the system is stable.".into(),
                },
                ExplainBlock {
                    level: 4,
                    category: ExplainCategory::WhatWillExecute,
                    content: "VoxLinux will attempt to restart the service using systemctl restart.".into(),
                },
            ],
            integrity_hash: String::new(),
        };

        plan.integrity_hash = plan.compute_hash();
        plans.push(plan);
    }

    // ─────────────────────────────
    // 3️⃣ HEALTH BROKEN
    // ─────────────────────────────
    if let Opinion::Broken { reason } = health {
        let mut plan = RepairPlan {
            id: generate_plan_id("health-broken"),
            issue: format!("system health broken: {}", reason),
            risk: RiskLevel::High,
            confidence_high: false,
            reversible: false,
            requires_reboot: true,
            actions: vec![
                "Investigate system logs".into(),
                "Consider rebooting".into(),
            ],
            explain: vec![
                ExplainBlock {
                    level: 1,
                    category: ExplainCategory::WhatHappened,
                    content: "The health module detected a critical system condition.".into(),
                },
                ExplainBlock {
                    level: 2,
                    category: ExplainCategory::RiskAnalysis,
                    content: "This condition may affect system stability or integrity.".into(),
                },
                ExplainBlock {
                    level: 3,
                    category: ExplainCategory::Preconditions,
                    content: "Further manual inspection is required before automated repair is attempted.".into(),
                },
            ],
            integrity_hash: String::new(),
        };

        plan.integrity_hash = plan.compute_hash();
        plans.push(plan);
    }

    // ─────────────────────────────
    // 4️⃣ SYSTEMD BROKEN
    // ─────────────────────────────
    if let Opinion::Broken { reason } = systemd {
        let mut plan = RepairPlan {
            id: generate_plan_id("systemd-broken"),
            issue: format!("systemd integrity issue: {}", reason),
            risk: RiskLevel::High,
            confidence_high: false,
            reversible: false,
            requires_reboot: true,
            actions: vec![
                "Check journalctl -xe".into(),
                "Reboot system".into(),
            ],
            explain: vec![
                ExplainBlock {
                    level: 1,
                    category: ExplainCategory::WhatHappened,
                    content: "The systemd subsystem reported a critical integrity issue.".into(),
                },
                ExplainBlock {
                    level: 2,
                    category: ExplainCategory::RiskAnalysis,
                    content: "Core service management instability may impact the entire operating system.".into(),
                },
                ExplainBlock {
                    level: 3,
                    category: ExplainCategory::Preconditions,
                    content: "Manual inspection of logs is recommended before attempting corrective action.".into(),
                },
            ],
            integrity_hash: String::new(),
        };

        plan.integrity_hash = plan.compute_hash();
        plans.push(plan);
    }

    plans
}
