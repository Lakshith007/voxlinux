use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::channel;

use crate::gui;
use voxlinux::repair_plan::RepairPlan;

const PLAN_DIR: &str = "/run/voxlinux/plans";

pub fn run() {

    println!("[WATCHER] Using filesystem watcher");

    let (tx, rx) = channel();

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            tx.send(res).unwrap();
        },
        Config::default(),
    )
    .expect("Failed to create watcher");

    watcher
    .watch(Path::new(PLAN_DIR), RecursiveMode::NonRecursive)
    .expect("Failed to watch plan directory");

    loop {

        match rx.recv() {

            Ok(Ok(event)) => {

                if matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_)) {

                    for path in event.paths {

                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {

                            if !name.ends_with(".json") {
                                continue;
                            }

                            // Small delay to avoid partial reads
                            std::thread::sleep(std::time::Duration::from_millis(100));

                            println!("[WATCHER] NEW PLAN FILE: {}", name);

                            match std::fs::read_to_string(&path) {

                                Ok(contents) => {

                                    match serde_json::from_str::<RepairPlan>(&contents) {

                                        Ok(plan) => {

                                            let explanation = plan
                                            .explain
                                            .iter()
                                            .map(|b| b.content.clone())
                                            .collect::<Vec<_>>()
                                            .join("\n\n");

                                            gui::show_notification(plan.id, explanation);
                                        }

                                        Err(e) => {
                                            println!("[WATCHER] JSON parse failed: {}", e);
                                        }
                                    }
                                }

                                Err(e) => {
                                    println!("[WATCHER] Failed to read plan: {}", e);
                                }
                            }
                        }
                    }
                }
            }

            Ok(Err(e)) => {
                println!("[WATCHER ERROR] {:?}", e);
            }

            Err(e) => {
                println!("[WATCHER CHANNEL ERROR] {:?}", e);
            }
        }
    }
}
