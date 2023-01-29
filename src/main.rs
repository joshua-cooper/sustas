//! sustas - generate status bars.

#![forbid(unsafe_code)]
#![warn(
    missing_docs,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]
#![allow(clippy::multiple_crate_versions)]

use std::error::Error;
use sustas::{bar::Bar, config::Config};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let config_dir = dirs::config_dir()
        .ok_or("cannot find config directory")?
        .join("sustas");

    std::fs::create_dir_all(&config_dir)?;

    let config_path = config_dir.join("config.toml");

    let config = std::fs::read_to_string(config_path)?;
    let config = toml::from_str::<Config>(&config)?;

    let mut bar = Bar::from(config);
    bar.run().await;

    Ok(())
}
