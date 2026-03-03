use crate::ipc_client;
use std::process::{Command, Stdio};
use std::io::Read;

pub fn show_notification(plan_id: String, explanation: String) {

    // 1️⃣ Send initial notification
    let mut first = Command::new("notify-send")
    .arg("⚠ VoxLinux detected an issue")
    .arg(format!("Recommended repair: {}", plan_id))
    .arg("--action=ai=🤖 AI Explanation")
    .arg("--print-id")
    .stdout(Stdio::piped())
    .spawn()
    .expect("Failed to send notification");

    let mut id = String::new();

    if let Some(mut stdout) = first.stdout.take() {
        stdout.read_to_string(&mut id).ok();
    }

    first.wait().ok();

    let notification_id = id.lines().next().unwrap_or("").trim().to_string();

    if notification_id.is_empty() {
        return;
    }

    // 2️⃣ Replace with explanation
    let mut second = Command::new("notify-send")
    .arg("🤖 AI Explanation")
    .arg(&explanation)
    .arg(format!("--replace-id={}", notification_id))
    .arg("--action=apply=Apply Repair")
    .arg("--print-id")
    .stdout(Stdio::piped())
    .spawn()
    .ok();

    if let Some(ref mut child) = second {
        let mut output = String::new();

        if let Some(mut stdout) = child.stdout.take() {
            stdout.read_to_string(&mut output).ok();
        }

        child.wait().ok();

        // 3️⃣ If user clicks Apply → send IPC
        if output.contains("apply") {
            ipc_client::send_apply(&plan_id);
        }
    }
}


