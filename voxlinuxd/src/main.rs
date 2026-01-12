mod health;
mod systemd;
mod pacman;
mod state;
mod explain;

use state::HealState;

fn main() {
    println!("voxlinuxd: self-healing engine started");

    let boot_mode = std::env::var("INVOCATION_ID").is_ok();
    let mut heal_state = HealState::load();

    if boot_mode {
        println!("voxlinuxd: boot-time recovery mode active");
    }

    loop {
        // Reconcile persistent state
        let failed_services = health::failed_services();
        heal_state.reconcile_with_systemd(&failed_services);

        // Boot-time: pacman recovery has highest priority
        if pacman::pacman_broken() {
            let action = pacman::heal();
            explain::report("pacman", &action);

            if boot_mode {
                break; // run once at boot
            }
        }

        // systemd healing
        match health::check() {
            Some(service) => {
                if heal_state.should_retry(&service) {
                    let action = systemd::heal(&service);
                    explain::report(&service, &action);
                }
            }
            None => {}
        }

        if boot_mode {
            break; // boot recovery runs once
        }

        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}
