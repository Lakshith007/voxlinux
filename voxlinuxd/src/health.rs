use crate::probe;
use crate::core::opinion::Opinion;

pub fn assess() -> Opinion {
    if probe::disk_full() {
        return Opinion::Broken {
            reason: "Root filesystem usage is critically high".to_string(),
        };
    }

    if probe::boot_degraded() {
        return Opinion::Degraded {
            reason: "Boot completed with degraded systemd state".to_string(),
        };
    }

    Opinion::Ok
}

