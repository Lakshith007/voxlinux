use crate::explain::ExplainBlock;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RepairPlan {
    pub id: String,                 // NEW
    pub issue: String,
    pub risk: RiskLevel,
    pub confidence_high: bool,

    pub reversible: bool,
    pub requires_reboot: bool,

    pub actions: Vec<String>,
    pub explain: Vec<ExplainBlock>,

    pub integrity_hash: String,
}

impl RepairPlan {
    pub fn compute_hash(&self) -> String {
        let mut hasher = Sha256::new();

        hasher.update(&self.id);
        hasher.update(&self.issue);
        hasher.update(format!("{:?}", self.risk));
        hasher.update(self.confidence_high.to_string());
        hasher.update(self.reversible.to_string());
        hasher.update(self.requires_reboot.to_string());

        for action in &self.actions {
            hasher.update(action);
        }

        for block in &self.explain {
            hasher.update(block.level.to_string());
            hasher.update(format!("{:?}", block.category));
            hasher.update(&block.content);
        }

        let result = hasher.finalize();
        hex::encode(result)
    }
}
