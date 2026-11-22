use std::process::Command;

use crate::error::SteveError;

pub(crate) fn eject() -> Result<(), SteveError> {
    #[cfg(target_os = "linux")]
    let (program, args) = (
        "udisksctl",
        ["unmount", "--block-device", "/dev/disk/by-label/IPOD"],
    );
    #[cfg(not(target_os = "linux"))]
    let (program, args) = ("diskutil", ["eject", crate::IPOD_ROOT]);

    let status = Command::new(program)
        .args(args)
        .status()
        .map_err(|source| SteveError::Command {
            source,
            context: "ejecting ipod",
        })?;

    if !status.success() {
        return Err(SteveError::EjectFailed { program, status });
    }

    Ok(())
}
