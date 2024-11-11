use crate::moree_state_manager::PackageState;
use crate::package_manager_integration;
use colored::*;
use std::collections::HashMap;
use std::io;

pub fn run(package_states: &HashMap<String, PackageState>, all: bool) -> io::Result<()> {
    let target_explicit_packages = package_manager_integration::get_installed_packages_hashset()?;
    let target_asdeps_packages =
        package_manager_integration::get_installed_packages_asdeps_hashset()?;

    for (package, state) in package_states {
        let source_explicit = state.explicit;

        let target_explicit = target_explicit_packages.contains(package);
        let target_asdeps = target_asdeps_packages.contains(package);

        if source_explicit && target_asdeps {
            println!("+ {} {}", package.green(), "[explicitly]");
            println!("- {} {}", package.red(), "[non-explicitly]");
        } else if !source_explicit && target_explicit {
            println!("+ {} {}", package.green(), "[non-explicitly]");
            println!("- {} {}", package.red(), "[explicitly]");
        }
    }

    if all {
        for (package, state) in package_states {
            if state.explicit
                && !(target_explicit_packages.contains(package)
                    || target_asdeps_packages.contains(package))
            {
                println!(
                    "+ {} {}",
                    package.green(),
                    "[explicitly managed, but not installed]"
                );
            }
        }

        let unmanaged: Vec<_> = target_explicit_packages
            .iter()
            .filter(|package| !package_states.contains_key(*package))
            .map(|p| format!("- {} [unmanaged]", p.red()))
            .collect();

        if !unmanaged.is_empty() {
            println!("{}", unmanaged.join("\n"));
        }

        println!("Note: moree don't print unmanaged packages installed as dependencies.",);
    }

    Ok(())
}
