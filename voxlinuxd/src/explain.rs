use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};


pub fn note(message: String) {
    let ts = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs();

    println!("[voxlinux][{}] {}", ts, message);
}

pub fn report(target: &str, action: &str) {
    println!("================ VoxLinux Report ================");
    println!("Detected failed service : {}", target);
    println!("Action taken            : {}", action);
    println!("Result                  : completed");
    println!("================================================");
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExplainCategory {
    WhatHappened,
    WhyDetected,
    WhySafe,
    RiskAnalysis,
    WhatWillExecute,
    Preconditions,
    WhyBlocked,
}



#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExplainBlock {
    pub level: u8,
    pub category: ExplainCategory,
    pub content: String,
}

pub fn explain_at_level(blocks: &[ExplainBlock], level: u8) {
    for block in blocks.iter().filter(|b| b.level <= level) {
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
