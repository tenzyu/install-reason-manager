use crate::moree_state_manager::PackageState;
use std::collections::HashMap;
use std::io;

pub fn run(package_states: &HashMap<String, PackageState>) -> io::Result<()> {
    for (name, state) in package_states {
        if state.explicit {
            println!("{}", name);
        }
    }
    Ok(())
}
