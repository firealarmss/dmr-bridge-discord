# dvm2discord

[![License](https://img.shields.io/badge/License-GPLv3-blue?style=for-the-badge)](https://www.gnu.org/licenses/gpl-3.0)

Bridge a DVM network with a Discord voice channel.

## Getting started

This script is inspired by <https://github.com/jess-sys/DMRBridgeWAV/blob/master/DMRBridgeWAV> and <https://github.com/jess-sys/dmr-bridge-discord>.

The target server is dvmbridge (see <https://github.com/DVMProject/dvmbridge>).

### Build

Make sure you have [Rust installed](https://rustup.rs/) and also [Opus codec library development files installed](https://packages.ubuntu.com/jammy/libopus-dev)

```bash
cargo build --release
# or run it directly :
# cargo run
```
### Configure

Edit the `.env` (the same directory or in /opt/dmr-bridge-discord) file to reflect your infrastructure :

* `BOT_TOKEN` : see [this link](https://github.com/reactiflux/discord-irc/wiki/Creating-a-discord-bot-&-getting-a-token) to know how to get a token
* `BOT_PREFIX` : prefix to add before the bot's commands
* `TARGET_RX_ADDR` : your Analog Bridge IP and port NOT USED AT THIS TIME
* `LOCAL_RX_ADDR` : your dmr-bridge-discord IP and port to receieve from dvmbridge

### Usage

Here are the bot's commands:

* `!join` : Make the bot join the channel (you need to be in a voice channel first)
* `!leave` : Make the bot left the channel

The bot will join the voice channel you're in after your type `!join`.

TX DOES NOT WORK AT THIS TIME

Make sure you don't TX and RX at the same time, as AnalogBridge and the rest of the stack is half-duplex. 

## Todo

* Discord multiple voice users at once (merge audio channels)
* Add TX support

## Useless stuff (Copyright)

Bridge a DMR network with a Discord voice channel.
Copyright (C) 2022 Jessy SOBREIRO

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, version 3.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
