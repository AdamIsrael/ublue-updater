#[derive(Clone, Debug)]
/// PluginProgress is used to communicate the progress of a plugin update
pub struct PluginProgress {
    /// The name of the plugin this status belongs to
    pub name: String,

    /// The progress of the update, from 0 to 100
    pub progress: u32,

    /// The status of the update
    pub status: String,

    /// The standard output from the update, if available
    pub stdout: Option<String>,

    /// The standard error from the update, if available
    pub stderr: Option<String>,
}

/// PluginMetadata is used to describe a plugin
#[derive(Clone, Debug)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub path: Option<String>,
}

impl PluginMetadata {
    /// Create a new plugin metadata
    pub fn new(plugin: Box<dyn Plugin>) -> Self {
        Self {
            name: plugin.name().to_string(),
            version: plugin.version().to_string(),
            description: plugin.description().to_string(),
            path: None,
        }
    }
}

/// Plugin is a trait that defines the interface for a plugin.
pub trait Plugin {
    /// The name of the plugin
    fn name(&self) -> &str;

    /// The description of the plugin
    fn description(&self) -> &str;

    /// The version of the plugin
    fn version(&self) -> &str;

    /// Determine if this plugin conflicts with another plugin.
    fn conflicts(&self, plugin_name: &str) -> bool;

    /// Run a blocking update
    ///
    /// # Arguments
    ///
    /// * `tx` - The sender channel to send progress updates to
    ///
    /// # Returns
    ///
    /// * `bool` - Whether the update was successful
    extern "Rust" fn update(&self, tx: flume::Sender<PluginProgress>) -> bool;
}
