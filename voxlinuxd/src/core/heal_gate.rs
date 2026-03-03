use crate::core::confidence::Confidence;
use crate::core::opinion::Opinion;
use crate::state::BootContext;

pub fn healing_allowed(
    health: &Opinion,
    systemd: &Opinion,
    confidence: Confidence,
    boot: BootContext,
) -> bool {

    // Never heal in early boot phases
    match boot {
        BootContext::EarlyBoot |
        BootContext::EarlyUserspace |
        BootContext::Rescue |
        BootContext::Unknown => return false,
        _ => {}
    }

    // Never heal if core system integrity is broken
    if matches!(systemd, Opinion::Broken { .. }) {
        return false;
    }

    // Never heal if global health is broken
    if matches!(health, Opinion::Broken { .. }) {
        return false;
    }

    // Require high confidence for autonomy
    if confidence != Confidence::High {
        return false;
    }

    true
}
