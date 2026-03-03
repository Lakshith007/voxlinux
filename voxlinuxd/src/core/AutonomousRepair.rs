use crate::repair_plan::{RepairPlan, RiskLevel};
use crate::core::confidence::Confidence;

pub fn attempt_autonomous_execution(
    plans: &[RepairPlan],
    confidence: Confidence,
    boot_safe: bool,
) {
    if confidence != Confidence::High || !boot_safe {
        println!("[AUTO] Autonomous healing blocked by policy.");
        return;
    }

    for plan in plans {
        if should_execute(plan) {
            println!("[AUTO] Executing plan: {}", plan.id);
            execute(plan);
        }
    }
}

fn should_execute(plan: &RepairPlan) -> bool {
    plan.confidence_high
    && plan.reversible
    && plan.risk != RiskLevel::High
}

fn execute(plan: &RepairPlan) {
    for action in &plan.actions {
        println!("[AUTO] Running: {}", action);

        let _ = std::process::Command::new("sh")
        .arg("-c")
        .arg(action)
        .status();
    }
}
