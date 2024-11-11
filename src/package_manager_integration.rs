use colored::*;
use std::collections::HashSet;
use std::io;
use std::process::Command;

pub fn get_installed_packages_hashset() -> io::Result<HashSet<String>> {
    let output = Command::new("paru").arg("-Qe").output()?;

    let installed_packages: HashSet<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.split_whitespace().next().map(String::from))
        .collect();

    Ok(installed_packages)
}

pub fn get_installed_packages_vec() -> io::Result<Vec<String>> {
    let output = Command::new("paru").arg("-Qe").output()?;

    let installed_packages: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.split_whitespace().next().map(String::from))
        .collect();

    Ok(installed_packages)
}

pub fn display_package_details(package_name: &str) -> io::Result<()> {
    let output = Command::new("paru").arg("-Qi").arg(package_name).output()?;

    if !output.status.success() {
        eprintln!(
            "Error getting package details for {}: {}",
            package_name,
            String::from_utf8_lossy(&output.stderr)
        ); // Print stderr for debugging
        return Err(io::Error::new(io::ErrorKind::Other, "paru -Qi failed"));
    }

    println!("{}", "\nPackage Details:".bold().yellow());
    println!("{}", String::from_utf8_lossy(&output.stdout));

    Ok(())
}

pub fn install_packages(packages: &[String]) -> io::Result<()> {
    let output = Command::new("paru").arg("-S").args(packages).status()?;

    if !output.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "paru -S failed"));
    }
    Ok(())
}

pub fn install_packages_as_deps(packages: &[String]) -> io::Result<()> {
    let output = Command::new("paru")
        .arg("-S")
        .arg("--asdeps")
        .args(packages)
        .status()?;

    if !output.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "paru -S --asdeps failed",
        ));
    }
    Ok(())
}

pub fn remove_packages(packages: &[String]) -> io::Result<()> {
    let output = Command::new("paru").arg("-R").args(packages).status()?;

    if !output.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "paru -R failed"));
    }

    Ok(())
}
