//! Types for configuring a status bar.

use crate::{format, modules::Module};
use serde::Deserialize;

/// Output format of the status bar.
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Format {
    /// Debug output.
    Debug,
    /// Swaybar output.
    #[cfg(feature = "swaybar")]
    Swaybar,
}

impl From<Format> for format::Format {
    fn from(format: Format) -> Self {
        match format {
            Format::Debug => Self::Debug,
            #[cfg(feature = "swaybar")]
            Format::Swaybar => Self::Swaybar,
        }
    }
}

/// Configuration for a status bar.
#[derive(Deserialize)]
pub struct Config {
    /// Output format of the status bar.
    pub format: Format,
    /// Configuration for each status bar module.
    pub modules: Vec<Module>,
}
