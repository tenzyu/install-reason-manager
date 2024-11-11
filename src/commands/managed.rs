use crate::moree_state_manager::PackageState;
use std::collections::HashMap;
use std::io;

pub fn run(package_states: &HashMap<String, PackageState>) -> io::Result<()> {
    let mut managed_packages: Vec<_> = package_states
        .iter()
        .filter_map(|(name, state)| if state.explicit { Some(name) } else { None })
        .cloned()
        .collect();

    managed_packages.sort();

    // Build a single string for output
    let output = managed_packages.join("\n");
    println!("{}", output); // Print once

    Ok(())
}
