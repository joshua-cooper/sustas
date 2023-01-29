//! Wi-Fi status bar module.

use self::interfaces::{DeviceProxy, NetworkProxy, StationProxy};
use crate::{
    block::Block,
    dbus::{get_object_path, option_change},
};
use async_stream::stream;
use futures_util::{Stream, StreamExt};
use serde::Deserialize;
use zbus::{
    fdo::ObjectManagerProxy,
    zvariant::{ObjectPath, OwnedObjectPath},
    Connection, PropertyChanged, PropertyStream,
};

/// Wi-Fi module config.
#[derive(Deserialize)]
pub struct Config {
    /// The name of the interface to watch, for example "wlan0".
    pub interface: String,
}

impl Config {
    /// Returns a stream of block updates.
    ///
    /// # Panics
    ///
    /// TODO
    pub fn stream(self) -> impl Stream<Item = Option<Block>> {
        stream! {
            let connection = Connection::system().await.unwrap();
            let mut wifi = Wifi::new(&connection, self).await.unwrap();
            loop {
                yield wifi.block();
                wifi.wait_for_update().await;
            }
        }
    }
}

/// Wi-Fi status bar module.
pub struct Wifi<'a> {
    connection: &'a Connection,
    device_path: ObjectPath<'a>,
    powered_changes: PropertyStream<'a, bool>,
    connected_network_changes: Option<PropertyStream<'a, OwnedObjectPath>>,
    connected_network_name_changes: Option<PropertyStream<'a, String>>,
    state: State,
}

impl<'a> Wifi<'a> {
    /// Creates a new instance of [`Wifi`].
    ///
    /// # Errors
    ///
    /// TODO
    ///
    /// # Panics
    ///
    /// TODO
    pub async fn new(connection: &'a Connection, config: Config) -> zbus::Result<Wifi<'a>> {
        let object_manager = ObjectManagerProxy::builder(connection)
            .destination("net.connman.iwd")
            .expect("valid bus name")
            .path("/")
            .expect("valid path")
            .build()
            .await?;

        let device_path = get_object_path(
            &object_manager.get_managed_objects().await?,
            "net.connman.iwd.Device",
            "Name",
            config.interface.as_str(),
        )
        .unwrap()
        .to_owned();

        let device = DeviceProxy::builder(connection)
            .path(&device_path)
            .expect("valid path")
            .build()
            .await?;

        Ok(Self {
            connection,
            device_path,
            powered_changes: device.receive_powered_changed().await,
            connected_network_changes: None,
            connected_network_name_changes: None,
            state: State::PoweredOff,
        })
    }

    /// Returns a stream of block updates.
    pub fn stream(mut self) -> impl Stream<Item = Option<Block>> + 'a {
        stream! {
            loop {
                yield self.block();
                self.wait_for_update().await;
            }
        }
    }

    async fn wait_for_update(&mut self) {
        tokio::select! {
            Some(change) = self.powered_changes.next() => {
                self.handle_powered_change(change).await;
            },

            Some(Some(change)) = option_change(self.connected_network_changes.as_mut()) => {
                self.handle_connected_network_change(change).await;
            },

            Some(Some(change)) = option_change(self.connected_network_name_changes.as_mut()) => {
                self.handle_connected_network_name_change(change).await;
            },
        }
    }

    fn block(&self) -> Option<Block> {
        match &self.state {
            State::PoweredOff => None,
            State::Disconnected => Some(Block {
                text: "".into(),
                short_text: Some("".into()),
                color: Some("#888888".into()),
            }),
            State::Connected => Some(Block {
                text: "".into(),
                short_text: Some("".into()),
                color: None,
            }),
            State::ConnectedTo(network_name) => Some(Block {
                text: format!(" {network_name}"),
                short_text: Some("".into()),
                color: None,
            }),
        }
    }

    async fn handle_powered_change(&mut self, change: PropertyChanged<'a, bool>) {
        if change.get().await.unwrap_or_default() {
            self.connected_network_changes = Some(
                StationProxy::builder(self.connection)
                    .path(&self.device_path)
                    .expect("valid path")
                    .build()
                    .await
                    .unwrap()
                    .receive_connected_network_changed()
                    .await,
            );

            self.state = State::Disconnected;
        } else {
            self.state = State::PoweredOff;
            self.connected_network_changes = None;
            self.connected_network_name_changes = None;
        }
    }

    async fn handle_connected_network_change(
        &mut self,
        change: PropertyChanged<'a, OwnedObjectPath>,
    ) {
        if let Ok(connected_network) = change.get().await {
            self.connected_network_name_changes = Some(
                NetworkProxy::builder(self.connection)
                    .path(connected_network)
                    .expect("valid path")
                    .build()
                    .await
                    .unwrap()
                    .receive_name_changed()
                    .await,
            );
            self.state = State::Connected;
        } else {
            self.state = State::Disconnected;
            self.connected_network_name_changes = None;
        }
    }

    async fn handle_connected_network_name_change(&mut self, change: PropertyChanged<'a, String>) {
        if let Ok(connected_network_name) = change.get().await {
            self.state = State::ConnectedTo(connected_network_name);
        } else {
            self.state = State::Connected;
        }
    }
}

#[derive(Debug)]
enum State {
    PoweredOff,
    Disconnected,
    Connected,
    ConnectedTo(String),
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
