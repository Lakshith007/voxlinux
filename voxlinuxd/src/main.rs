mod core;
mod systemd;
mod health;
mod state;
mod predictive;

mod pacman;
mod system_state;
mod probe;
mod healing_level;
mod verifier;

mod ipc;
mod repair_executor;


use std::process::Command;
use std::thread;
use std::time::Duration;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use voxlinux::explain::{ExplainBlock, ExplainCategory};
use core::{detector, classifier, policy, reporter};
use core::heal_gate::healing_allowed;
use core::confidence_eval::evaluate;
use core::opinion::Opinion;
use core::classifier::{Severity, FailureClass};
use voxlinux::repair_plan::{RepairPlan, RiskLevel};
use core::ai_advisor;
use crate::core::repair_builder::build_repair_plans;
use crate::core::reporter::ObserverReport;
use crate::core::deferred::DeferredHealQueue;
use crate::core::healer::HealingSession;
use crate::healing_level::HealingLevel;

fn init_runtime_dirs() {
    let base = "/run/voxlinux";
    let plans = "/run/voxlinux/plans";

    fs::create_dir_all(plans).expect("Failed to create runtime directory");

    fs::set_permissions(base, fs::Permissions::from_mode(0o700))
    .expect("Failed to set permissions on /run/voxlinux");
}


fn main() {


    let mut healing_level = HealingLevel::AssistedRepair;
    let mut last_notified_issue: Option<String> = None;
    let mut deferred_queue = DeferredHealQueue::default();
    let mut healing_session = HealingSession::default();


    init_runtime_dirs();   // FIRST create /run/voxlinux

    let _ = std::fs::remove_dir_all("/run/voxlinux/plans");
    let _ = std::fs::create_dir_all("/run/voxlinux/plans");

    std::thread::spawn(|| {
        ipc::start_ipc_server();
    });

    println!("voxlinuxd: self-healing engine started");

    loop {
        // ─────────────────────────────
        // 0️⃣ Observer snapshot
        // ─────────────────────────────
        let report = ObserverReport::collect();

        // ─────────────────────────────
        // 1️⃣ Detection & unit classification
        // ─────────────────────────────
        let detections = detector::scan();

        for d in detections {
            let classified = classifier::classify(d);

            if classified.severity == Severity::Critical {
                deferred_queue.enqueue(&classified);
            }

            let filtered = policy::apply_policy(classified);
            reporter::emit(&filtered);
        }

        // ─────────────────────────────
        // 2️⃣ Advisory opinions
        // ─────────────────────────────
        let health_op = health::assess();
        let systemd_op = systemd::assess();

        // ─────────────────────────────
        // 3️⃣ Confidence evaluation
        // ─────────────────────────────
        let confidence = evaluate(
            &health_op,
            &systemd_op,
            report.confidence,
        );

        // ─────────────────────────────
        // 3️⃣.1 System-wide failure classification
        // ─────────────────────────────
        let failure_class =
        classifier::classify_system(&report, &health_op, &systemd_op);

        println!("[FAILURE_CLASS] {:?}", failure_class);

        if failure_class == FailureClass::CoreIntegrityFailure {
            println!("[POLICY] Integrity failure detected → autonomy restricted.");
        }

        println!(
            "[CONFIDENCE] health={:?}, systemd={:?} → {:?}",
            health_op, systemd_op, confidence
        );

        // ─────────────────────────────
        // 4️⃣ Human-readable status
        // ─────────────────────────────
        match &health_op {
            Opinion::Ok => {}
            Opinion::Degraded { reason } => {
                println!("[WARN] health → {}", reason);
            }
            Opinion::Broken { reason } => {
                println!("[CRITICAL] health → {}", reason);
            }
        }

        match &systemd_op {
            Opinion::Ok => {}
            Opinion::Degraded { reason } => {
                println!("[WARN] systemd → {}", reason);
            }
            Opinion::Broken { reason } => {
                println!("[CRITICAL] systemd → {}", reason);
            }
        }

        // ─────────────────────────────
        // 5️⃣ Healing gate
        // ─────────────────────────────
        let allowed = healing_allowed(
            &health_op,
            &systemd_op,
            confidence,
            report.boot_context,
        );

        if !allowed {
            println!(
                "[SAFE] Healing blocked (boot={:?}, confidence={:?})",
                     report.boot_context,
                     confidence
            );
        }

        // ─────────────────────────────
        // Stage-2 Assisted Repair
        // ─────────────────────────────
        if healing_level == HealingLevel::AssistedRepair {
            let plans = build_repair_plans(&report, &health_op, &systemd_op);

            if plans.is_empty() {
                println!("[STAGE2] No repair plans generated.");
            } else {
                reporter::emit_repair_plans(&plans);

                for plan in &plans {
                    reporter::print_plan_summary(plan);
                }
            }
        }

        // ─────────────────────────────
        // Stage-1 RuntimeSafe healing
        // ─────────────────────────────
        if healing_level == HealingLevel::RuntimeSafe && allowed {
            deferred_queue.try_execute(
                report.boot_context,
                confidence,
                |action| {
                    match healing_session.restart_service(
                        &action.unit,
                        report.boot_context,
                        confidence,
                    ) {
                        Ok(_) => {
                            println!(
                                "[HEAL] action=restart unit={} decision=executed reason=runtime-safe+high-confidence",
                                action.unit
                            );
                        }
                        Err(reason) => {
                            println!(
                                "[HEAL] action=restart unit={} decision=skipped reason={}",
                                action.unit,
                                reason
                            );
                        }
                    }
                },
            );
        }

        // ─────────────────────────────
        // Stage-3 Autonomous Repair
        // ─────────────────────────────
        if healing_level == HealingLevel::AutonomousRepair
            && allowed
            && failure_class != FailureClass::CoreIntegrityFailure
            {
                let plans = build_repair_plans(&report, &health_op, &systemd_op);

                for plan in plans {
                    if plan.risk != voxlinux::repair_plan::RiskLevel::Low {
                        continue;
                    }

                    let key = plan.issue.clone();

                    if !state::should_retry(&key) {
                        println!("[AUTO] Backoff active for {}", key);
                        continue;
                    }

                    let level = state::current_level(&key);

                    println!(
                        "[AUTO] Executing plan={} (Level {})",
                             plan.id, level
                    );

                    let mut success = true;

                    match level {
                        1 => {
                            for action in &plan.actions {
                                println!("[AUTO] Running: {}", action);
                                let status_ok = std::process::Command::new("sh")
                                .arg("-c")
                                .arg(&action)
                                .status()
                                .map(|s| s.success())
                                .unwrap_or(false);

                                if !status_ok {
                                    success = false;
                                } else {
                                    // VERIFY service actually recovered
                                    if let Some(unit) = action.split_whitespace().last() {
                                        let is_active = std::process::Command::new("systemctl")
                                        .arg("is-active")
                                        .arg(unit)
                                        .status()
                                        .map(|s| s.success())
                                        .unwrap_or(false);

                                        if !is_active {
                                            success = false;
                                        }
                                    }
                                }
                            }
                        }

                        2 => {
                            println!("[AUTO] Level 2 → reload + restart");

                            let _ = std::process::Command::new("systemctl")
                            .arg("daemon-reload")
                            .status();

                            for action in &plan.actions {
                                println!("[AUTO] Running: {}", action);
                                let status_ok = std::process::Command::new("sh")
                                .arg("-c")
                                .arg(&action)
                                .status()
                                .map(|s| s.success())
                                .unwrap_or(false);

                                if !status_ok {
                                    success = false;
                                } else {
                                    // Wait briefly for service to stabilize
                                    std::thread::sleep(std::time::Duration::from_secs(2));

                                    if let Some(unit) = action.split_whitespace().last() {
                                        let is_active = std::process::Command::new("systemctl")
                                        .arg("is-active")
                                        .arg(unit)
                                        .output()
                                        .map(|o| {
                                            let out = String::from_utf8_lossy(&o.stdout);
                                            out.trim() == "active"
                                        })
                                        .unwrap_or(false);

                                        if !is_active {
                                            success = false;
                                        }
                                    }
                                }
                            }
                        }

                        3 => {
                            println!(
                                "[AUTO] Level 3 reached for {} → switching to AssistedRepair",
                                key
                            );
                            healing_level = HealingLevel::AssistedRepair;
                            continue;
                        }

                        _ => {
                            println!(
                                "[AUTO] Level 4 → quarantining service {}",
                                key
                            );

                            let _ = std::process::Command::new("systemctl")
                            .arg("disable")
                            .arg(&plan.actions[0]
                            .split_whitespace()
                            .last()
                            .unwrap_or(""))
                            .status();
                            continue;
                        }
                    }

                    if success {
                        println!("[AUTO] Plan={} succeeded", plan.id);
                        state::reset_level(&key);
                        state::bump_confidence(&key);
                    } else {
                        println!("[AUTO] Plan={} failed → escalating", plan.id);
                        state::escalate_level(&key);
                        state::drop_confidence(&key);
                    }
                }
            }

            thread::sleep(Duration::from_secs(60));
    }
}
