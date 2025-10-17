use renovatio::{Plugin, Progress};

use serde::{Deserialize, Serialize};

use std::io::BufRead;
use std::io::BufReader;
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct UupdProgress {
    pub level: String,
    pub msg: String,

    #[serde(default)]
    pub title: String,

    #[serde(default)]
    pub description: String,

    #[serde(default)]
    pub previous_overall: u32,

    #[serde(default)]
    pub progress: u32,

    #[serde(default)]
    pub total: u32,

    #[serde(default)]
    pub step_progress: u32,

    #[serde(default)]
    pub overall: u32,
}

// Implementation of uupd
pub struct Uupd;

impl Plugin for Uupd {
    fn name(&self) -> &str {
        "uupd"
    }

    fn description(&self) -> &str {
        "uupd updates the bootc, rpm-ostree, flatpak, brew, and distrobox."
    }

    /// Uupd conflicts with all other plugins
    fn conflicts(&self, _plugin_name: &str) -> bool {
        true
    }

    /// Run uupd
    extern "Rust" fn update(&self, tx: Sender<Progress>, tick: extern "Rust" fn()) -> bool {
        // This will run uupd and output the progress in json, which we'll use serde to parse
        // the status, do some conversion to make the progress bar more accurate, and bubble
        // that information up to the status closure.
        let cmd = "pkexec uupd --json";

        if let Ok(child_process) = Command::new("sh")
            .args(["-c", cmd])
            .stdout(Stdio::piped())
            .spawn()
        {
            let incoming = child_process.stdout.unwrap();
            let mut previous_overall = 0;

            let _ = &BufReader::new(incoming).lines().for_each(|line| {
                let data = line.unwrap();
                println!("Received data: {}", data);
                let mut finished = false;

                // Unwrap the data from uupd
                let mut p: UupdProgress = serde_json::from_str(&data).unwrap();

                p.previous_overall = previous_overall;

                // Track the previous progress
                previous_overall = p.overall;

                let progress = if (p.progress + 1) < p.total {
                    p.progress + 1
                } else {
                    p.progress
                };

                let mut msg = format!(
                    "{} {} - {} (step {}/{})...",
                    p.msg,
                    p.title,
                    p.description,
                    progress,
                    p.total + 1
                );

                if p.progress == 100 || (progress == 0 && p.total == 0) {
                    finished = true;
                }

                if finished {
                    msg = "Update complete.".to_string();
                }

                // Update renovatio with our current progress
                let pgrss = Progress {
                    progress: p.previous_overall,
                    status: msg,
                };

                // Send the progress back to the main thread and update the UI
                let _ = tx.send(pgrss);
                tick();
            });
        };
        true
    }
}

// Export a function to create an instance of the plugin
#[unsafe(no_mangle)]
pub fn create_plugin() -> *mut dyn Plugin {
    Box::into_raw(Box::new(Uupd))
}
