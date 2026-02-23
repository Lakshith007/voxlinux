#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealingLevel {
    ObserveOnly,      // Stage 0
    RuntimeSafe,      // Stage 1
    AssistedRepair,   // Stage 2
}

