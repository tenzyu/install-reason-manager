use crate::moree_state_manager::{self, PackageState};
use crate::utils;
use colored::*;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

pub fn run(
    package_states: &mut HashMap<String, PackageState>,
    state_file_path: &PathBuf,
    package_name: &str,
) -> io::Result<()> {
    if !package_states.contains_key(package_name) {
        println!(
            "{} {} {}",
            "Package".bold().yellow(),
            package_name.bold().yellow(),
            "is not managed.  Use `add` to manage this package."
                .bold()
                .yellow(),
        );
        return Ok(()); // Or Err if you want to treat this as an error condition
    }

    let options = &["Explicit Status", "Memo", "Quit"];
    loop {
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "Edit {} (current explicit status: {})",
                package_name.bold(),
                package_states[package_name].explicit
            ))
            .items(options)
            .default(0)
            .interact()
            .unwrap();

        match options[selection] {
            "Explicit Status" => {
                let new_status = utils::confirm_prompt(
                    "Is this package explicitly installed?",
                    package_states[package_name].explicit,
                )?;
                package_states
                    .entry(package_name.to_string())
                    .and_modify(|state| {
                        state.explicit = new_status;
                    }); // Use and_modify to update in place
            }
            "Memo" => {
                let current_memo = package_states[package_name]
                    .memo
                    .clone()
                    .unwrap_or_default();
                let memo: String = Input::new()
                    .with_prompt("Enter new memo (optional, enter to keep current memo):")
                    .with_initial_text(current_memo)
                    .allow_empty(true)
                    .interact_text()
                    .unwrap();

                package_states
                    .entry(package_name.to_string())
                    .and_modify(|state| {
                        state.memo = Some(memo).filter(|s| !s.is_empty());
                    });
            }
            "Quit" => break,
            _ => unreachable!(),
        }
    }

    moree_state_manager::save_package_states(state_file_path, package_states)?;

    Ok(())
}
