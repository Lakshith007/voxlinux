mod core;
mod systemd;
mod health;
mod state;
mod predictive;
mod explain;
mod pacman;
mod system_state;
mod probe;
mod healing_level;
mod verifier;

use std::thread;
use std::time::Duration;

use core::{detector, classifier, policy, reporter};
use core::heal_gate::healing_allowed;
use core::confidence_eval::evaluate;
use core::confidence::Confidence;
use core::opinion::Opinion;
use core::classifier::Severity;

use crate::core::reporter::ObserverReport;
use crate::core::deferred::DeferredHealQueue;
use crate::core::healer::HealingSession;

fn main() {
    println!("voxlinuxd: observe-only mode started");

    // ─────────────────────────────
    // Memory-only session state
    // ─────────────────────────────
    let mut deferred_queue = DeferredHealQueue::default();
    let mut healing_session = HealingSession::default();

    loop {
        // ─────────────────────────────
        // 0️⃣ Observer snapshot (ONCE)
        // ─────────────────────────────
        let report = ObserverReport::collect();

        // ─────────────────────────────
        // 1️⃣ Detection & classification
        // ─────────────────────────────
        let detections = detector::scan();

        for d in detections {
            let classified = classifier::classify(d);

            // Record intent only (no action yet)
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
        // 5️⃣ Healing gate (permission)
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
        // 6️⃣ Stage-1 deferred healing
        // ─────────────────────────────
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

        thread::sleep(Duration::from_secs(60));
    }
}
