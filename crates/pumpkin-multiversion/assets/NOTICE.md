# Third-Party Assets & Attribution Notice

`pumpkin-multiversion` is distributed under **GPL-3.0** as part of the Kabocha
workspace (see the `license` field in the workspace `Cargo.toml`). The
original source and the bundled data carry the separate notices below.

## Original plugin

* **Project**: `pumpkin-java-multiversion` by Pumpkin-MC
  (<https://github.com/Pumpkin-MC/pumpkin-java-multiversion>).
* **License**: MIT OR Apache-2.0 (dual-licensed).
* The Rust port in this crate is derived from that plugin. It is relicensed
  under the Kabocha workspace license (GPL-3.0); the original copyright and
  license are acknowledged here.

## Protocol translation data (ViaVersion / ViaBackwards / ViaRewind / ViaVersion-Mappings)

* **Files**: `assets/viabackwards/`, `assets/viarewind/`.
* **Copyright**: © ViaVersion contributors.
* **License**: GPL-3.0.
* **Credits and source projects**:
  * ViaVersion — <https://github.com/ViaVersion/ViaVersion>
  * ViaBackwards — <https://github.com/ViaVersion/ViaBackwards>
  * ViaRewind — <https://github.com/ViaVersion/ViaRewind>
  * ViaVersion/Mappings — <https://github.com/ViaVersion/Mappings>
* These mapping tables are used to translate packets, ids and block/item data
  for older clients. They remain under their upstream GPL-3.0 license.

## Minecraft Data

* **Files**: `assets/tracked_data/`, `assets/meta_data_type/`.
* **Copyright**: © Mojang Studios / Microsoft Corporation.
* **Terms**: These files are extracted or derived from Minecraft client and server releases. They are provided solely for compatibility, server emulation, and interoperability under the terms of the [Minecraft End User License Agreement (EULA)](https://www.minecraft.net/en-us/eula) and [Mojang Brand and Assets Guidelines](https://www.minecraft.net/en-us/usage-guidelines).
* **Note**: Vanilla data is not distributed in this repository. `build.rs` downloads the server jars from Mojang's official release servers and writes what it needs into the git-ignored `assets/datapacks/` at build time: the jar's data files for 1.19.4 and newer, and the reports of the jar's own data generator for the registries built into the game before 1.19.4. These files are **not** licensed under this plugin's license and remain the intellectual property of Mojang Studios.
