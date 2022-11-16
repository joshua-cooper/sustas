//! A status bar block.

use std::{
    pin::Pin,
    task::{Context, Poll},
};

/// A status bar block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    /// The full text to display.
    pub text: String,
    /// The text to display when the bar is shortened.
    pub short_text: Option<String>,
    /// The color to display the text with.
    pub color: Option<String>,
}

impl From<String> for Block {
    fn from(text: String) -> Self {
        Self {
            text,
            short_text: None,
            color: None,
        }
    }
}

impl From<&str> for Block {
    fn from(text: &str) -> Self {
        text.to_owned().into()
    }
}

/// A type erased stream of blocks.
pub struct Stream {
    id: usize,
    stream: Pin<Box<dyn futures_util::Stream<Item = Option<Block>>>>,
}

impl Stream {
    /// Constructs a new instance of [`Stream`].
    pub fn new<S>(id: usize, stream: S) -> Self
    where
        S: futures_util::Stream<Item = Option<Block>> + 'static,
    {
        Self {
            id,
            stream: Box::pin(stream),
        }
    }
}

impl futures_util::Stream for Stream {
    type Item = (usize, Option<Block>);

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.stream)
            .poll_next(cx)
            .map(|option| option.map(|block| (self.id, block)))
    }
}
