//! Status bar output format.

use crate::block::Block;

/// Status bar output format.
pub enum Format {
    /// Debug output format.
    Debug,
    /// Swaybar output format.
    #[cfg(feature = "swaybar")]
    Swaybar,
}

impl Format {
    /// Initializes the status bar.
    pub fn init(&mut self) {
        match self {
            Self::Debug => {}
            #[cfg(feature = "swaybar")]
            Self::Swaybar => {
                println!(r#"{{"version":1}}"#);
                println!("[");
            }
        }
    }

    /// Updates the status bar.
    pub fn update(&mut self, blocks: &[Option<Block>]) {
        match self {
            Self::Debug => {
                println!("{blocks:?}");
            }

            #[cfg(feature = "swaybar")]
            Self::Swaybar => {
                let bar = blocks
                    .iter()
                    .filter_map(|block| {
                        let block = block.as_ref()?;

                        Some(serde_json::json!({
                            "full_text": block.text,
                            "short_text": block.short_text,
                            "color": block.color,
                        }))
                    })
                    .collect::<Vec<_>>();

                println!(
                    "{},",
                    serde_json::to_string(&bar).expect("infallible serialization")
                );
            }
        }
    }
}
