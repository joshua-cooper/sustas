//! Components for building a status bar.

#![forbid(unsafe_code)]
#![warn(
    missing_docs,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]
#![allow(clippy::module_name_repetitions, clippy::multiple_crate_versions)]

pub mod config;
pub mod dbus;
pub mod modules;

use anyhow::Context;
use config::Config;
use modules::{battery::Battery, clock::Clock, wifi::Wifi};
use tokio::{
    sync::mpsc::{channel, Sender},
    task::JoinHandle,
};
use zbus::Connection;

/// A status bar block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    /// The full text to display.
    pub text: String,
    /// A shorter version of text that can be displayed when there isn't enough room.
    pub short_text: Option<String>,
    /// The color to display the text with.
    pub color: Option<String>,
}

/// A status bar module that produces blocks.
#[async_trait::async_trait]
pub trait Module {
    /// Wait for the next block update.
    async fn next_block(&mut self) -> anyhow::Result<Option<Block>>;
}

struct Update {
    id: usize,
    block: Option<Block>,
}

fn spawn_module<M>(
    mut module: M,
    id: usize,
    sender: Sender<anyhow::Result<Update>>,
) -> JoinHandle<()>
where
    M: Module + Send + 'static,
{
    tokio::spawn(async move {
        loop {
            let result = module.next_block().await.map(|block| Update { id, block });

            if sender.send(result).await.is_err() {
                break;
            }
        }
    })
}

/// Run a bar with the given config.
///
/// # Errors
///
/// Returns an error if connecting to dbus fails.
pub async fn run(config: Config) -> anyhow::Result<()> {
    let dbus_connection = Connection::system()
        .await
        .with_context(|| "failed to connect to dbus")?;

    let mut cache = vec![None; config.modules.len()];
    let (sender, mut receiver) = channel(10);

    for (i, module) in config.modules.into_iter().enumerate() {
        match module {
            config::Module::Battery(config) => {
                spawn_module(Battery::new(config), i, sender.clone());
            }
            config::Module::Clock(config) => {
                spawn_module(Clock::new(config), i, sender.clone());
            }
            config::Module::Wifi(config) => {
                let wifi = Wifi::new(dbus_connection.clone(), config)
                    .await
                    .with_context(|| "failed to initialize wifi module")?;
                spawn_module(wifi, i, sender.clone());
            }
        }
    }

    println!(r#"{{"version":1}}"#);
    println!("[");

    fn print(blocks: &[Option<Block>]) {
        #[derive(serde::Serialize)]
        struct SwayBlock<'a> {
            full_text: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            short_text: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            color: Option<&'a str>,
        }

        let bar = blocks
            .iter()
            .filter_map(|block| {
                let block = block.as_ref()?;

                Some(SwayBlock {
                    full_text: &block.text,
                    short_text: block.short_text.as_deref(),
                    color: block.color.as_deref(),
                })
            })
            .collect::<Vec<_>>();

        println!(
            "{},",
            serde_json::to_string(&bar).expect("infallible serialization")
        );
    }

    while let Some(result) = receiver.recv().await {
        match result {
            Ok(Update { id, block }) => {
                if let Some(cached_block) = cache.get_mut(id) {
                    if *cached_block != block {
                        *cached_block = block;
                        print(&cache);
                    }
                }
            }

            Err(_) => unimplemented!(),
        }
    }

    Ok(())
}
