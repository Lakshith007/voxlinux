use super::classifier::{Detection, Severity};
use crate::state::BootContext;
use std::process::Command;
use crate::core::detector::detect_boot_context;
use crate::core::confidence::Confidence;



impl From<ObserverConfidence> for Confidence {
    fn from(c: ObserverConfidence) -> Self {
        match c {
            ObserverConfidence::Low => Confidence::Low,
            ObserverConfidence::Medium => Confidence::Medium,
            ObserverConfidence::High => Confidence::High,
        }
    }
}
#[derive(Debug)]
pub struct ObserverReport {
    pub boot_context: BootContext,
    pub failed_units: Vec<String>,
    pub confidence: ObserverConfidence,
}

#[derive(Debug, Clone, Copy)]
pub enum ObserverConfidence {
    Low,
    Medium,
    High,
}

pub fn emit(d: &Detection) {
    let level = match d.severity {
        Severity::Info => "INFO",
        Severity::Warn => "WARN",
        Severity::Critical => "CRITICAL",
    };

    println!(
        "[{}] {} → {}",
        level,
        d.unit,
        d.reason
    );
}

impl ObserverReport {
    pub fn collect() -> Self {
        let boot_context = detect_boot_context();
        let failed_units = collect_failed_units();
        let confidence = derive_confidence(boot_context, &failed_units);

        Self {
            boot_context,
            failed_units,
            confidence,
        }
    }
}

fn collect_failed_units() -> Vec<String> {
    let out = Command::new("systemctl")
    .args(["list-units", "--failed", "--no-legend"])
    .output();

    match out {
        Ok(o) if o.status.success() => {
            String::from_utf8_lossy(&o.stdout)
            .lines()
            .filter_map(|l| l.split_whitespace().next())
            .map(|s| s.to_string())
            .collect()
        }
        _ => Vec::new(),
    }
}

fn derive_confidence(
    boot_context: BootContext,
    failed_units: &[String],
) -> ObserverConfidence {
    match boot_context {
        BootContext::Unknown | BootContext::EarlyBoot => ObserverConfidence::Low,
        BootContext::EarlyUserspace if failed_units.len() > 5 => ObserverConfidence::Low,
        BootContext::Graphical | BootContext::MultiUser => ObserverConfidence::High,
        _ => ObserverConfidence::Medium,
    }
}
