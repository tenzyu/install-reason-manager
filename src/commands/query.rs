use crate::moree_state_manager::PackageState;
use colored::*;
use std::collections::HashMap;
use std::io;

pub fn run(
    package_states: &HashMap<String, PackageState>,
    information: bool,
    explicit: bool,
    deps: bool,
) -> io::Result<()> {
    if explicit && deps {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Error: '--deps' and '--explicit' may not be used together",
        ));
    }

    let mut packages: Vec<_> = package_states
        .iter()
        .filter(|(_, state)| {
            if explicit {
                state.explicit
            } else if deps {
                !state.explicit
            } else {
                true // No filter if neither --explicit nor --deps is specified
            }
        })
        .map(|(name, _)| name)
        .cloned()
        .collect();

    packages.sort();

    if information {
        for package_name in &packages {
            println!(
                "{}",
                format!("Name            : {}", package_name).bold().blue()
            );
            if let Some(state) = package_states.get(package_name) {
                println!(
                    "{}",
                    format!(
                        "Install Reason  : {}",
                        if state.explicit {
                            "Explicitly installed"
                        } else {
                            "Dependency"
                        }
                    )
                    .bold()
                    .yellow()
                );
                println!(
                    "{}",
                    format!(
                        "Memo            : {}",
                        state.memo.as_ref().map_or("None", |s| s.as_str())
                    )
                    .bold()
                    .green()
                );
            } else {
                eprintln!("Package {} not found in state", package_name); // Handle unexpected case
            }
            println!(); // Add a newline for separation
        }
    } else {
        let output = packages.join("\n");
        println!("{}", output);
    }

    Ok(())
}
