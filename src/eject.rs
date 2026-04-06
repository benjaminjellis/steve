use std::process::Command;

use crate::{IPOD_ROOT, error::SteveError, ui};

pub(crate) fn eject() -> Result<(), SteveError> {
    let eject_command = Command::new("diskutil")
        .args(["eject", IPOD_ROOT])
        .status()
        .map_err(|source| SteveError::Command {
            source,
            context: "ejecting ipod",
        })?;

    if !eject_command.success() {
        match eject_command.code() {
            Some(code) => ui::red_std_err(format!("diskutil eject failed with exit code {code}")),
            None => ui::red_std_err("diskutil eject terminated without an exit code".into()),
        }
    }

    Ok(())
}
