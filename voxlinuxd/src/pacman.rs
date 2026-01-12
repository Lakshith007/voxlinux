use std::path::Path;
use std::process::Command;

pub fn pacman_broken() -> bool {
    Path::new("/var/lib/pacman/db.lck").exists()
}

pub fn heal() -> String {
    let _ = Command::new("rm")
        .arg("-f")
        .arg("/var/lib/pacman/db.lck")
        .status();

    "removed stale pacman lock".to_string()
}
