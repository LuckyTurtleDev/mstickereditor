[![GitHub actions](https://github.com/Lukas1818/mstickereditor/workflows/Rust/badge.svg)](https://github.com/Lukas1818/mstickereditor/actions?query=workflow%3ARust)
[![crates.io](https://img.shields.io/crates/v/mstickereditor.svg)](https://crates.io/crates/mstickereditor)
[![License Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
[![AUR package](https://repology.org/badge/version-for-repo/aur/mstickereditor.svg)](https://aur.archlinux.org/packages/mstickereditor/)

# mstickereditor
Import sticker packs from telegram, to be used at the [Maunium sticker picker](https://github.com/maunium/stickerpicker) for Matrix

## Features:
* import Stickerpacks from Telegram (including animated Stickerspacks)
* enable Stickerpicker widget for supported Matrix Clients

## Requirements:
* a Stickerpickerserver [msrd0/docker-stickerpicker](https://github.com/msrd0/docker-stickerpicker) or [maunium/stickerpicker](https://github.com/maunium/stickerpicker)
* a telegram bot key

#### Dependencies:
* [libwebp](https://chromium.googlesource.com/webm/libwebp)
* [rlottie v0.2](https://github.com/Samsung/rlottie/tree/v0.2)
* [cargo](https://www.rust-lang.org) (make)
* [clang](https://lld.llvm.org/) (make)
* [ldd](https://clang.llvm.org/) (make)

### Configuration:
You need to create the following `config.toml` file and enter you values:
```toml
[telegram]
bot_key = "YOUR-TELEGRAM-BOT-KEY"

[matrix]
user = "@user:matrix.org"
homeserver_url = "https://matrix-client.matrix.org"
access_token = "YOUR-MATIRX-ACESSTOKEN"
```

## Installation:
Current are no prebuild binaries available. You must build mstickereditor by yourself. See below.

For Arch Linux user or user of an Arch based distrubution an [aur package](https://aur.archlinux.org/packages/mstickereditor) is available.

### Building:

 Install the following packages. (I recommand to use the package managment system of your operating system):
* [libwebp](https://chromium.googlesource.com/webm/libwebp)
* [rlottie v0.2](https://github.com/Samsung/rlottie/tree/v0.2)
* [rust](https://www.rust-lang.org/tools/install)
* [clang](https://lld.llvm.org/)
* [ldd](https://clang.llvm.org/)

To build and install mstickereditor execute the following command:
```bash
cargo install --locked mstickereditor
```
Check out [rust doc](https://doc.rust-lang.org/cargo/commands/cargo-install.html) for more information about `cargo install`.
