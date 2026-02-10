use crate::state::BootContext;
use crate::core::confidence::Confidence;
use crate::core::classifier::Detection;

use std::time::Instant;

#[derive(Debug, Clone)]
pub struct DeferredHealAction {
    pub unit: String,
    pub reason: String,
    pub created_at: Instant,
}

#[derive(Debug, Default)]
pub struct DeferredHealQueue {
    actions: Vec<DeferredHealAction>,
}

impl DeferredHealQueue {
    pub fn enqueue(&mut self, detection: &Detection) {
        if self.actions.iter().any(|a| a.unit == detection.unit) {
            return; // already queued
        }

        self.actions.push(DeferredHealAction {
            unit: detection.unit.clone(),
                          reason: detection.reason.clone(),
                          created_at: Instant::now(),
        });
    }

    pub fn try_execute<F>(
        &mut self,
        boot_context: BootContext,
        confidence: Confidence,
        mut exec: F,
    )
    where
    F: FnMut(&DeferredHealAction),
    {
        if boot_context != BootContext::Graphical {
            return;
        }

        if confidence != Confidence::High {
            return;
        }

        for action in self.actions.drain(..) {
            exec(&action);
        }
    }

    pub fn clear(&mut self) {
        self.actions.clear();
    }
}
