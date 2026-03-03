mod reader;
mod executor;
mod explain_cmd;
mod advisor;
mod gui;
mod ipc_client;
mod watcher;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "intentctl")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Repair {
        #[command(subcommand)]
        action: RepairAction,
    },

    Notify {
        plan_id: String,
        explanation: String,
    },
    Watch,
}

#[derive(Subcommand)]
enum RepairAction {
    List,
    Explain { id: String, level: Option<u8> },
    Apply { id: String, yes: bool, dry_run: bool },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Notify { plan_id, explanation } => {
            gui::show_notification(plan_id, explanation);
        }

        Commands::Watch => {
            watcher::run();
        }

        Commands::Repair { action } => {
            match action {
                RepairAction::List => {
                    reader::list_plans();
                }
                RepairAction::Explain { id, level } => {
                    let level = level.unwrap_or(1); // default level = 1
                    explain_cmd::explain_plan(&id, level);
                }

                RepairAction::Apply { id, yes, dry_run } => {
                    if let Some(plan) = reader::find_plan(&id) {
                        executor::apply_plan(plan, yes, dry_run);
                    } else {
                        println!("Plan not found.");
                    }
                }
            }
        }
    }
}
