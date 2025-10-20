use renovatio::{Plugin, PluginProgress, execute};

// Implementation of distrobox
pub struct Distrobox;

impl Plugin for Distrobox {
    fn name(&self) -> &str {
        "distrobox"
    }

    fn description(&self) -> &str {
        "Update distrobox containers."
    }

    fn version(&self) -> &str {
        // TODO: implement versioning
        "0.1.0"
    }

    /// Uupd conflicts with all other plugins
    fn conflicts(&self, _plugin_name: &str) -> bool {
        true
    }

    /// Run uupd
    extern "Rust" fn update(&self, tx: flume::Sender<PluginProgress>) -> bool {
        let mut pgrss = PluginProgress {
            name: self.name().to_string(),
            progress: 0,
            status: "".to_string(),
            stdout: None,
            stderr: None,
        };

        // List the distroboxes
        let distroboxes = list();

        // calculate the total progress based on the number of outdated formulae and casks
        let total_progress = distroboxes.len();

        // Figure out how much each step should progress
        let step_progress = 100 / total_progress;

        for distrobox in distroboxes {
            pgrss.status = format!("Upgrading distrobox {}...", distrobox);
            pgrss.stdout = None;
            pgrss.stderr = None;
            let _ = tx.send(pgrss.clone());

            let (stdout, stderr, success) = upgrade(&distrobox);
            if success != 0 {
                pgrss.status = format!("Failed to upgrade distrobox {}", distrobox);
                pgrss.stderr = Some(stderr.clone());
                let _ = tx.send(pgrss.clone());
                // Continue updating
            }

            pgrss.progress = step_progress as u32;
            pgrss.stdout = Some(stdout.clone());
            if !stderr.is_empty() {
                pgrss.stderr = Some(stderr.clone());
            }
            let _ = tx.send(pgrss.clone());
        }

        // Done!
        pgrss.progress = 100;
        pgrss.stdout = None;
        pgrss.stderr = None;

        pgrss.status = "Upgrade completed!".to_string();
        let _ = tx.send(pgrss.clone());

        true
    }
}

// Export a function to create an instance of the plugin
#[unsafe(no_mangle)]
pub fn create_plugin() -> *mut dyn Plugin {
    Box::into_raw(Box::new(Distrobox))
}

pub fn list() -> Vec<String> {
    // todo: run `distrobox list --no-color` and parse the output
    let (stdout, stderr, success) = execute("distrobox list --no-color");
    println!("stdout: {}", stdout);
    vec![
        "fedora42".to_string(),
        "trixie".to_string(),
        "ubuntu-24.04".to_string(),
    ]
}

pub fn upgrade(name: &str) -> (String, String, i32) {
    execute(format!("distrobox upgrade {}", name).as_str())
}
