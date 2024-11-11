use crate::moree_state_manager::PackageState;
use crate::package_manager_integration;
use std::collections::HashMap;
use std::io;

pub fn run(package_states: &HashMap<String, PackageState>) -> io::Result<()> {
    let installed_packages = package_manager_integration::get_installed_packages_hashset()?;
    let mut unmanaged_packages: Vec<_> = installed_packages
        .iter()
        .filter(|package| !package_states.contains_key(*package))
        .cloned()
        .collect();

    unmanaged_packages.sort();

    let output = unmanaged_packages.join("\n");
    println!("{}", output);

    Ok(())
}
