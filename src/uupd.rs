use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Progress {
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
