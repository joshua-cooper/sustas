//! Clock status bar module.

use crate::block::Block;
use async_stream::stream;
use chrono::Local;
use futures_util::Stream;
use serde::Deserialize;
use std::time::Duration;
use tokio::time::{Interval, MissedTickBehavior};

/// Clock status bar module.
#[derive(Deserialize)]
#[serde(default)]
pub struct Clock {
    /// The date and time format to display.
    pub format: String,
    /// The date and time format to display when the bar is shortened.
    pub short_format: String,
}

impl Clock {
    /// Returns a stream of block updates.
    pub fn stream(self) -> impl Stream<Item = Option<Block>> {
        stream! {
            let mut state = State {
                interval: {
                    let mut interval = tokio::time::interval(Duration::from_secs(1));
                    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
                    interval
                },
                format: self.format,
                short_format: self.short_format,
            };

            loop {
                yield state.next().await;
            }
        }
    }
}

impl Default for Clock {
    fn default() -> Self {
        Self {
            format: "%Y-%m-%d %H:%M:%S".into(),
            short_format: "%H:%M".into(),
        }
    }
}

struct State {
    interval: Interval,
    format: String,
    short_format: String,
}

impl State {
    async fn next(&mut self) -> Option<Block> {
        self.interval.tick().await;

        let now = Local::now();

        Some(Block {
            text: format!("{}", now.format(&self.format)),
            short_text: Some(format!("{}", now.format(&self.short_format))),
            color: None,
        })
    }
}
