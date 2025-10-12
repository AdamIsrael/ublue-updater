use inline_xml::xml;
use std::process::{Command, Stdio};

/// Installs our GSettings schema, if they're not already installed.
pub fn install_gsettings_schema() {
    let xml = xml! {
        <schemalist>
            <schema id="com.github.AdamIsrael.UblueUpdater" path="/com/github/AdamIsrael/UblueUpdater/">
            <key name="auto-reboot" type="b">
                <default>false</default>
                <summary>A flag to enable automatic reboot after update</summary>
            </key>
            </schema>
        </schemalist>
    };

    // write the xml to ~/.local/share/glib-2.0/schemas
    let glib_schemas_dir = format!(
        "{}/.local/share/glib-2.0/schemas",
        std::env::var("HOME").unwrap()
    );
    std::fs::create_dir_all(&glib_schemas_dir).unwrap();

    let xml_path = format!(
        "{}/{}",
        glib_schemas_dir, "com.github.AdamIsrael.UblueUpdater.gschema.xml"
    );

    if !std::fs::exists(&xml_path).unwrap_or(false) {
        println!("Installing schema...");
        std::fs::write(&xml_path, xml.to_string()).unwrap();

        // glib-compile-schemas
        println!("Compiling schema...");
        let _ = std::process::Command::new("glib-compile-schemas")
            .arg(&glib_schemas_dir)
            .output()
            .unwrap();
    }
}

/// Checks rpm-ostree status for pending updates.
pub fn check_reboot_needed() -> bool {
    let cmd = "rpm-ostree status --pending-exit-77";
    let rc = Command::new("sh")
        .args(["-c", cmd])
        // pipe stdout to /dev/null for now. Otherwise it goes to this app's stdout
        .stdout(Stdio::null())
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
