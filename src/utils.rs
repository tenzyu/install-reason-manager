use dialoguer::{theme::ColorfulTheme, Confirm};
use std::io;

pub fn confirm_prompt(message: &str, default: bool) -> io::Result<bool> {
    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(message)
        .default(default)
        .interact()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
}
