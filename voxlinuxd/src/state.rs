use std::collections::HashMap;
use std::fs;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootContext {
    EarlyBoot,
    EarlyUserspace,
    MultiUser,
    Graphical,
    Rescue,
    Unknown,
}


const STATE_DIR: &str = "/var/lib/voxlinux";
const STATE_FILE: &str = "/var/lib/voxlinux/state.json";

/// Global singleton state (thread-safe)
static STATE: OnceLock<Mutex<HealState>> = OnceLock::new();

#[derive(Debug, Serialize, Deserialize)]
pub struct HealState {
    // ─────────────────────────────────────────
    // Reactive healing (backoff & retries)
    // ─────────────────────────────────────────
    retries: HashMap<String, u8>,
    next_retry: HashMap<String, u64>,

    // ─────────────────────────────────────────
    // Healing escalation level (L1–L4)
    // ─────────────────────────────────────────
    healing_level: HashMap<String, u8>,

    // ─────────────────────────────────────────
    // Confidence scoring (0.0 – 1.0)
    // ─────────────────────────────────────────
    confidence: HashMap<String, f32>,

    // ─────────────────────────────────────────
    // Predictive healing (history & trends)
    // ─────────────────────────────────────────
    last_restart_count: HashMap<String, u32>,
    last_observed_at: HashMap<String, u64>,
}

impl HealState {
    fn new() -> Self {
        HealState {
            retries: HashMap::new(),
            next_retry: HashMap::new(),
            healing_level: HashMap::new(),
            confidence: HashMap::new(),
            last_restart_count: HashMap::new(),
            last_observed_at: HashMap::new(),
        }
    }

    fn load_from_disk() -> Self {
        if let Ok(data) = fs::read_to_string(STATE_FILE) {
            if let Ok(state) = serde_json::from_str(&data) {
                return state;
            }
        }
        HealState::new()
    }

    fn save_to_disk(&self) {
        let _ = fs::create_dir_all(STATE_DIR);
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(STATE_FILE, json);
        }
    }
}

/// Access the global state safely
fn with_state<F, R>(f: F) -> R
where
F: FnOnce(&mut HealState) -> R,
{
    let mutex = STATE.get_or_init(|| {
        Mutex::new(HealState::load_from_disk())
    });

    let mut guard = mutex.lock().unwrap();
    let result = f(&mut guard);
    guard.save_to_disk();
    result
}

/// Current UNIX timestamp
fn now_ts() -> u64 {
    SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs()
}

//
// ─────────────────────────────────────────────
// Reactive self-healing (backoff & retries)
// ─────────────────────────────────────────────
//

pub fn should_retry(key: &str) -> bool {
    with_state(|state| {
        let retries = state.retries.entry(key.to_string()).or_insert(0);

        // Max 3 retry attempts per level
        if *retries >= 3 {
            return false;
        }

        let now = now_ts();

        if let Some(next) = state.next_retry.get(key) {
            if now < *next {
                return false;
            }
        }

        *retries += 1;

        // Exponential backoff: 10s, 20s, 40s
        let delay = 10 * (1 << (*retries - 1));
        state
        .next_retry
        .insert(key.to_string(), now + delay);

        true
    })
}

//
// ─────────────────────────────────────────────
// Healing level management (L1–L4)
// ─────────────────────────────────────────────
//

/// Get current healing level for a state (default = L1)
pub fn current_level(key: &str) -> u8 {
    with_state(|state| {
        *state.healing_level.get(key).unwrap_or(&1)
    })
}

/// Escalate healing level safely (max L4)
pub fn escalate_level(key: &str) {
    with_state(|state| {
        let level = state.healing_level.entry(key.to_string()).or_insert(1);
        if *level < 4 {
            *level += 1;
        }
    });
}

/// Reset healing state after successful recovery
pub fn reset_level(key: &str) {
    with_state(|state| {
        state.healing_level.remove(key);
        state.retries.remove(key);
        state.next_retry.remove(key);
        state.confidence.remove(key);
    });
}

//
// ─────────────────────────────────────────────
// Confidence scoring (Step 2)
// ─────────────────────────────────────────────
//

/// Get confidence for a state (default = 0.5)
pub fn get_confidence(key: &str) -> f32 {
    with_state(|state| {
        *state.confidence.get(key).unwrap_or(&0.5)
    })
}

/// Set confidence explicitly (clamped 0.0 – 1.0)
pub fn set_confidence(key: &str, value: f32) {
    with_state(|state| {
        state
        .confidence
        .insert(key.to_string(), value.clamp(0.0, 1.0));
    });
}

/// Increase confidence after successful healing
pub fn bump_confidence(key: &str) {
    with_state(|state| {
        let c = state.confidence.entry(key.to_string()).or_insert(0.5);
        *c = (*c + 0.1).min(1.0);
    });
}

/// Decrease confidence after failed healing
pub fn drop_confidence(key: &str) {
    with_state(|state| {
        let c = state.confidence.entry(key.to_string()).or_insert(0.5);
        *c = (*c - 0.15).max(0.0);
    });
}

//
// ─────────────────────────────────────────────
// Predictive self-healing (history & trends)
// ─────────────────────────────────────────────
//

pub fn get_last_restart_count(service: &str) -> Option<u32> {
    with_state(|state| {
        state.last_restart_count.get(service).copied()
    })
}

pub fn set_last_restart_count(service: &str, count: u32) {
    with_state(|state| {
        state
        .last_restart_count
        .insert(service.to_string(), count);
        state
        .last_observed_at
        .insert(service.to_string(), now_ts());
    });
}
