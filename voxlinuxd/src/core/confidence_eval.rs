use crate::core::confidence::Confidence;
use crate::core::opinion::Opinion;

pub fn evaluate(
    health: &Opinion,
    systemd: &Opinion,
    observer_confidence: Confidence,
) -> Confidence {
    let engine_confidence = evaluate_engine_confidence(health, systemd);

    engine_confidence.min(observer_confidence)
}

fn evaluate_engine_confidence(
    health: &Opinion,
    systemd: &Opinion,
) -> Confidence {
    match (health, systemd) {
        (Opinion::Ok, Opinion::Ok) => Confidence::High,
        (Opinion::Broken { .. }, _) |
        (_, Opinion::Broken { .. }) => Confidence::Low,
        _ => Confidence::Medium,
    }
}
