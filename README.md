[![GitHub actions](https://github.com/Lukas1818/mstickereditor/workflows/Rust/badge.svg)](https://github.com/Lukas1818/mstickereditor/actions?query=workflow%3ARust)
[![crates.io](https://img.shields.io/crates/v/mstickereditor.svg)](https://crates.io/crates/mstickereditor)
[![License Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
[![Matrix chat](https://img.shields.io/badge/chat-matrix-informational?logo=matrix)](https://matrix.to/#/#mstickereditor:lukas1818.de)
[![AUR package](https://repology.org/badge/version-for-repo/aur/mstickereditor.svg)](https://aur.archlinux.org/packages/mstickereditor/)

# mstickereditor
Import sticker packs from telegram, to be used at the [Maunium sticker picker](https://github.com/maunium/stickerpicker) for Matrix

## Features:
* import Stickerpacks from Telegram (including animated Stickerspacks)
* enable Stickerpicker widget for supported Matrix Clients

## Client Support for Animated Sticker:
| Client                   | gif         | webp(default)     | stickerpicker type | 
|--------------------------|-------------|-------------------|-------------------|
| [Cinny]                  | ✅          | ✅                | ponies            |
| [Element] Desktop/Web    | ✅          | ✅                | widget            |
| [Element] Android        | no autoplay | static image only | widget            |
| [FluffyChat]             | ✅          | ✅                | ponies            |
| [Nheko]                  | ✅          | ✅                | ponies            |
| [Schildi] Desktop/Web    | ✅          | ✅                | widget            |
| [Schildi] Android        | ✅          | static image only | widget            |
| [mautrix-telegram-bridge]| ✅          | static image only | -                 |

Other clients were not tested.
(I am not assioted with Schildi, although they also love turtles)

Gif does not support semitransparent pixel, which probably leads to ugly effects,
if the background of the client does not match the `transparent_color`.

[Cinny]: https://cinny.in/
[Element]: https://element.io/download
[FluffyChat]: https://fluffychat.im/
[Nheko]: https://github.com/Nheko-Reborn/nheko
[Schildi]: https://schildi.chat/
[mautrix-telegram-bridge]: https://github.com/mautrix/telegram



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
You need to create the following `config.toml` file (located at *~/.config/mstickereditor/config.toml*) and enter your values:
```toml
[telegram]
bot_key = "YOUR-TELEGRAM-BOT-KEY"

[matrix]
user = "@user:matrix.org"
homeserver_url = "https://matrix-client.matrix.org"
access_token = "YOUR-MATIRX-ACESSTOKEN"

[sticker]
transparent_color = { r = 0, g = 0, b = 0, alpha = true }
animation_format = "webp"
```
The `[sticker]` section is optional and can be left out.

`transparent_color` is used as color for semitransparent pixel in `gif`s.
The field has no effect, if the sticker is not animated or will be converted to `webp` (default).
`r`,`g`,`b` must been between 0 and 255 inclusive. 

`animation_format`: is used to convert the animated stickers to, you can either choose `webp` (default) or `gif`.

## Installation:
Current are no prebuild binaries available. You must build mstickereditor by yourself. See below.

For Arch Linux user or user of an Arch based distrubution an [aur package](https://aur.archlinux.org/packages/mstickereditor) is available.

Nix user can use the NUR package [nur.repos.linyinfeng.mstickereditor](https://github.com/nix-community/nur-combined/tree/master/repos/linyinfeng/pkgs/mstickereditor/default.nix).

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
