use std::sync::mpsc::Sender;

#[derive(Clone, Debug)]
pub struct Progress {
    /// The progress of the update, from 0 to 100
    pub progress: u32,
    /// The status of the update
    pub status: String,
}

/// Plugin is a trait that defines the interface for a plugin.
pub trait Plugin {
    /// The name of the plugin
    fn name(&self) -> &str;

    /// The description of the plugin
    fn description(&self) -> &str;

    /// Determine if this plugin conflicts with another plugin.
    fn conflicts(&self, plugin_name: &str) -> bool;

    /// Run an update
    // fn update(&self, f: fn(f32, &str)) -> bool;
    extern "Rust" fn update(&self, tx: Sender<Progress>, tick: extern "Rust" fn()) -> bool;
    // extern "Rust" fn update(&self, f: extern "Rust" fn(u32)) -> bool;
    // fn update(&self) -> bool;

    // extern "C" fn update(&self, f: fn(Sender<&str>)) -> bool;
}
