use std::process::Command;

/// Checks rpm-ostree status for pending updates.
pub fn check_reboot_needed() -> bool {
    let cmd = "rpm-ostree status --pending-exit-77";
    let rc = Command::new("sh")
        .args(["-c", cmd])
        .status()
        .expect("Failed to execute command");
    rc.code() == Some(77)
}

/// Reboots the system.
pub fn reboot_system() -> bool {
    let cmd = "systemctl reboot";
    let rc = Command::new("sh")
        .args(["-c", cmd])
        .status()
        .expect("Failed to reboot");
    rc.code() == Some(0)
}
