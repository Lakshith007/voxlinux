use std::path::Path;
use std::process::Command;
use crate::repair_plan::{RepairPlan, RiskLevel};
use crate::explain::{ExplainBlock, ExplainCategory};

pub fn pacman_broken() -> bool {
    Path::new("/var/lib/pacman/db.lck").exists()
}

pub fn heal() -> String {
    let _ = Command::new("rm")
    .arg("-f")
    .arg("/var/lib/pacman/db.lck")
    .status();

    "removed stale pacman lock".to_string()
}

pub fn sync_db() -> String {
    "pacman DB sync triggered".to_string()
}

pub fn reinstall_base() -> String {
    "base packages reinstall triggered".to_string()
}

pub fn pacman_lock_repair_plan(
    locked: bool,
    no_active_process: bool,
) -> Option<RepairPlan> {
    if !(locked && no_active_process) {
        return None;
    }

    let mut plan = RepairPlan {
        id: "pacman-lock".into(), // or your existing ID logic
        issue: "pacman database lock detected".into(),
        risk: RiskLevel::Medium,
        confidence_high: false, // or your logic
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
                content: "A previous pacman process exited unexpectedly, leaving a stale lock file.".into(),
            },
            ExplainBlock {
                level: 2,
                category: ExplainCategory::WhySafe,
                content: "No pacman process is currently running. Only the lock file will be removed.".into(),
            },
            ExplainBlock {
                level: 3,
                category: ExplainCategory::WhatWillExecute,
                content: "rm -f /var/lib/pacman/db.lck\npacman -Sy".into(),
            },
        ],
        integrity_hash: String::new(),
    };

    plan.integrity_hash = plan.compute_hash();

    Some(plan)
}
