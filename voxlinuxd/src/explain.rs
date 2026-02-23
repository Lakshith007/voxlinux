use std::time::{SystemTime, UNIX_EPOCH};
use serde::Serialize;

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

#[derive(Debug, Clone, Serialize)]
pub struct ExplainBlock {
    pub level: u8,
    pub title: &'static str,
    pub body: &'static str,
}

pub fn explain_at_level(blocks: &[ExplainBlock], level: u8) {
    for block in blocks.iter().filter(|b| b.level == level) {
        println!("\n{}:\n{}", block.title, block.body);
    }
}
