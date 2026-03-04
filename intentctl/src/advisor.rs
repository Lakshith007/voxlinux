use serde::Serialize;
use voxlinux::repair_plan::{RepairPlan, RiskLevel};

#[derive(Debug, Serialize)]
pub struct AdvisoryItem {
    pub id: String,
    pub score: i32,
    pub risk: String,
    pub reversible: bool,
    pub requires_reboot: bool,
    pub confidence_high: bool,
}

#[derive(Debug, Serialize)]
pub struct AdvisoryReport {
    pub recommended: Option<String>,
    pub ordered: Vec<AdvisoryItem>,
    pub summary: String,
}

pub fn generate_advisory(plans: Vec<RepairPlan>) -> AdvisoryReport {
    let mut items: Vec<AdvisoryItem> = plans
    .into_iter()
    .map(|p| {
        let mut score = 0;

        // Lower risk = better score
        score += match p.risk {
            RiskLevel::Low => 30,
            RiskLevel::Medium => 15,
            RiskLevel::High => 0,
        };

        // Reversible = safer
        if p.reversible {
            score += 20;
        }

        // No reboot required = better
        if !p.requires_reboot {
            score += 15;
        }

        // High confidence = better
        if p.confidence_high {
            score += 25;
        }

        AdvisoryItem {
            id: p.id,
            score,
            risk: format!("{:?}", p.risk),
         reversible: p.reversible,
         requires_reboot: p.requires_reboot,
         confidence_high: p.confidence_high,
        }
    })
    .collect();

    // Sort descending by score
    items.sort_by(|a, b| b.score.cmp(&a.score));

    let recommended = items.first().map(|i| i.id.clone());

    let summary = if let Some(ref id) = recommended {
        format!(
            "Plan '{}' is recommended because it has the highest safety score.",
            id
        )
    } else {
        "No repair plans available.".into()
    };

    AdvisoryReport {
        recommended,
        ordered: items,
        summary,
    }
}
