# Sustas

A tool to generate desktop status lines.

## Supported formats

- swaybar

## Configuration

The config file lives at `$XDG_CONFIG_HOME/sustas/config.toml`.

The status line is configured via modules, for example:

```toml
format = "swaybar"

[[modules]]
kind = "wifi"
interface = "wlan0"

[[modules]]
kind = "bluetooth"
address = "1A:12:75:7D:E8:5D"

[[modules]]
kind = "bluetooth_device"
address = "1F:BA:15:9A:81:B1"

[[modules]]
kind = "battery"
name = "BAT0"

[[modules]]
kind = "clock"
```
