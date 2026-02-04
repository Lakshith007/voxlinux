use std::time::{SystemTime, UNIX_EPOCH};

pub fn note(message: String) {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    println!("[voxlinux][{}] {}", ts, message);
}

pub fn report(target: &str, action: &str) {
    println!("================ VoxLinux Report ================");
    println!("Detected failed service : {}", target);
    println!("Action taken            : {}", action);
    println!("Result                  : completed");
    println!("================================================");
}
