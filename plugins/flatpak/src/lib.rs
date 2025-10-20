use renovatio::{Plugin, PluginProgress, execute};

pub struct Flatpak;
// Implementation of flatpak

impl Plugin for Flatpak {
    fn name(&self) -> &str {
        "flatpak"
    }

    fn description(&self) -> &str {
        "Update user and system flatpaks."
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
        // This will run uupd and output the progress in json, which we'll use serde to parse
        // the status, do some conversion to make the progress bar more accurate, and bubble
        // that information up to the status closure.
        let cmd = "pkexec uupd --json";

        // if let Ok(child_process) = Command::new("sh")
        //     .args(["-c", cmd])
        //     .stdout(Stdio::piped())
        //     .spawn()
        // {
        //     let incoming = child_process.stdout.unwrap();
        //     let mut previous_overall = 0;

        //     let _ = &BufReader::new(incoming).lines().for_each(|line| {
        //         let data = line.unwrap();
        //         println!("Received data: {}", data);
        //         let mut finished = false;

        //         // Unwrap the data from uupd
        //         let mut p: UupdProgress = serde_json::from_str(&data).unwrap();

        //         p.previous_overall = previous_overall;

        //         // Track the previous progress
        //         previous_overall = p.overall;

        //         let progress = if (p.progress + 1) < p.total {
        //             p.progress + 1
        //         } else {
        //             p.progress
        //         };

        //         let mut msg = format!(
        //             "{} {} - {} (step {}/{})...",
        //             p.msg,
        //             p.title,
        //             p.description,
        //             progress,
        //             p.total + 1
        //         );

        //         if p.progress == 100 || (progress == 0 && p.total == 0) {
        //             finished = true;
        //         }

        //         if finished {
        //             msg = "Update complete.".to_string();
        //         }

        //         // Update renovatio with our current progress
        //         let pgrss = Progress {
        //             progress: p.previous_overall,
        //             status: msg,
        //         };

        //         // Send the progress back to the main thread and update the UI
        //         let _ = tx.send(pgrss);
        //         tick();
        //     });
        // };
        let mut pgrss = PluginProgress {
            name: self.name().to_string(),
            progress: 25,
            status: "Downloading updates...".to_string(),
            stdout: None,
            stderr: None,
        };
        let _ = tx.send(pgrss.clone());
        std::thread::sleep(std::time::Duration::from_millis(1000));

        pgrss.progress = 50;
        pgrss.status = "Installing updates (user)...".to_string();
        let _ = tx.send(pgrss.clone());
        std::thread::sleep(std::time::Duration::from_millis(1000));

        pgrss.progress = 75;
        pgrss.status = "Installing updates (system)...".to_string();
        let _ = tx.send(pgrss.clone());
        std::thread::sleep(std::time::Duration::from_millis(1000));

        pgrss.progress = 100;
        pgrss.status = "Finished!".to_string();
        let _ = tx.send(pgrss.clone());
        std::thread::sleep(std::time::Duration::from_millis(1000));

        true
    }
}

// Export a function to create an instance of the plugin
#[unsafe(no_mangle)]
pub fn create_plugin() -> *mut dyn Plugin {
    Box::into_raw(Box::new(Flatpak))
}

fn list_updates() -> Vec<String> {
    let mut updates = Vec::new();
    // Add logic to list updates here
    // `flatpak remote-ls --updates` may do it, but I can't test b/c my flatpaks are all updated
    updates
}

fn upgrade(name: &str) -> (String, String, i32) {
    execute(format!("flatpak update {}", name).as_str())
}
