use std::fs::{OpenOptions, create_dir_all};
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use std::process::Command;
use crate::repair_plan::{RepairPlan, RiskLevel};
use sha2::{Sha256, Digest};
use hex;
use std::fs;

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

pub fn apply_plan(plan: RepairPlan) {

    println!("[EXECUTOR] Applying plan {}", plan.id);

    if !verify_integrity(&plan) {
        println!("[EXECUTOR] ❌ Integrity verification failed.");
        log_event("Integrity failure");
        return;
    }

    if plan.risk == RiskLevel::High {
        println!("[EXECUTOR] High-risk plan blocked by policy.");
        log_event("Blocked high-risk plan");
        return;
    }

    if !plan.confidence_high {
        println!("[EXECUTOR] Low-confidence plan blocked.");
        log_event("Blocked low-confidence plan");
        return;
    }

    log_event(&format!("PLAN START {}", plan.id));

    for action in plan.actions {

        println!("[EXECUTOR] Running: {}", action);
        log_event(&format!("EXEC {}", action));

        let status = Command::new("sh")
        .arg("-c")
        .arg(&action)
        .status();

        match status {
            Ok(s) if s.success() => {
                println!("[EXECUTOR] ✔ Success");
                log_event("RESULT success");
            }
            _ => {
                println!("[EXECUTOR] ✖ Failed. Aborting.");
                log_event("RESULT failure");
                return;
            }
        }
    }

    log_event(&format!("PLAN END {}", plan.id));
    println!("[EXECUTOR] Plan executed successfully.");

    let plan_path = format!("/run/voxlinux/plans/{}.json", plan.id);
    let _ = fs::remove_file(&plan_path);
    println!("[EXECUTOR] Plan file removed.");



    match std::fs::remove_file(&plan_path) {
        Ok(_) => println!("[EXECUTOR] Removed plan file {}", plan.id),
        Err(e) => println!("[EXECUTOR] Failed to remove plan file: {}", e),
    }
}
