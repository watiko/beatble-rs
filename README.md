# beatble

Disclaimer: This project is not offical. And there is no support. Therefore, use this application at your own risk.

Use your arcade style controller through Bluetooth Low Energy (BLE) for beatmania IIDX ULTIMATE MOBILE.

## Prerequisites

- cargo: https://rustup.rs/
- (optional) cargo-deb: `cargo install cargo-deb`
- `libdbus-1-dev`
- `libudev-dev`

```bash
$ sudo apt install libdbus-1-dev libudev-dev
```

## Build

```bash
$ cargo build --release
# target/release/beatble
```

```bash
$ cargo deb
# target/debian/beatble_0.1.0_armhf.deb
```

## Install

```bash
$ sudo apt install ./beatble_0.1.0_armhf.deb
```

## Links

- https://github.com/watiko/beatble
