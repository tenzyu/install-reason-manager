use crate::moree_state_manager::PackageState;
use crate::package_manager_integration;
use std::collections::HashMap;
use std::io;

pub fn run(
    package_states: &HashMap<String, PackageState>,
    with_install: bool,
    with_uninstall: bool,
    sync: bool,
) -> io::Result<()> {
    if sync || (with_install && with_uninstall) {
        apply_sync(package_states)
    } else {
        apply(package_states, with_install, with_uninstall)
    }
}

fn apply(
    package_states: &HashMap<String, PackageState>,
    with_install: bool,
    with_uninstall: bool,
) -> io::Result<()> {
    let installed_packages = package_manager_integration::get_installed_packages_hashset()?;
    let mut to_install = Vec::new();
    let mut to_install_asdeps = Vec::new();

    for (package, state) in package_states {
        if state.explicit {
            if with_install && !installed_packages.contains(package) {
                to_install.push(package.clone())
            }
        } else {
            if installed_packages.contains(package) {
                to_install_asdeps.push(package.clone())
            }
        }
    }

    if !to_install.is_empty() {
        package_manager_integration::install_packages(&to_install)?;
    }

    if !to_install_asdeps.is_empty() {
        package_manager_integration::install_packages_as_deps(&to_install_asdeps)?;
    }

    let mut to_remove = Vec::new();
    if with_uninstall {
        for package in installed_packages {
            if !package_states.contains_key(&package) {
                to_remove.push(package)
            }
        }

        if !to_remove.is_empty() {
            package_manager_integration::remove_packages(&to_remove)?;
        }
    }

    Ok(())
}

fn apply_sync(package_states: &HashMap<String, PackageState>) -> io::Result<()> {
    let installed_packages = package_manager_integration::get_installed_packages_hashset()?;
    let mut to_install = Vec::new();
    let mut to_remove = Vec::new();

    for (package, state) in package_states {
        if state.explicit && !installed_packages.contains(package) {
            to_install.push(package.clone());
        }
    }

    for package in &installed_packages {
        if !package_states.contains_key(package) {
            to_remove.push(package.clone());
        } else if let Some(state) = package_states.get(package) {
            if !state.explicit {
                // Install as dependency if it exists and marked as not explicit.
                package_manager_integration::install_packages_as_deps(&[package.clone()])?;
            }
        }
    }

    if !to_install.is_empty() {
        package_manager_integration::install_packages(&to_install)?;
    }

    if !to_remove.is_empty() {
        package_manager_integration::remove_packages(&to_remove)?;
    }

    Ok(())
}
