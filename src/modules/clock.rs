//! Clock status bar module.

use crate::block::Block;
use async_stream::stream;
use futures_util::Stream;
use serde::Deserialize;
use std::time::Duration;
use time::{
    format_description::{self, FormatItem},
    OffsetDateTime, UtcOffset,
};
use tokio::time::{Interval, MissedTickBehavior};

/// Clock status bar module.
#[derive(Deserialize)]
#[serde(default)]
pub struct Clock {
    /// The UTC offset of the clock.
    pub offset: UtcOffset,
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
                offset: self.offset,
                format: format_description::parse(&self.format).unwrap(),
                short_format: format_description::parse(&self.short_format).unwrap(),
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
            offset: UtcOffset::UTC,
            format: "[year]-[month]-[day] [hour]:[minute]".into(),
            short_format: "[hour]:[minute]".into(),
        }
    }
}

struct State<'a> {
    interval: Interval,
    offset: UtcOffset,
    format: Vec<FormatItem<'a>>,
    short_format: Vec<FormatItem<'a>>,
}

impl<'a> State<'a> {
    async fn next(&mut self) -> Option<Block> {
        self.interval.tick().await;

        let now = OffsetDateTime::now_utc().replace_offset(self.offset);

        let text = now.format(&self.format).unwrap();

        let short_text = now.format(&self.short_format).unwrap();

        Some(Block {
            text,
            short_text: Some(short_text),
            color: None,
        })
    }
}
