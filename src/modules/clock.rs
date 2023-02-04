//! Clock status bar module.

use crate::{Block, Module};
use async_trait::async_trait;
use chrono::Local;
use serde::Deserialize;
use std::time::Duration;
use tokio::time::{interval, Interval, MissedTickBehavior};

/// Clock module config.
#[derive(Deserialize)]
#[serde(default)]
pub struct Config {
    /// The date and time format to display.
    pub format: String,
    /// The short date and time format to display.
    pub short_format: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            format: "%Y-%m-%d %H:%M".into(),
            short_format: "%H:%M".into(),
        }
    }
}

/// Clock status bar module.
pub struct Clock {
    interval: Interval,
    format: String,
    short_format: String,
}

impl Clock {
    /// Creates a new clock module with the provided config.
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self {
            interval: {
                let mut interval = interval(Duration::from_secs(1));
                interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
                interval
            },
            format: config.format,
            short_format: config.short_format,
        }
    }
}

#[async_trait]
impl Module for Clock {
    async fn next_block(&mut self) -> anyhow::Result<Option<Block>> {
        self.interval.tick().await;

        let now = Local::now();

        Ok(Some(Block {
            text: format!("{}", now.format(&self.format)),
            short_text: Some(format!("{}", now.format(&self.short_format))),
            color: None,
        }))
    }
}
