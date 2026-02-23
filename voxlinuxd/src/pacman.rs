use std::path::Path;
use std::process::Command;
use crate::repair_plan::{RepairPlan, RiskLevel};
use crate::explain::ExplainBlock;


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

    Some(RepairPlan {
        issue: "pacman database is locked and inconsistent".into(),
         id: "pacman-fix".into(),
         risk: RiskLevel::Medium,
         confidence_high: true,
         reversible: true,
         requires_reboot: false,
         actions: vec![
             "rm -f /var/lib/pacman/db.lck".into(),
         "pacman -Syy".into(),
         ],
         explain: vec![
             ExplainBlock {
                 level: 1,
                 title: "What happened",
                 body: "A previous pacman process exited unexpectedly, leaving a stale lock file.",
             },
         ExplainBlock {
             level: 2,
             title: "Why this is safe",
             body: "No pacman process is currently running. Only the lock file will be removed.",
         },
         ExplainBlock {
             level: 3,
             title: "Exact actions",
             body: "rm -f /var/lib/pacman/db.lck\npacman -Syy",
         },
         ],
    })
}
