//! Battery status bar module.

use crate::{Block, Module};
use anyhow::Context;
use async_trait::async_trait;
use serde::Deserialize;
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::{
    fs::read_to_string,
    time::{interval, Interval, MissedTickBehavior},
};

/// Battery module config.
#[derive(Deserialize)]
pub struct Config {
    /// The name of the battery.
    ///
    /// This is usually in the format "BATX", where "X" is an integer.
    pub name: String,
}

/// Battery status bar module.
pub struct Battery {
    interval: Interval,
    capacity_path: PathBuf,
    status_path: PathBuf,
}

impl Battery {
    /// Creates a new battery module with the provided config.
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self {
            interval: {
                let mut interval = interval(Duration::from_secs(5));
                interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
                interval
            },
            capacity_path: Path::new("/sys/class/power_supply")
                .join(&config.name)
                .join("capacity"),
            status_path: Path::new("/sys/class/power_supply")
                .join(config.name)
                .join("status"),
        }
    }

    async fn get_capacity(&self) -> anyhow::Result<u8> {
        let capacity = read_to_string(&self.capacity_path)
            .await
            .with_context(|| {
                format!(
                    "failed to read battery capacity from {}",
                    self.capacity_path.display()
                )
            })?
            .trim()
            .parse::<u8>()
            .with_context(|| "failed to parse battery capacity")?;

        Ok(capacity)
    }

    async fn get_status(&self) -> anyhow::Result<Status> {
        let status = read_to_string(&self.status_path).await.with_context(|| {
            format!(
                "failed to read battery status from {}",
                self.status_path.display()
            )
        })?;

        match status.trim() {
            "Charging" => Ok(Status::Charging),
            _ => Ok(Status::Discharging),
        }
    }
}

#[async_trait]
impl Module for Battery {
    async fn next_block(&mut self) -> anyhow::Result<Option<Block>> {
        self.interval.tick().await;

        let (status, capacity) = tokio::join!(self.get_status(), self.get_capacity());
        let status = status?;
        let capacity = capacity?;

        if status.is_charging() {
            return Ok(Some(Block {
                text: format!(" {capacity}%"),
                short_text: None,
                color: Some("#00ff00".into()),
            }));
        }

        let (icon, color) = match capacity {
            0..=25 => ("", Some("#ff0000".into())),
            26..=50 => ("", None),
            51..=75 => ("", None),
            _ => ("", None),
        };

        Ok(Some(Block {
            text: format!("{icon} {capacity}%"),
            short_text: None,
            color,
        }))
    }
}

#[derive(Clone, Copy)]
enum Status {
    Charging,
    Discharging,
}

impl Status {
    const fn is_charging(self) -> bool {
        matches!(self, Self::Charging)
    }
}
