//! sustas - A fast and simple tool to generate status bars.

#![forbid(unsafe_code)]
#![warn(
    missing_docs,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]
#![allow(clippy::multiple_crate_versions)]

use sustas::config::Config;
use tokio::fs::create_dir_all;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("cannot find config directory"))?
        .join("sustas");

    create_dir_all(&config_dir).await?;

    let config_path = config_dir.join("config.toml");

    let config = std::fs::read_to_string(config_path)?;
    let config = toml::from_str::<Config>(&config)?;

    sustas::run(config).await
}
