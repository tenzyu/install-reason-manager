use clap::Parser;
use colored::*;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;

const PROGRAM_NAME: &str = "install_reason_manager";
const STATE_FILE: &str = "state.json";

#[derive(Parser)]
#[command(name = PROGRAM_NAME)]
#[command(about = "Manage explicitly installed packages interactively.")]
struct Cli {
    #[arg(long)]
    list: bool,
    #[arg(long)]
    reinstall: bool,
    // #[arg(long)]
    // cleanup: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct PackageState {
    explicit: bool,
    memo: Option<String>,
}

fn main() -> io::Result<()> {
    let args = Cli::parse();
    let data_dir = get_data_dir()?;
    let state_file_path = data_dir.join(STATE_FILE);

    let mut package_states: HashMap<String, PackageState> = load_package_states(&state_file_path)?;

    if args.list {
        display_explicit_packages(&package_states);
        return Ok(());
    }

    let output = Command::new("paru")
        .arg("-Qe")
        .output()
        .expect("Failed to execute paru.");
    let output_str = String::from_utf8_lossy(&output.stdout);

    if args.reinstall {
        reinstall_as_deps(&output_str, &package_states)?;
        return Ok(());
    }
    // if args.cleanup {
    // cleanup_packages(&output_str, &package_states)?;
    // return Ok(());
    // }

    for line in output_str.lines() {
        let package_name = line.split_whitespace().next().unwrap().to_string();
        if let Some(state) = package_states.get(&package_name) {
            if state.explicit {
                continue;
            }
        }
        handle_package_interactively(&mut package_states, &package_name)?;
    }

    save_package_states(&state_file_path, &package_states)?;

    Ok(())
}

fn get_data_dir() -> io::Result<PathBuf> {
    let data_dir = std::env::var("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| dirs::home_dir().unwrap().join(".local/state"))
        .join(PROGRAM_NAME);
    fs::create_dir_all(&data_dir)?;
    Ok(data_dir)
}

fn load_package_states(file_path: &PathBuf) -> io::Result<HashMap<String, PackageState>> {
    if let Ok(data) = fs::read_to_string(file_path) {
        serde_json::from_str(&data).or_else(|_| Ok(HashMap::new()))
    } else {
        Ok(HashMap::new())
    }
}

fn save_package_states(
    file_path: &PathBuf,
    package_states: &HashMap<String, PackageState>,
) -> io::Result<()> {
    let data = serde_json::to_string(package_states).unwrap();
    fs::write(file_path, data)
}

fn handle_package_interactively(
    package_states: &mut HashMap<String, PackageState>,
    package_name: &str,
) -> io::Result<()> {
    println!("{}", format!("Package: {}", package_name).bold().cyan());
    // TODO: もっと小さい情報を見せる
    display_package_details(package_name);

    let options = vec!["Yes (y)", "No (n)", "Skip (s)", "Details (d)"];
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Did you explicitly install this package?")
        .items(&options)
        .default(0)
        .interact()
        .unwrap();

    match options[selection].chars().next().unwrap() {
        'y' | 'Y' => {
            let memo: String = Input::new()
                .with_prompt("Why did you install this package? (optional)")
                .allow_empty(true)
                .interact_text()
                .unwrap();

            package_states.insert(
                package_name.to_string(),
                PackageState {
                    explicit: true,
                    memo: if memo.is_empty() { None } else { Some(memo) },
                },
            );
        }
        'n' | 'N' => {
            package_states.insert(
                package_name.to_string(),
                PackageState {
                    explicit: false,
                    memo: None,
                },
            );
        }
        's' | 'S' => {}
        'd' | 'D' => {
            display_package_details(package_name);
            handle_package_interactively(package_states, package_name)?;
        }
        _ => {}
    }

    Ok(())
}

fn display_package_details(package_name: &str) {
    let qi_output = Command::new("paru")
        .arg("-Qi")
        .arg(package_name)
        .output()
        .expect("Failed to execute paru -Qi.");
    let qi_output_str = String::from_utf8_lossy(&qi_output.stdout);

    // TODO: pactree のOUTPUTをいい感じに使う
    //
    // let pactree_output = Command::new("pactree")
    //     .arg("-u")
    //     .arg(package_name)
    //     .output()
    //     .expect("Failed to execute pactree.");
    // let pactree_output_str = String::from_utf8_lossy(&pactree_output.stdout);

    println!("{}", "\nPackage Details:".bold().yellow());
    println!("{}", qi_output_str);
    // println!("{}", "\nDirect Dependencies Tree:".bold().yellow());
    // println!("{}", pactree_output_str);
}

fn display_explicit_packages(package_states: &HashMap<String, PackageState>) {
    println!("{}", "Explicitly Installed Packages:".bold().green());
    for (name, state) in package_states {
        if state.explicit {
            let memo = state.memo.as_deref().unwrap_or("No memo provided.");
            println!("{} - {}", name.bold(), memo);
        }
    }
}

fn reinstall_as_deps(
    installed_output: &str,
    package_states: &HashMap<String, PackageState>,
) -> io::Result<()> {
    let mut to_reinstall = Vec::new();
    for line in installed_output.lines() {
        let package_name = line.split_whitespace().next().unwrap().to_string();
        if let Some(state) = package_states.get(&package_name) {
            if !state.explicit {
                to_reinstall.push(package_name);
            }
        }
    }

    if !to_reinstall.is_empty() {
        println!(
            "{}",
            format!("Re-installing as deps {} packages...", to_reinstall.len())
                .bold()
                .red()
        );
        Command::new("paru")
            .arg("-S")
            .arg("--asdeps")
            .args(&to_reinstall)
            .status()
            .expect("Failed to execute paru -S --asdeps.");
    }
    Ok(())
}
// fn cleanup_packages(
//     installed_output: &str,
//     package_states: &HashMap<String, PackageState>,
// ) -> io::Result<()> {
//     let mut to_remove = Vec::new();
//     for line in installed_output.lines() {
//         let package_name = line.split_whitespace().next().unwrap().to_string();
//         if let Some(state) = package_states.get(&package_name) {
//             if !state.explicit {
//                 to_remove.push(package_name);
//             }
//         }
//     }

//     if !to_remove.is_empty() {
//         println!(
//             "{}",
//             format!("Cleaning up {} packages...", to_remove.len())
//                 .bold()
//                 .red()
//         );
//         Command::new("paru")
//             .arg("-Rns")
//             .args(&to_remove)
//             .status()
//             .expect("Failed to execute paru -R.");
//     }
//     Ok(())
// }
