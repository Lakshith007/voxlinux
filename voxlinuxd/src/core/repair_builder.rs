use crate::core::reporter::ObserverReport;
use crate::core::opinion::Opinion;
use crate::core::confidence::Confidence;
use crate::repair_plan::{RepairPlan, RiskLevel};
use crate::explain::ExplainBlock;
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
"getty@tt1.service",   // add this if you want to ignore it
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
        plans.push(RepairPlan {
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
                           title: "What happened?",
                           body: "The pacman database lock file exists but no pacman process is running.",
                       },
                       ExplainBlock {
                           level: 2,
                           title: "Why is this safe?",
                           body: "Removing a stale lock file is safe when no package transaction is active.",
                       },
                   ],
        });
    }

    // ─────────────────────────────
    // 2️⃣ FAILED SYSTEMD UNITS
    // ─────────────────────────────
    for unit in &report.failed_units {
        if DENYLIST.contains(&unit.as_str()) {
            continue;
        }
        plans.push(RepairPlan {
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
                           title: "What happened?",
                           body: "The service is listed as failed by systemd.",
                       },
                       ExplainBlock {
                           level: 2,
                           title: "What will this do?",
                           body: "Attempt to restart the failed service.",
                       },
                   ],
        });
    }

    // ─────────────────────────────
    // 3️⃣ HEALTH BROKEN
    // ─────────────────────────────
    if let Opinion::Broken { reason } = health {
        plans.push(RepairPlan {
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
                           title: "Critical state",
                           body: "The system health module detected a broken condition.",
                       },
                   ],
        });
    }

    // ─────────────────────────────
    // 4️⃣ SYSTEMD BROKEN
    // ─────────────────────────────
    if let Opinion::Broken { reason } = systemd {
        plans.push(RepairPlan {
            id: generate_plan_id("systemd-broken"),   // ← FIXED (no unit here)
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
                           title: "Systemd is broken",
                           body: "Core service manager reported critical failure.",
                       },
                   ],
        });
    }

    plans
}
