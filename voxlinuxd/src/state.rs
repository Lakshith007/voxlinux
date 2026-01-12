use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs;
use serde::{Serialize, Deserialize};

const STATE_DIR: &str = "/var/lib/voxlinux";
const STATE_FILE: &str = "/var/lib/voxlinux/state.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct HealState {
    retries: HashMap<String, u8>,
    next_retry: HashMap<String, u64>,
}

impl HealState {
    /// Load persistent state from disk
    pub fn load() -> Self {
        if let Ok(data) = fs::read_to_string(STATE_FILE) {
            if let Ok(state) = serde_json::from_str(&data) {
                return state;
            }
        }

        HealState {
            retries: HashMap::new(),
            next_retry: HashMap::new(),
        }
    }

    /// Save state to disk
    fn save(&self) {
        let _ = fs::create_dir_all(STATE_DIR);
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(STATE_FILE, json);
        }
    }

    /// Decide whether a service should be retried (with exponential backoff)
    pub fn should_retry(&mut self, service: &str) -> bool {
        let retries = self.retries.entry(service.to_string()).or_insert(0);

        // Max 3 attempts
        if *retries >= 3 {
            self.save();
            return false;
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Respect backoff window
        if let Some(next) = self.next_retry.get(service) {
            if now < *next {
                return false;
            }
        }

        *retries += 1;

        // Exponential backoff: 10s, 20s, 40s
        let delay = 10 * (1 << (*retries - 1));
        self.next_retry
            .insert(service.to_string(), now + delay);

        self.save();
        true
    }

    /// Reconcile persistent state with live systemd failures
    pub fn reconcile_with_systemd(&mut self, failed_services: &[String]) {
        let mut still_failed = HashMap::new();
        for s in failed_services {
            still_failed.insert(s.clone(), true);
        }

        // Forget services that are no longer failed
        self.retries.retain(|svc, _| still_failed.contains_key(svc));
        self.next_retry.retain(|svc, _| still_failed.contains_key(svc));

        self.save();
    }
}
