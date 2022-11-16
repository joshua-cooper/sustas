//! Bluetooth status bar module.

use crate::{block::Block, dbus};
use async_stream::stream;
use futures_util::{Stream, StreamExt};
use serde::Deserialize;
use zbus::{fdo::ObjectManagerProxy, zvariant::ObjectPath, Connection, PropertyStream};

/// Bluetooth status bar module.
#[derive(Deserialize)]
pub struct Bluetooth {
    /// The address of the bluetooth adapter.
    pub address: String,
}

impl Bluetooth {
    /// Returns a stream of block updates.
    ///
    /// # Panics
    ///
    /// TODO
    pub fn stream(self) -> impl Stream<Item = Option<Block>> {
        stream! {
            let connection = Connection::system().await.unwrap();

            let object_manager = ObjectManagerProxy::builder(&connection)
                .destination("org.bluez")
                .unwrap()
                .path("/")
                .unwrap()
                .build()
                .await
                .unwrap();

            let path = dbus::find_path(
                &object_manager,
                "org.bluez.Adapter1",
                "Address",
                self.address.as_str(),
            )
            .await
            .unwrap()
            .unwrap();

            let mut adapter = Adapter::connect(&connection, path.as_ref()).await.unwrap();

            loop {
                yield adapter.block();
                adapter.wait_for_change().await.unwrap();
            }
        }
    }
}

/// Bluetooth device status bar module.
#[derive(Deserialize)]
pub struct BluetoothDevice {
    /// The address of the bluetooth device.
    pub address: String,
}

impl BluetoothDevice {
    /// Returns a stream of block updates.
    ///
    /// # Panics
    ///
    /// TODO
    pub fn stream(self) -> impl Stream<Item = Option<Block>> {
        stream! {
            let connection = Connection::system().await.unwrap();

            let object_manager = ObjectManagerProxy::builder(&connection)
                .destination("org.bluez")
                .unwrap()
                .path("/")
                .unwrap()
                .build()
                .await
                .unwrap();

            let path = dbus::find_path(
                &object_manager,
                "org.bluez.Device1",
                "Address",
                self.address.as_str(),
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

struct Adapter<'a> {
    powered: bool,
    powered_changes: PropertyStream<'a, bool>,
}

impl<'a> Adapter<'a> {
    async fn connect(
        connection: &'a Connection,
        path: ObjectPath<'a>,
    ) -> zbus::Result<Adapter<'a>> {
        let adapter = interfaces::AdapterProxy::builder(connection)
            .path(&path)?
            .build()
            .await?;

        Ok(Self {
            powered: adapter.powered().await?,
            powered_changes: adapter.receive_powered_changed().await,
        })
    }

    fn block(&self) -> Option<Block> {
        self.powered.then_some(Block {
            text: "".into(),
            short_text: Some("".into()),
            color: None,
        })
    }

    async fn wait_for_change(&mut self) -> zbus::Result<()> {
        if let Some(change) = self.powered_changes.next().await {
            self.powered = change.get().await?;
        }

        Ok(())
    }
}

struct Device<'a> {
    alias: String,
    alias_changes: PropertyStream<'a, String>,
    connected: bool,
    connected_changes: PropertyStream<'a, bool>,
    icon: String,
    icon_changes: PropertyStream<'a, String>,
    battery_percentage: Option<u8>,
    battery_percentage_changes: PropertyStream<'a, u8>,
}

impl<'a> Device<'a> {
    async fn connect(connection: &'a Connection, path: ObjectPath<'a>) -> zbus::Result<Device<'a>> {
        let device = interfaces::DeviceProxy::builder(connection)
            .path(&path)?
            .build()
            .await?;

        let battery = interfaces::BatteryProxy::builder(connection)
            .path(&path)?
            .build()
            .await?;

        Ok(Self {
            alias: device.alias().await?,
            alias_changes: device.receive_alias_changed().await,
            connected: device.connected().await?,
            connected_changes: device.receive_connected_changed().await,
            icon: device.icon().await?,
            icon_changes: device.receive_icon_changed().await,
            battery_percentage: battery.percentage().await.ok(),
            battery_percentage_changes: battery.receive_percentage_changed().await,
        })
    }

    fn block(&self) -> Option<Block> {
        if !self.connected {
            return None;
        }

        let icon = match self.icon.as_ref() {
            "audio-card" | "audio-headset" => "",
            "input-gaming" => "",
            "input-keyboard" => "",
            "input-mouse" => "",
            _ => "",
        };

        let text = self
            .battery_percentage
            .map_or_else(|| icon.into(), |percentage| format!("{icon} {percentage}"));

        Some(Block {
            text: text.clone(),
            short_text: Some(text),
            color: None,
        })
    }

    async fn wait_for_change(&mut self) -> zbus::Result<()> {
        tokio::select! {
            Some(change) = self.alias_changes.next() => {
                self.alias = change.get().await?;
            }

            Some(change) = self.connected_changes.next() => {
                self.connected = change.get().await?;
            }

            Some(change) = self.icon_changes.next() => {
                self.icon = change.get().await?;
            }

            Some(change) = self.battery_percentage_changes.next() => {
                self.battery_percentage = change.get().await.ok();
            }
        }

        Ok(())
    }
}

mod interfaces {
    #[zbus::dbus_proxy(interface = "org.bluez.Adapter1", default_service = "org.bluez")]
    trait Adapter {
        #[dbus_proxy(property)]
        fn powered(&self) -> zbus::Result<bool>;
    }

    #[zbus::dbus_proxy(interface = "org.bluez.Device1", default_service = "org.bluez")]
    trait Device {
        #[dbus_proxy(property)]
        fn connected(&self) -> zbus::Result<bool>;

        #[dbus_proxy(property)]
        fn alias(&self) -> zbus::Result<String>;

        #[dbus_proxy(property)]
        fn icon(&self) -> zbus::Result<String>;
    }

    #[zbus::dbus_proxy(interface = "org.bluez.Battery1", default_service = "org.bluez")]
    trait Battery {
        #[dbus_proxy(property)]
        fn percentage(&self) -> zbus::Result<u8>;
    }
}
