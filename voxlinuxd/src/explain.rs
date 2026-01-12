pub fn report(target: &str, action: &str) {
    println!("================ VoxLinux Report ================");
    println!("Detected failed service : {}", target);
    println!("Action taken            : {}", action);
    println!("Result                  : completed");
    println!("================================================");
}
