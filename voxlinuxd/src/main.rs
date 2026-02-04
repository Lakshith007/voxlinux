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
use core::heal_gate::healing_allowed;
use core::{detector, classifier, policy, reporter};
use core::opinion::Opinion;
use core::confidence_eval::evaluate;
use core::confidence::Confidence;
use crate::core::reporter::ObserverReport;

fn main() {
    println!("voxlinuxd: observe-only mode started");

    loop {
        // ===============================
        // 1️⃣ Core detection pipeline
        // ===============================
        let detections = detector::scan();
        let report = ObserverReport::collect();

        for d in detections {
            let classified = classifier::classify(d);
            let filtered = policy::apply_policy(classified);
            reporter::emit(&filtered);
        }

        // ===============================
        // 2️⃣ Advisory opinions
        // ===============================
        let health_op = health::assess();
        let systemd_op = systemd::assess();

        // ===============================
        // 3️⃣ Confidence evaluation
        // ===============================
        let report = ObserverReport::collect();

        let confidence = evaluate(
            &health_op,
            &systemd_op,
            report.confidence,
        );


        println!(
            "[CONFIDENCE] health={:?}, systemd={:?} → {:?}",
            health_op, systemd_op, confidence
        );

        // ===============================
        // 4️⃣ Human-readable output
        // ===============================
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

        // ===============================
        // 5️⃣ Healing gate (observe-only)
        // ===============================
        if confidence == Confidence::High {
            println!("[HEAL-GATE] Eligible for healing (BLOCKED for now)");
        } else {
            println!("[SAFE] Healing blocked (confidence: {:?})", confidence);
        }

        let report = ObserverReport::collect();

        let confidence = evaluate(
            &health_op,
            &systemd_op,
            report.confidence,
        );

        if healing_allowed(
            &health_op,
            &systemd_op,
            confidence,
            report.boot_context,
        ) {
            println!(
                "[HEAL-CANDIDATE] Eligible for healing (confidence: {:?})",
                     confidence
            );
        } else {
            println!(
                "[SAFE] Healing blocked (boot={:?}, confidence={:?})",
                     report.boot_context,
                     confidence
            );
        }


        thread::sleep(Duration::from_secs(60));


    }
}
