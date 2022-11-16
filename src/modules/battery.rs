//! Battery status bar module.

use crate::block::Block;
use async_stream::stream;
use futures_util::Stream;
use serde::Deserialize;
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::{
    fs,
    time::{Interval, MissedTickBehavior},
};

/// Battery status bar module.
#[derive(Deserialize)]
#[serde(default)]
pub struct Battery {
    /// The name of the battery.
    ///
    /// This is the name found in `/sys/class/power_supply`, which should look like `BATX` where
    /// `X` is an integer.
    pub name: String,
}

impl Battery {
    /// Returns a stream of block updates.
    pub fn stream(self) -> impl Stream<Item = Option<Block>> {
        stream! {
            let mut state = State {
                interval: {
                    let mut interval = tokio::time::interval(Duration::from_secs(10));
                    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
                    interval
                },
                capacity_path: Path::new("/sys/class/power_supply").join(&self.name).join("capacity"),
                status_path: Path::new("/sys/class/power_supply").join(&self.name).join("status"),
            };

            loop {
                yield state.next().await;
            }
        }
    }
}

impl Default for Battery {
    fn default() -> Self {
        Self {
            name: "BAT0".into(),
        }
    }
}

struct State {
    interval: Interval,
    capacity_path: PathBuf,
    status_path: PathBuf,
}

impl State {
    async fn next(&mut self) -> Option<Block> {
        self.interval.tick().await;

        let capacity = fs::read_to_string(&self.capacity_path)
            .await
            .unwrap()
            .trim()
            .parse::<u8>()
            .unwrap();

        let status = fs::read_to_string(&self.status_path).await.unwrap();

        let is_charging = status.trim() == "Charging";

        let icon = match (is_charging, capacity) {
            (true, _) => "",
            (false, 0..=25) => "",
            (false, 26..=50) => "",
            (false, 51..=75) => "",
            _ => "",
        };

        let color = if is_charging {
            Some("#00ff00".into())
        } else {
            (capacity <= 15).then_some("#ff0000".into())
        };

        Some(Block {
            text: format!("{icon} {capacity}%"),
            short_text: Some(format!("{icon} {capacity}%")),
            color,
        })
    }
}
