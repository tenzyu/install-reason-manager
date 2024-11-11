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

pub fn get_installed_packages_asdeps_hashset() -> io::Result<HashSet<String>> {
    let output = Command::new("paru").arg("-Qdq").output()?; // Use pacman -Qeq to get explicitly installed packages

    if !output.status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "paru -Qdq failed"));
    }

    let packages: HashSet<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(String::from)
        .collect();

    Ok(packages)
}

pub fn display_package_details(package_name: &str) -> io::Result<()> {
    let output = Command::new("paru").arg("-Qi").arg(package_name).output()?;

    if let Some(code) = output.status.code() {
        // Correct: Removed extra parentheses
        if code != 0 {
            // Only print an error if the command wasn't successful
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!(
                "Error getting package details for {} (exit code {}): {}",
                package_name, code, stderr
            );
            return Err(io::Error::new(io::ErrorKind::Other, "paru -Qi failed"));
        }
    }
    // If output.status.code() is None, or it's 0, continue as normal

    println!("{}", "\nPackage Details:".bold().yellow());
    println!("{}", String::from_utf8_lossy(&output.stdout));

    Ok(())
}

pub fn install_packages(packages: &[String]) -> io::Result<()> {
    run_paru_command("-S", packages)
}

pub fn install_packages_as_deps(packages: &[String]) -> io::Result<()> {
    run_paru_command("--asdeps", packages)
}

pub fn remove_packages(packages: &[String]) -> io::Result<()> {
    run_paru_command("-R", packages)
}

fn run_paru_command(flag: &str, packages: &[String]) -> io::Result<()> {
    let output = Command::new("paru").arg(flag).args(packages).status()?;

    if !output.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "paru command failed"));
    }
    Ok(())
}
