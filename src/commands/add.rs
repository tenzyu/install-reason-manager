use crate::moree_state_manager::{self, PackageState};
use crate::package_manager_integration;
use crate::utils;
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
    let packages_to_process = get_packages_to_process(packages)?;
    let mut should_save = true; // Flag to track whether to save

    for package_name in packages_to_process {
        if package_states
            .get(&package_name)
            .map_or(false, |p| p.explicit)
        {
            continue;
        }

        match handle_package_interactively(package_states, &package_name) {
            Ok(_) => {} // Continue to the next package
            Err(e) if e.to_string() == "Quit" => {
                should_save = utils::confirm_prompt("Save changes before quitting? (Y/n)", true)?;
                break; // Exit the loop
            }
            Err(e) => return Err(e), // Handle other errors
        }
    }

    println!("{}", &should_save);
    if should_save {
        moree_state_manager::save_package_states(state_file_path, package_states)?;
    }

    Ok(())
}

fn get_packages_to_process(packages: &[String]) -> io::Result<Vec<String>> {
    if packages.is_empty() {
        package_manager_integration::get_installed_packages_vec()
    } else {
        let installed_packages = package_manager_integration::get_installed_packages_hashset()?;
        let missing_packages: Vec<_> = packages
            .iter()
            .filter(|p| !installed_packages.contains(*p))
            .cloned()
            .collect();

        if !missing_packages.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "The following packages are not installed: {:?}",
                    missing_packages
                ),
            ));
        }
        Ok(packages.to_vec())
    }
}

fn handle_package_interactively(
    package_states: &mut HashMap<String, PackageState>,
    package_name: &str,
) -> io::Result<()> {
    println!("{}", format!("Package: {}", package_name).bold().cyan());
    package_manager_integration::display_package_details(package_name)?;

    let options = &["Yes", "No", "Skip", "Quit"]; // Make options a slice.
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Did you explicitly install this package?")
        .items(options)
        .default(0)
        .interact()
        .unwrap();

    match options[selection] {
        // Direct match
        "Yes" => {
            let memo: String = Input::new()
                .with_prompt("Why did you install this package? (optional)")
                .allow_empty(true)
                .interact_text()
                .unwrap_or_default();

            package_states.insert(
                package_name.to_string(),
                PackageState {
                    explicit: true,
                    memo: Some(memo).filter(|s| !s.is_empty()), // More concise
                },
            );
        }
        "No" => {
            package_states.insert(
                package_name.to_string(),
                PackageState {
                    explicit: false,
                    memo: None,
                },
            );
        }
        "Skip" => {}
        "Quit" => return Err(io::Error::new(io::ErrorKind::Other, "Quit")), // Clean return
        _ => unreachable!(), // This shouldn't be possible given the Select.
    }

    Ok(())
}
