use notify::{RecommendedWatcher, RecursiveMode, Watcher, EventKind};
use std::sync::mpsc::channel;
use std::path::Path;
use crate::gui;

const PLAN_DIR: &str = "/run/voxlinux/plans";

pub fn run() {
    println!("[WATCHER] Using inotify watcher...");

    let (tx, rx) = channel();

    let mut watcher: RecommendedWatcher =
    Watcher::new(tx, notify::Config::default()).expect("Failed to create watcher");

    watcher
    .watch(Path::new(PLAN_DIR), RecursiveMode::NonRecursive)
    .expect("Failed to watch plan directory");

    loop {
        match rx.recv() {
            Ok(event) => {
                if let Ok(ev) = event {
                    if matches!(ev.kind, EventKind::Create(_)) {
                        for path in ev.paths {
                            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                if name.ends_with(".json") {
                                    let plan_id =
                                    name.trim_end_matches(".json").to_string();

                                    println!("[WATCHER] NEW PLAN: {}", plan_id);

                                    if let Ok(contents) =
                                        std::fs::read_to_string(&path)
                                        {
                                            if let Ok(plan) =
                                                serde_json::from_str::<serde_json::Value>(&contents)
                                                {
                                                    let explanation = plan["explain"]
                                                    .as_array()
                                                    .map(|blocks| {
                                                        blocks
                                                        .iter()
                                                        .map(|b| {
                                                            b["content"]
                                                            .as_str()
                                                            .unwrap_or("")
                                                        })
                                                        .collect::<Vec<_>>()
                                                        .join("\n\n")
                                                    })
                                                    .unwrap_or_else(|| {
                                                        "AI explanation unavailable.".into()
                                                    });

                                                    gui::show_notification(plan_id, explanation);
                                                }
                                        }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => println!("[WATCHER ERROR] {:?}", e),
        }
    }
}
