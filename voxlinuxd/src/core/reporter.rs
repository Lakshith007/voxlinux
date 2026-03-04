use super::classifier::{Detection, Severity};
use crate::state::BootContext;
use crate::core::detector::detect_boot_context;
use crate::core::confidence::Confidence;
use voxlinux::repair_plan::RepairPlan;
use voxlinux::explain::{ExplainBlock, ExplainCategory};

use std::process::Command;
use std::fs;
use std::io::Write;

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

                let first = parts.next()?;

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

    if let Err(e) = fs::create_dir_all(dir) {
        eprintln!("[ERROR] failed to create plan directory: {}", e);
        return;
    }

    for plan in plans {

        let tmp_path = format!("{}/{}.tmp", dir, plan.id);
        let final_path = format!("{}/{}.json", dir, plan.id);

        match serde_json::to_string_pretty(plan) {
            Ok(json) => {

                match fs::File::create(&tmp_path) {
                    Ok(mut file) => {

                        if let Err(e) = file.write_all(json.as_bytes()) {
                            eprintln!("[ERROR] failed writing tmp plan {}: {}", plan.id, e);
                            continue;
                        }

                        if let Err(e) = file.sync_all() {
                            eprintln!("[ERROR] sync failed for {}: {}", plan.id, e);
                            continue;
                        }

                        if let Err(e) = fs::rename(&tmp_path, &final_path) {
                            eprintln!("[ERROR] rename failed for {}: {}", plan.id, e);
                            continue;
                        }

                        println!("[PLAN] saved → {}", final_path);
                    }

                    Err(e) => {
                        eprintln!("[ERROR] failed creating tmp plan {}: {}", plan.id, e);
                    }
                }
            }

            Err(e) => {
                eprintln!("[ERROR] failed to serialize plan {}: {}", plan.id, e);
            }
        }
    }
}

pub fn print_explanation(plan: &RepairPlan, level: u8) {

    println!("\nExplanation (Level {}):", level);

    for block in plan.explain.iter().filter(|b| b.level <= level) {

        let label = match block.category {
            ExplainCategory::WhatHappened => "What happened",
            ExplainCategory::WhyDetected => "Why detected",
            ExplainCategory::WhySafe => "Why safe",
            ExplainCategory::RiskAnalysis => "Risk analysis",
            ExplainCategory::WhatWillExecute => "What will execute",
            ExplainCategory::Preconditions => "Preconditions",
            ExplainCategory::WhyBlocked => "Why blocked",
        };

        println!("\n{}:\n{}", label, block.content);
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

        BootContext::EarlyUserspace if failed_units.len() > 5 => {
            Confidence::Low
        }

        BootContext::Graphical | BootContext::MultiUser => {
            Confidence::High
        }

        _ => Confidence::Medium,
    }
}
