use crate::core::confidence::Confidence;
use crate::core::opinion::Opinion;
use crate::core::reporter::ObserverConfidence;

pub fn evaluate(
    health: &Opinion,
    systemd: &Opinion,
    observer_confidence: ObserverConfidence,
) -> Confidence {
    let engine_confidence = evaluate_engine_confidence(health, systemd);

    let observer_cap: Confidence = observer_confidence.into();

    engine_confidence.min(observer_cap)
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
