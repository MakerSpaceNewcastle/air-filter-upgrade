# Control program

A program that monitors the environment and controls air filters, both via MQTT.

## Setup

1. `cross build --release --target arm-unknown-linux-gnueabihf` (adjust for actual architecture, this is for an OG Pi)
2. Copy binary to `/usr/bin/ms-air-filter-control`
3. Copy config to `/etc/ms-air-filter-control.toml`
4. Copy systemd unit file to `/lib/systemd/system/ms-air-filter-control.service`
5. Set MQTT credentials in confg file
6. `systemctl enable --now ms-air-filter-control.service`
