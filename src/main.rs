use clap::{CommandFactory, Parser, Subcommand};
use colored::*;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;

const PROGRAM_NAME: &str = "moree";
const DEFAULT_STATE_FILE: &str = "state.json";

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
    Managed,
    Unmanaged,
}

#[derive(Serialize, Deserialize, Debug)]
struct PackageState {
    explicit: bool,
    memo: Option<String>,
}

fn main() -> io::Result<()> {
    let args = Cli::parse();

    let state_file_path = get_state_file_path(&args.data)?;
    let mut package_states: HashMap<String, PackageState> = load_package_states(&state_file_path)?;

    match &args.command {
        Some(Commands::Add { packages }) => {
            if packages.is_empty() {
                interactive_mode(&mut package_states)?;
            } else {
                add_packages(&mut package_states, packages)?;
            }
        }
        Some(Commands::Apply {
            with_install,
            with_uninstall,
            sync,
        }) => {
            if *sync || (*with_install && *with_uninstall) {
                apply_changes(&package_states, true, true)?;
            } else {
                apply_changes(&package_states, *with_install, *with_uninstall)?;
            }
        }
        Some(Commands::Managed) => display_managed_packages(&package_states),
        Some(Commands::Unmanaged) => display_unmanaged_packages(&package_states)?,
        None => {
            println!("{}", Cli::command().render_long_help());
        }
    }

    save_package_states(&state_file_path, &package_states)?;
    Ok(())
}

// NOTE: 参照じゃなくても良いかも
fn get_custom_state_file_path(custom_path: &PathBuf) -> io::Result<PathBuf> {
    // 指定されたパスがディレクトリでないか確認
    if custom_path.is_dir() {
        eprintln!("Error: The provided path is a directory, not a file.");
        std::process::exit(1);
    }

    // ディレクトリがない場合は作成
    if let Some(parent_dir) = custom_path.parent() {
        fs::create_dir_all(parent_dir)?;
    }

    // ファイル名が .json で終わっているか確認
    if custom_path.extension().and_then(|ext| ext.to_str()) != Some("json") {
        println!("The provided path does not have a .json extension. Are you sure you want to use this path? (y/n)");
        let confirmation: String = Input::<String>::new()
            .interact_text()
            .unwrap_or_default()
            .to_lowercase();
        if confirmation != "y" {
            eprintln!("Operation canceled by the user.");
            std::process::exit(1);
        }
    }

    Ok(custom_path.clone())
}

fn get_default_state_file_path() -> io::Result<PathBuf> {
    let data_dir = std::env::var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| dirs::home_dir().unwrap().join(".local/share"))
        .join(PROGRAM_NAME);

    fs::create_dir_all(&data_dir)?;
    Ok(data_dir.join(DEFAULT_STATE_FILE))
}

fn get_state_file_path(custom_path: &Option<PathBuf>) -> io::Result<PathBuf> {
    match custom_path {
        Some(path) => get_custom_state_file_path(path),
        None => get_default_state_file_path(),
    }
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

fn add_packages(
    package_states: &mut HashMap<String, PackageState>,
    packages: &[String],
) -> io::Result<()> {
    let installed_packages = Command::new("paru")
        .arg("-Q")
        .output()
        .expect("Failed to execute paru -Q.");
    let installed_list: Vec<String> = String::from_utf8_lossy(&installed_packages.stdout)
        .lines()
        .map(|line| line.split_whitespace().next().unwrap().to_string())
        .collect();

    let mut missing_packages = vec![];

    for package in packages {
        if installed_list.contains(package) {
            handle_package_interactively(package_states, package)?;
        } else {
            missing_packages.push(package.clone());
        }
    }

    if !missing_packages.is_empty() {
        println!(
            "Error: Some packages are not installed: {:?}",
            missing_packages
        );
    }

    Ok(())
}

fn interactive_mode(package_states: &mut HashMap<String, PackageState>) -> io::Result<()> {
    let output = Command::new("paru")
        .arg("-Qe")
        .output()
        .expect("Failed to execute paru -Qe.");
    let output_str = String::from_utf8_lossy(&output.stdout);

    for line in output_str.lines() {
        let package_name = line.split_whitespace().next().unwrap().to_string();
        if let Some(state) = package_states.get(&package_name) {
            if state.explicit {
                continue;
            }
        }
        if handle_package_interactively(package_states, &package_name).is_err() {
            loop {
                println!("Would you like to save your progress? (Y/n)");
                let save: String = Input::<String>::new()
                    .interact_text()
                    .unwrap_or_default()
                    .to_lowercase();
                if save == "y" || save == "n" {
                    if save == "y" {
                        break;
                    }
                } else {
                    println!("Invalid input. Please enter 'y' or 'n'.");
                }
            }
        }
    }

    Ok(())
}

fn handle_package_interactively(
    package_states: &mut HashMap<String, PackageState>,
    package_name: &str,
) -> io::Result<()> {
    println!("{}", format!("Package: {}", package_name).bold().cyan());
    // TODO: もっと小さい情報を見せる
    display_package_details(package_name);

    let options = vec!["Yes (y)", "No (n)", "Skip (s)", "Details (d)", "Quit (q)"];
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
        'q' | 'Q' => return Err(io::Error::new(io::ErrorKind::Other, "Quit")),
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

fn display_managed_packages(package_states: &HashMap<String, PackageState>) {
    println!("{}", "Managed Packages:".bold().green());
    for (name, state) in package_states {
        if state.explicit {
            let memo = state.memo.as_deref().unwrap_or("No memo provided.");
            println!("{} - {}", name.bold(), memo);
        }
    }
}

fn display_unmanaged_packages(package_states: &HashMap<String, PackageState>) -> io::Result<()> {
    let output = Command::new("paru")
        .arg("-Qe")
        .output()
        .expect("Failed to execute paru -Qe.");
    let output_str = String::from_utf8_lossy(&output.stdout);

    println!("{}", "Unmanaged Packages:".bold().red());
    for line in output_str.lines() {
        let package_name = line.split_whitespace().next().unwrap().to_string();
        if !package_states.contains_key(&package_name) {
            println!("{}", package_name.bold());
        }
    }

    Ok(())
}

fn apply_changes(
    package_states: &HashMap<String, PackageState>,
    with_install: bool,
    with_uninstall: bool,
) -> io::Result<()> {
    if with_install {
        for (package, state) in package_states {
            if state.explicit && !is_installed(package) {
                Command::new("paru")
                    .arg("-S")
                    .arg(package)
                    .status()
                    .expect("Failed to install package.");
            }
        }
    }
    if with_uninstall {
        let output = Command::new("paru")
            .arg("-Qe")
            .output()
            .expect("Failed to execute paru -Qe.");
        let output_str = String::from_utf8_lossy(&output.stdout);

        for line in output_str.lines() {
            let package_name = line.split_whitespace().next().unwrap();
            if !package_states.contains_key(package_name) {
                Command::new("paru")
                    .arg("-R")
                    .arg(package_name)
                    .status()
                    .expect("Failed to uninstall package.");
            }
        }
    }
    Ok(())
}

fn is_installed(package: &str) -> bool {
    let status = Command::new("paru").arg("-Qi").arg(package).status();
    status.map(|s| s.success()).unwrap_or(false)
}
