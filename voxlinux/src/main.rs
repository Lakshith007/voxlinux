use std::process::Command;
use std::env;

fn main() {
    let exe_dir = env::current_exe()
    .expect("Cannot get current exe path")
    .parent()
    .unwrap()
    .to_path_buf();

    let daemon_path = exe_dir.join("voxlinuxd");

    Command::new(daemon_path)
    .spawn()
    .expect("Failed to start voxlinuxd");

    println!("VoxLinux daemon started.");
}
