use super::classifier::{Detection, Severity};
use crate::state::BootContext;
use crate::core::detector::detect_boot_context;
use crate::core::confidence::Confidence;
use crate::repair_plan::RepairPlan;

use std::process::Command;
use std::fs;

#[derive(Debug)]
pub struct ObserverReport {
    pub boot_context: BootContext,
    pub failed_units: Vec<String>,
    pub confidence: Confidence,
    pub pacman: PacmanState,
}

#[derive(Debug)]
pub struct PacmanState {
    pub locked: bool,
    pub no_active_process: bool,
}

pub fn emit(d: &Detection) {
    let level = match d.severity {
        Severity::Info => "INFO",
        Severity::Warn => "WARN",
        Severity::Critical => "CRITICAL",
    };

    println!("[{}] {} → {}", level, d.unit, d.reason);
}

impl ObserverReport {
    pub fn collect() -> Self {
        let boot_context = detect_boot_context();
        let failed_units = collect_failed_units();
        let confidence = derive_confidence(boot_context, &failed_units);

        let lock_exists = std::path::Path::new("/var/lib/pacman/db.lck").exists();

        let pacman_running = Command::new("pgrep")
        .arg("pacman")
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

        Self {
            boot_context,
            failed_units,
            confidence,
            pacman: PacmanState {
                locked: lock_exists,
                no_active_process: !pacman_running,
            },
        }
    }
}

fn collect_failed_units() -> Vec<String> {
    let out = Command::new("systemctl")
    .args(["list-units", "--failed", "--no-legend"])
    .output();

    match out {
        Ok(o) if o.status.success() => {
            String::from_utf8_lossy(&o.stdout)
            .lines()
            .filter_map(|line| {
                let mut parts = line.split_whitespace();

                let first = parts.next()?; // could be "●" or unit name

                if first == "●" {
                    parts.next().map(|s| s.to_string())
                } else {
                    Some(first.to_string())
                }
            })
            .collect()
        }
        _ => Vec::new(),
    }
}

pub fn emit_repair_plans(plans: &[RepairPlan]) {
    let dir = "/run/voxlinux/plans";
    let _ = std::fs::create_dir_all(dir);

    for plan in plans {
        let path = format!("{}/{}.json", dir, plan.id);

        match serde_json::to_string_pretty(plan) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&path, json) {
                    eprintln!("[ERROR] failed to write plan {}: {}", plan.id, e);
                } else {
                    println!("[PLAN] {} → {}", plan.id, path);
                }
            }
            Err(e) => {
                eprintln!("[ERROR] failed to serialize plan {}: {}", plan.id, e);
            }
        }
    }
}

pub fn print_plan_summary(plan: &RepairPlan) {
    println!("\n⚠ VoxLinux detected an issue");
    println!("ID: {}", plan.id);
    println!("Issue: {}", plan.issue);
    println!("Risk: {:?}", plan.risk);
    println!("Confidence High: {}", plan.confidence_high);
    println!("Reversible: {}", plan.reversible);
    println!("Requires Reboot: {}", plan.requires_reboot);
    println!("Actions:");
    for a in &plan.actions {
        println!("  • {}", a);
    }
}

fn derive_confidence(
    boot_context: BootContext,
    failed_units: &[String],
) -> Confidence {
    match boot_context {
        BootContext::Unknown | BootContext::EarlyBoot => Confidence::Low,
        BootContext::EarlyUserspace if failed_units.len() > 5 => Confidence::Low,
        BootContext::Graphical | BootContext::MultiUser => Confidence::High,
        _ => Confidence::Medium,
    }
}
