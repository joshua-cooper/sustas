//! Wi-Fi status bar module.

use crate::{block::Block, dbus};
use async_stream::stream;
use futures_util::{future::OptionFuture, Stream, StreamExt};
use serde::Deserialize;
use zbus::{
    fdo::ObjectManagerProxy,
    zvariant::{ObjectPath, OwnedObjectPath},
    Connection, PropertyStream,
};

/// Wi-Fi status bar module.
#[derive(Deserialize)]
#[serde(default)]
pub struct Wifi {
    /// The interface to watch.
    pub interface: String,
}

impl Wifi {
    /// Returns a stream of block updates.
    ///
    /// # Panics
    ///
    /// TODO
    pub fn stream(self) -> impl Stream<Item = Option<Block>> {
        stream! {
            let connection = Connection::system().await.unwrap();

            let object_manager = ObjectManagerProxy::builder(&connection)
                .destination("net.connman.iwd")
                .unwrap()
                .path("/")
                .unwrap()
                .build()
                .await
                .unwrap();

            let path = dbus::find_path(
                &object_manager,
                "net.connman.iwd.Device",
                "Name",
                self.interface.as_str(),
            )
            .await
            .unwrap()
            .unwrap();

            let mut device = Device::connect(&connection, path.as_ref()).await.unwrap();

            loop {
                yield device.block();
                device.wait_for_change().await.unwrap();
            }
        }
    }
}

impl Default for Wifi {
    fn default() -> Self {
        Self {
            interface: "wlan0".into(),
        }
    }
}

struct Network<'a> {
    name: String,
    name_changes: PropertyStream<'a, String>,
}

impl<'a> Network<'a> {
    async fn wait_for_change(&mut self) -> zbus::Result<()> {
        if let Some(change) = self.name_changes.next().await {
            self.name = change.get().await?;
        }

        Ok(())
    }
}

struct Device<'a> {
    connection: &'a Connection,
    powered: bool,
    powered_changes: PropertyStream<'a, bool>,
    connected_network: Option<Network<'a>>,
    connected_network_changes: PropertyStream<'a, OwnedObjectPath>,
}

impl<'a> Device<'a> {
    async fn connect(connection: &'a Connection, path: ObjectPath<'a>) -> zbus::Result<Device<'a>> {
        let device = interfaces::DeviceProxy::builder(connection)
            .path(&path)?
            .build()
            .await?;

        let station = interfaces::StationProxy::builder(connection)
            .path(&path)?
            .build()
            .await?;

        let connected_network = match station.connected_network().await.ok() {
            Some(path) => {
                let network = interfaces::NetworkProxy::builder(connection)
                    .path(path)?
                    .build()
                    .await?;

                Some(Network {
                    name: network.name().await?,
                    name_changes: network.receive_name_changed().await,
                })
            }
            None => None,
        };

        Ok(Self {
            connection,
            powered: device.powered().await?,
            powered_changes: device.receive_powered_changed().await,
            connected_network,
            connected_network_changes: station.receive_connected_network_changed().await,
        })
    }

    fn block(&self) -> Option<Block> {
        self.powered
            .then_some(self.connected_network.as_ref().map_or_else(
                || Block {
                    text: "".into(),
                    short_text: Some("".into()),
                    color: Some("#888888".into()),
                },
                |network| Block {
                    text: format!(" {}", network.name),
                    short_text: Some("".into()),
                    color: None,
                },
            ))
    }

    async fn wait_for_change(&mut self) -> zbus::Result<()> {
        let connected_network_change = OptionFuture::from(
            self.connected_network
                .as_mut()
                .map(Network::wait_for_change),
        );

        tokio::select! {
            Some(change) = self.powered_changes.next() => {
                self.powered = change.get().await?;
            }

            Some(change) = self.connected_network_changes.next() => {
                self.connected_network = match change.get().await.ok() {
                    Some(path) => {
                        let network = interfaces::NetworkProxy::builder(self.connection)
                            .path(path)?
                            .build()
                            .await?;

                        Some(Network {
                            name: network.name().await?,
                            name_changes: network.receive_name_changed().await,
                        })
                    },
                    None => None,
                };
            }

            Some(result) = connected_network_change => {
                result?;
            }
        }

        Ok(())
    }
}

mod interfaces {
    use zbus::zvariant::OwnedObjectPath;

    #[zbus::dbus_proxy(
        interface = "net.connman.iwd.Device",
        default_service = "net.connman.iwd"
    )]
    trait Device {
        #[dbus_proxy(property)]
        fn powered(&self) -> zbus::Result<bool>;
    }

    #[zbus::dbus_proxy(
        interface = "net.connman.iwd.Station",
        default_service = "net.connman.iwd"
    )]
    trait Station {
        #[dbus_proxy(property)]
        fn connected_network(&self) -> zbus::Result<OwnedObjectPath>;
    }

    #[zbus::dbus_proxy(
        interface = "net.connman.iwd.Network",
        default_service = "net.connman.iwd"
    )]
    trait Network {
        #[dbus_proxy(property)]
        fn name(&self) -> zbus::Result<String>;
    }
}
