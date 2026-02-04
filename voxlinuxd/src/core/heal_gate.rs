use crate::core::confidence::Confidence;
use crate::core::opinion::Opinion;
use crate::state::BootContext;

pub fn healing_allowed(
    health: &Opinion,
    systemd: &Opinion,
    confidence: Confidence,
    boot_context: BootContext,
) -> bool {
    // 🔒 HARD BOOT SAFETY RULE
    if boot_context != BootContext::Graphical {
        return false;
    }

    // 🔒 TRUST RULE
    if confidence != Confidence::High {
        return false;
    }

    // 🔒 SYSTEM INTEGRITY RULES
    match (health, systemd) {
        (Opinion::Broken { .. }, _) => false,
        (_, Opinion::Broken { .. }) => false,
        _ => true,
    }
}
