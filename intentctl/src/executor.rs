use libc;
use std::fs::{OpenOptions, create_dir_all};
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use std::process::Command;
use voxlinuxd::repair_plan::{RepairPlan, RiskLevel};
use sha2::{Sha256, Digest};
use hex;

fn compute_hash(plan: &RepairPlan) -> String {
    let mut hasher = Sha256::new();

    hasher.update(&plan.id);
    hasher.update(&plan.issue);
    hasher.update(format!("{:?}", plan.risk));
    hasher.update(plan.confidence_high.to_string());
    hasher.update(plan.reversible.to_string());
    hasher.update(plan.requires_reboot.to_string());

    for action in &plan.actions {
        hasher.update(action);
    }

    for block in &plan.explain {
        hasher.update(block.level.to_string());
        hasher.update(format!("{:?}", block.category));
        hasher.update(&block.content);
    }

    hex::encode(hasher.finalize())
}

fn verify_integrity(plan: &RepairPlan) -> bool {
    let original_hash = plan.integrity_hash.clone();

    let mut cloned = plan.clone();
    cloned.integrity_hash = String::new();

    let recalculated = compute_hash(&cloned);

    original_hash == recalculated
}

fn ensure_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

fn log_event(message: &str) {
    let _ = create_dir_all("/tmp/voxlinux");

    let ts = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs();

    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/voxlinux/exec.log")
        {
            let _ = writeln!(file, "[{}] {}", ts, message);
        }
}


pub fn apply_plan(plan: RepairPlan, force: bool, dry_run: bool) {

    if !verify_integrity(&plan) {
        println!("❌ Plan integrity verification failed.");
        println!("The repair plan may have been tampered with.");
        return;
    }

    if !ensure_root() {
        println!("Error: intentctl repair apply must be run as root.");
        return;
    }

    if plan.risk == RiskLevel::High && !force {
        println!("High-risk plan requires --yes flag.");
        return;
    }

    if !plan.confidence_high && !force {
        println!("Plan confidence is not high. Use --yes to force.");
        return;
    }

    log_event(&format!("PLAN START {}", plan.id));

    for action in plan.actions {
        if dry_run {
            println!("Would execute: {}", action);
            log_event(&format!("DRY RUN {}", action));
            continue;
        }

        println!("Executing: {}", action);
        log_event(&format!("EXEC {}", action));

        let parts: Vec<&str> = action.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let status = Command::new(parts[0])
        .args(&parts[1..])
        .status();

        match status {
            Ok(s) if s.success() => {
                println!("✔ Success");
                log_event("RESULT success");
            }
            _ => {
                println!("✖ Failed. Aborting.");
                log_event("RESULT failure");
                return;
            }
        }
    }

    log_event(&format!("PLAN END {}", plan.id));
    println!("\nPlan executed successfully.");
}
