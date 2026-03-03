use std::collections::HashSet;
use std::fs;
use std::thread;
use std::time::Duration;

use crate::gui;

const PLAN_DIR: &str = "/run/voxlinux/plans";

pub fn run() {
    println!("[WATCHER] Watching for new repair plans...");

    let mut seen: HashSet<String> = HashSet::new();

    loop {
        if let Ok(entries) = fs::read_dir(PLAN_DIR) {
            for entry in entries.flatten() {
                let path = entry.path();

                if let Some(file_name) = path.file_name() {
                    if let Some(name) = file_name.to_str() {
                        if name.ends_with(".json") {
                            let plan_id = name.trim_end_matches(".json").to_string();

                            if !seen.contains(&plan_id) {
                                seen.insert(plan_id.clone());

                                // Load explanation from plan JSON
                                if let Ok(contents) = fs::read_to_string(&path) {
                                    if let Ok(plan) = serde_json::from_str::<serde_json::Value>(&contents) {

                                        let explanation = plan["explain"]
                                        .as_array()
                                        .map(|blocks| {
                                            blocks.iter()
                                            .map(|b| b["content"].as_str().unwrap_or(""))
                                            .collect::<Vec<_>>()
                                            .join("\n\n")
                                        })
                                        .unwrap_or_else(|| "AI explanation unavailable.".into());

                                        gui::show_notification(plan_id, explanation);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        thread::sleep(Duration::from_secs(3));
    }
}
