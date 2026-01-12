use std::process::Command;

/// Return ONE failed service (used for healing decision)
pub fn check() -> Option<String> {
    let output = Command::new("systemctl")
        .arg("--failed")
        .arg("--no-legend")
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.lines().next()?;

    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 {
        Some(parts[1].to_string())
    } else {
        None
    }
}

/// Return ALL failed services (used for reconciliation)
pub fn failed_services() -> Vec<String> {
    let output = Command::new("systemctl")
        .arg("--failed")
        .arg("--no-legend")
        .output();

    let output = match output {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);

    stdout
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                Some(parts[1].to_string())
            } else {
                None
            }
        })
        .collect()
}
