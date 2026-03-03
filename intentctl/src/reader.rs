use std::fs;
use voxlinuxd::repair_plan::RepairPlan;

const PLAN_DIR: &str = "/run/voxlinux/plans";

pub fn load_plans() -> Vec<RepairPlan> {
    let mut plans = Vec::new();

    if let Ok(entries) = fs::read_dir(PLAN_DIR) {
        for entry in entries.flatten() {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                match serde_json::from_str::<RepairPlan>(&content) {
                    Ok(plan) => plans.push(plan),
                    Err(e) => {
                        println!("Failed to load plan: {}", entry.path().display());
                        println!("Reason: {}", e);
                    }
                }
            }
        }
    }

    plans
}

pub fn find_plan(id: &str) -> Option<RepairPlan> {
    load_plans().into_iter().find(|p| p.id == id)
}

pub fn list_plans() {
    let plans = load_plans();

    if plans.is_empty() {
        println!("No repair plans available.");
        return;
    }

    for plan in plans {
        println!(
            "[{}] {} | Risk: {:?} | High Confidence: {}",
            plan.id,
            plan.issue,
            plan.risk,
            plan.confidence_high
        );
    }
}
