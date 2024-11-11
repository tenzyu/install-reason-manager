use crate::moree_state_manager::{self, PackageState};
use crate::package_manager_integration;

use colored::*;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

pub fn run(
    package_states: &mut HashMap<String, PackageState>,
    state_file_path: &PathBuf,
    packages: &[String],
) -> io::Result<()> {
    let packages_to_process = if packages.is_empty() {
        package_manager_integration::get_installed_packages_vec()?
    } else {
        let installed_packages = package_manager_integration::get_installed_packages_hashset()?;
        let mut missing_packages = Vec::new();
        let mut packages_to_process = Vec::new();

        for package in packages {
            if installed_packages.contains(package) {
                packages_to_process.push(package.clone());
            } else {
                missing_packages.push(package.clone());
            }
        }

        if !missing_packages.is_empty() {
            eprintln!(
                "Error: The following packages are not installed: {:?}",
                missing_packages
            );
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Some packages are not installed.",
            ));
        }
        packages_to_process
    };

    let mut save_enabled = true;

    'outer: for package_name in packages_to_process {
        if package_states
            .get(&package_name)
            .map(|p| p.explicit)
            .unwrap_or(false)
        {
            continue;
        }

        if handle_package_interactively(package_states, &package_name).is_err() {
            loop {
                println!("Would you like to save your progress? (y/n)");
                let save: String = Input::<String>::new()
                    .interact_text()
                    .unwrap_or_default()
                    .to_lowercase();
                if save == "y" || save == "n" {
                    if save == "n" {
                        save_enabled = false;
                    }
                    break 'outer;
                } else {
                    println!("Invalid input. Please enter 'y' or 'n'.");
                }
            }
        }
    }

    if save_enabled {
        moree_state_manager::save_package_states(state_file_path, package_states)?;
    }

    Ok(())
}

fn handle_package_interactively(
    package_states: &mut HashMap<String, PackageState>,
    package_name: &str,
) -> io::Result<()> {
    println!("{}", format!("Package: {}", package_name).bold().cyan());
    package_manager_integration::display_package_details(package_name)?;

    let options = vec!["Yes", "No", "Skip", "Quit"];
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
                .unwrap_or_default();

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
        'q' | 'Q' => return Err(io::Error::new(io::ErrorKind::Other, "Quit")),
        _ => {}
    }

    Ok(())
}
