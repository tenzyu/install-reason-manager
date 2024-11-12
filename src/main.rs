mod commands;
mod moree_state_manager;
mod package_manager_integration;
mod utils;

use clap::{CommandFactory, Parser, Subcommand};
use std::io;
use std::path::PathBuf;

const PROGRAM_NAME: &str = "moree";

#[derive(Parser)]
#[command(name = PROGRAM_NAME)]
#[command(about = "Manage packages and their explicit installation reasons")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(long, value_name = "path")]
    data: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    Add {
        packages: Vec<String>,
    },
    Apply {
        #[arg(long)]
        with_install: bool,
        #[arg(long)]
        with_uninstall: bool,
        #[arg(long)]
        sync: bool,
    },
    Unmanaged,
    Diff {
        #[arg(long)]
        all: bool,
    },
    Edit {
        package: String,
    },
    Query {
        #[arg(short, long)]
        information: bool,
        #[arg(short, long)]
        explicit: bool,
        #[arg(short, long)]
        deps: bool,
    },
}

fn main() -> io::Result<()> {
    let args = Cli::parse();
    let state_file_path = moree_state_manager::get_state_file_path(&args.data)?;
    let mut package_states = moree_state_manager::load_package_states(&state_file_path)?;

    let result = match &args.command {
        Some(Commands::Add { packages }) => {
            commands::add::run(&mut package_states, &state_file_path, packages)
        }
        Some(Commands::Apply {
            with_install,
            with_uninstall,
            sync,
        }) => commands::apply::run(&package_states, *with_install, *with_uninstall, *sync),
        Some(Commands::Unmanaged) => commands::unmanaged::run(&package_states),
        Some(Commands::Diff { all }) => commands::diff::run(&package_states, *all),
        Some(Commands::Edit { package }) => {
            commands::edit::run(&mut package_states, &state_file_path, package)
        }
        Some(Commands::Query {
            information,
            explicit,
            deps,
        }) => commands::query::run(&package_states, *information, *explicit, *deps),
        None => {
            println!("{}", Cli::command().render_long_help());
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
