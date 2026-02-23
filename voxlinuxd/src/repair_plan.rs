use crate::explain::ExplainBlock;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize)]
pub struct RepairPlan {
    pub id: String,                 // NEW
    pub issue: String,
    pub risk: RiskLevel,
    pub confidence_high: bool,

    pub reversible: bool,
    pub requires_reboot: bool,

    pub actions: Vec<String>,
    pub explain: Vec<ExplainBlock>,
}
