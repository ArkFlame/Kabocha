<div align="center">

# Kabocha

[![CI](https://github.com/ArkFlame/Kabocha/actions/workflows/rust.yml/badge.svg)](https://github.com/ArkFlame/Kabocha/actions/workflows/rust.yml)
[![License: GPL](https://img.shields.io/badge/License-GPLv3-yellow.svg)](https://opensource.org/licenses/gpl-3-0)

[Website](https://kabocha.arkflame.com) (coming soon, not published yet) • [Methodology](docs/METHODOLOGY.md) • [Process](docs/PROCESS.md) • [Tracker](TRACKER.md) • [Upstream Pumpkin](https://github.com/Pumpkin-MC/Pumpkin)

</div>

**Kabocha** is a fork of [Pumpkin](https://github.com/Pumpkin-MC/Pumpkin), the Minecraft server built entirely in Rust. Pumpkin is extremely fast and
high performance, but it does not have 100% vanilla parity yet. Kabocha works through what is left: every upstream issue, every upstream PR and every
parity gap, driven by AI agents (Claude as orchestrator, DeepSeek as implementer). It tracks upstream hourly and stays ahead of it.

- **Goal:** close all open upstream issues and PRs, then reach full vanilla parity, keeping the build green and the code clean.
- **Status** (as of 2026-11-19):
  - Upstream: all 782 upstream issues and PRs are processed: 516 DONE and 266 SKIPPED, none pending ([docs/LEDGER.md](docs/LEDGER.md), [docs/DONE.md](docs/DONE.md)).
  - Parity: final verdicts over 1369 tracker ids: 51 FIXED, 826 ALREADY-FIXED, 194 WONTFIX, 298 BLOCKED. The blocked ids are grouped by named reason in [docs/TODO.md](docs/TODO.md).
  - Tracking: [TRACKER.md](TRACKER.md) (parity gaps) and [docs/GAPS_FIXED.md](docs/GAPS_FIXED.md) (what was fixed).
  - An audit note closing a tracker row means a vanilla review was recorded, not that every detail is fixed.
- **How it is built:** [docs/METHODOLOGY.md](docs/METHODOLOGY.md) explains the approach and the cost; [docs/PROCESS.md](docs/PROCESS.md) is the runbook.
- **Upstream:** no pull requests are opened against Pumpkin yet. The fork's `master` is kept current with `Pumpkin-MC/Pumpkin` master.

<div align="center">

![Pumpkin Chunk Loading](./assets/pumpkin-chunk-loading.webp)

</div>

> [!IMPORTANT]
> Kabocha is under heavy development, like the Pumpkin project it builds on.
>
> The checklist below is the upstream Pumpkin feature list. [Upstream release plan](https://github.com/Pumpkin-MC/Pumpkin/issues/449)
>
> [See what needs to be done on Kabocha's tracker](TRACKER.md)

## Quick Start

Kabocha has no prebuilt releases yet. Build it from source with a stable Rust toolchain:

```sh
git clone https://github.com/ArkFlame/Kabocha.git
cd Kabocha
cargo run --release
```

The upstream installer and downloads at pumpkinmc.org install the original Pumpkin, not Kabocha. Release binaries and a download page will come with [kabocha.arkflame.com](https://kabocha.arkflame.com) once it is published.

Prebuilt Linux binaries require glibc. Alpine Linux uses musl, so the binaries do not run on a standard Alpine installation. On an Alpine host with Docker, use the [official Pumpkin container image](https://docs.pumpkinmc.org/#docker), which includes a glibc-based userspace.

## Goals

- **Performance**: Leveraging multi-threading for maximum speed and efficiency.
- **Compatibility**: Supports the latest Java & Bedrock Minecraft server version while adhering to vanilla game mechanics.
- **Security**: Prioritizes security by preventing known security exploits.
- **Flexibility**: Highly configurable, with the ability to disable unnecessary features.
- **Extensibility**: Provides a foundation for plugin development.
- **Multi-version**: Built-in multi-version support, so Java clients from 1.8 up to the current version can join.

## What Kabocha adds

- **Full vanilla parity**: behaviour is checked against decompiled vanilla and the upstream Pumpkin tracker (see [docs/PARITY.tsv](docs/PARITY.tsv)).
- **Minecraft 1.8 combat**: optional `legacy_combat` game rule per world (default from `pvp.legacy_combat` in the config): no attack cooldown, no sweeping, sword blocking. See [docs/COMBAT_1_8.md](docs/COMBAT_1_8.md).
- **Fixed connectivity errors**: join, protocol and disconnect issues from upstream Pumpkin are fixed.
- **Built-in multi-version support**: older Java clients (down to 1.8) connect without plugins; tracker in [docs/VERSION_SUPPORT.tsv](docs/VERSION_SUPPORT.tsv).

## Plugin development

- [WASM plugin API v0.1 to v0.2 migration guide](docs/PLUGIN-API-V0_2-MIGRATION.md)

## Features

- [x] Configuration (toml)
- [Tracking: Protocol](https://github.com/Pumpkin-MC/Pumpkin/issues/1401)
  - [x] Server Status/Ping
  - [x] Encryption
  - [x] Packet Compression
  - [x] Java Edition
  - [x] Bedrock Edition (W.I.P)
  - ...
- [Tracking: World](https://github.com/Pumpkin-MC/Pumpkin/issues/1403)
  - [x] Player Tab-list
  - [x] Scoreboard
  - [x] World Loading
  - [x] World Time
  - [x] World Borders
  - [x] World Saving
  - [x] Lighting
  - [x] Entity Spawning
  - [x] Bossbar
  - [x] Chunk Loading (Vanilla, Linear, Pump)
  - [Chunk Generation](https://github.com/Pumpkin-MC/Pumpkin/issues/36)
  - [x] Chunk Saving (Vanilla, Linear, Pump)
  - [Redstone](https://github.com/Pumpkin-MC/Pumpkin/issues/1402)
  - [x] Liquid Physics
  - ...
- [Tracking: Player](https://github.com/Pumpkin-MC/Pumpkin/issues/1405)
  - [x] Skins
  - [x] Teleport
  - [x] Movement
  - [x] Animation
  - [x] Inventory
  - [Combat](https://github.com/Pumpkin-MC/Pumpkin/issues/1404)
  - [x] Experience
  - [x] Hunger
  - [X] Off Hand
  - [X] Advancements (W.I.P)
  - [x] Eating
  - ...
- Entities
  - [x] Non-Living (Minecart, Eggs...) (W.I.P)
  - [x] Entity Effects
  - [x] Players
  - [x] Mobs (W.I.P)
  - [x] Animals (W.I.P)
  - [Entity AI](https://github.com/Pumpkin-MC/Pumpkin/issues/1406)
  - [x] Boss (W.I.P)
  - [x] Villagers (W.I.P)
  - [X] Entity Saving
- Server
  - [Plugins](https://github.com/Pumpkin-MC/Pumpkin/issues/1407)
  - [x] Query
  - [x] [Minecraft Server Management Protocol (MSMP)](https://minecraft.wiki/w/Minecraft_Server_Management_Protocol)
  - [x] Inventories
  - [x] Particles
  - [x] Chat
  - [Commands](https://github.com/Pumpkin-MC/Pumpkin/issues/15)
  - [x] Permissions
  - [x] Translations
- Proxy
  - [x] [BungeeCord](https://github.com/SpigotMC/BungeeCord)
  - [x] [BungeeGuard](https://github.com/lucko/BungeeGuard)
  - [x] [Velocity](https://github.com/PaperMC/Velocity)

<!-- Check out our [Github Project](https://github.com/orgs/Pumpkin-MC/projects/3) to see current progress. -->

## Contributions

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md)

## Communication

Consider joining [our Discord server](https://discord.gg/wT8XjrjKkf) to stay up-to-date on events, updates, and connect with other members.

## Funding

If you want to fund me and help the project, check out the [Donation Page](https://pumpkinmc.org/donate/).

## License & Attribution

* **Pumpkin Server**: Licensed under the [GNU General Public License v3.0 (GPLv3)](LICENSE).
* **Plugin API (`pumpkin-plugin-api` & `pumpkin-plugin-wit`)**: Dual-licensed under [MIT](crates/pumpkin-plugin-api/LICENSE-MIT) OR [Apache-2.0](crates/pumpkin-plugin-api/LICENSE-APACHE) for maximum flexibility when writing plugins.
* **Third-Party Assets & Data**: Bedrock mappings, protocol conversion data, and Minecraft assets are subject to their respective licenses and attribution terms. See [assets/NOTICE.md](assets/NOTICE.md) for full details.
