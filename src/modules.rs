//! A collection of status bar modules.

#[cfg(feature = "battery")]
pub mod battery;
#[cfg(feature = "bluetooth")]
pub mod bluetooth;
#[cfg(feature = "clock")]
pub mod clock;
#[cfg(feature = "wifi")]
pub mod wifi;

use serde::Deserialize;

/// A status bar module.
#[derive(Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum Module {
    /// Clock module.
    #[cfg(feature = "clock")]
    Clock(self::clock::Clock),
    /// Battery module.
    #[cfg(feature = "battery")]
    Battery(self::battery::Battery),
    /// Bluetooth module.
    #[cfg(feature = "bluetooth")]
    Bluetooth(self::bluetooth::Bluetooth),
    /// Bluetooth device module.
    #[cfg(feature = "bluetooth")]
    BluetoothDevice(self::bluetooth::BluetoothDevice),
    /// Wi-Fi module.
    #[cfg(feature = "wifi")]
    Wifi(self::wifi::Config),
}
