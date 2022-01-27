[![GitHub actions](https://github.com/Lukas1818/mstickereditor/workflows/Rust/badge.svg)](https://github.com/Lukas1818/mstickereditor/actions?query=workflow%3ARust)
[![License Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
[![AUR package](https://repology.org/badge/version-for-repo/aur/mstickereditor.svg)](https://aur.archlinux.org/packages/mstickereditor/)

# mstickereditor
Import sticker packs from telegram, to be used at the Maunium sticker picker for Matrix


### Requirements:
* a Stickerpickerserver [msrd0/docker-stickerpicker](https://github.com/msrd0/docker-stickerpicker) or [maunium/stickerpicker](https://github.com/maunium/stickerpicker)
* a telegram bot key

### Dependencies:
* libwebp
* rlottie
* cargo (make)
* clang (make)
* ldd (make)


### Config file:
you need to create the following config file and enter you values:
```toml
[telegram]
bot_key = "YOUR-TELEGRAM-BOT-KEY"

[matrix]
user = "@user:matrix.org"
homeserver_url = "https://matrix-client.matrix.org"
access_token = "YOUR-MATIRX-ACESSTOKEN"
```
