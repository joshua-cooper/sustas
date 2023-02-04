//! Sustas config.

use crate::modules::{battery, clock, wifi};
use serde::Deserialize;

/// Sustas config.
#[derive(Deserialize)]
pub struct Config {
    /// Configuration for each module in the bar.
    pub modules: Vec<Module>,
}

/// A single module's config.
#[derive(Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum Module {
    /// Battery module config.
    Battery(battery::Config),
    /// Clock module config.
    Clock(clock::Config),
    /// Wifi module config.
    Wifi(wifi::Config),
}
