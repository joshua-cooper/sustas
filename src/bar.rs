//! Status bar.

use crate::{
    block::{Block, Stream},
    config::Config,
    format::Format,
    modules::Module,
};
use futures_util::{
    stream::{self, SelectAll},
    StreamExt,
};

/// Status bar.
pub struct Bar {
    format: Format,
    cache: Vec<Option<Block>>,
    updates: SelectAll<Stream>,
}

impl Bar {
    /// Creates a new instance of [`Bar`].
    #[must_use]
    pub fn new(format: Format, modules: Vec<Module>) -> Self {
        let cache = vec![None; modules.len()];

        let updates = modules
            .into_iter()
            .enumerate()
            .map(|(position, module)| match module {
                #[cfg(feature = "clock")]
                Module::Clock(module) => Stream::new(position, module.stream()),
                #[cfg(feature = "battery")]
                Module::Battery(module) => Stream::new(position, module.stream()),
                #[cfg(feature = "battery")]
                Module::Bluetooth(module) => Stream::new(position, module.stream()),
                #[cfg(feature = "battery")]
                Module::BluetoothDevice(module) => Stream::new(position, module.stream()),
                #[cfg(feature = "wifi")]
                Module::Wifi(module) => Stream::new(position, module.stream()),
            });

        let updates = stream::select_all(updates);

        Self {
            format,
            cache,
            updates,
        }
    }

    /// Runs the status bar, producing updates each time there's a change.
    #[allow(clippy::future_not_send)]
    pub async fn run(&mut self) {
        self.format.init();

        while let Some((id, block)) = self.updates.next().await {
            if let Some(cached_block) = self.cache.get_mut(id) {
                if *cached_block != block {
                    *cached_block = block;
                    self.format.update(&self.cache);
                }
            }
        }
    }
}

impl From<Config> for Bar {
    fn from(config: Config) -> Self {
        Self::new(config.format.into(), config.modules)
    }
}
