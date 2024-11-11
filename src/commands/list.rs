use crate::moree_state_manager::PackageState;
use std::collections::HashMap;
use std::io;

pub fn run(package_states: &HashMap<String, PackageState>) -> io::Result<()> {
    let mut packages: Vec<_> = package_states.iter().collect();
    packages.sort_by_key(|(name, _)| *name);

    for (name, state) in packages {
        let install_reason = if state.explicit {
            "Explicitly installed"
        } else {
            "Installed as a dependency for another package"
        };
        println!("Name            : {}", name);
        println!("Install Reason  : {}", install_reason);
        println!(
            "Memo            : {}",
            state.memo.as_ref().map_or("None", |s| s.as_str())
        );
        println!();
    }

    Ok(())
}
