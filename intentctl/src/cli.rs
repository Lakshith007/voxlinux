use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "intentctl")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Repair {
        #[command(subcommand)]
        action: RepairAction,
    },
}

#[derive(Subcommand)]
pub enum RepairAction {
    List,

    Explain {
        id: String,

        #[arg(long, default_value_t = 1)]
        level: u8,
    },

    Apply {
        id: String,

        #[arg(long)]
        yes: bool,

        #[arg(long)]
        dry_run: bool,
    },

    Advise {
        #[arg(long)]
        json: bool,
    },

    Interactive,
}
