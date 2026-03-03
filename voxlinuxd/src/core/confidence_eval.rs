use crate::core::confidence::Confidence;
use crate::core::opinion::Opinion;
use crate::state::BootContext;

pub fn evaluate(
    health: &Opinion,
    systemd: &Opinion,
    observer_conf: Confidence,
) -> Confidence {
    match (health, systemd) {
        (_, Opinion::Broken { .. }) => Confidence::Low,

        (Opinion::Broken { .. }, _) => Confidence::Low,

        // If only degraded and boot stable → keep High
        (Opinion::Ok, Opinion::Degraded { .. }) => {
            if observer_conf == Confidence::High {
                Confidence::High
            } else {
                Confidence::Medium
            }
        }

        (Opinion::Degraded { .. }, Opinion::Ok) => Confidence::Medium,

        _ => observer_conf,
    }
}
