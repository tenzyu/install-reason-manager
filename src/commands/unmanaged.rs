use crate::moree_state_manager::PackageState;
use crate::package_manager_integration;
use std::collections::HashMap;
use std::io;

pub fn run(package_states: &HashMap<String, PackageState>) -> io::Result<()> {
    let installed_packages = package_manager_integration::get_installed_packages_hashset()?;

    for package in installed_packages {
        if !package_states.contains_key(&package) {
            println!("{}", package);
        }
    }

    Ok(())
}
