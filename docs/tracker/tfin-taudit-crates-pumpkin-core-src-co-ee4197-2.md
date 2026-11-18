# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-co-ee4197.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: mod.rs

- 8. Pack order model / `before vanilla`: BLOCKED-DATA. Missing subsystem: a vanilla `PackRepository`-style ordered selection model stored in `LevelData` (built-in packs included in `DataPacks.enabled`), plus a rewrite of `load_all`/`load_bundled_datapacks` in `data/datapack/mod.rs` so a world pack can precede `vanilla`. Verified against `.vendor/vanilla-decompiled/net/minecraft/server/packs/repository/PackRepository.java` and `DataPackCommand.java`.

## Result
- 8. Pack order model / `before vanilla`: BLOCKED-DATA. Missing subsystem: a vanilla `PackRepository`-style ordered selection model stored in `LevelData` (built-in packs included in `DataPacks.enabled`), plus a rewrite of `load_all`/`load_bundled_datapacks` in `data/datapack/mod.rs` so a world pack can precede `vanilla`. Verified against `.vendor/vanilla-decompiled/net/minecraft/server/packs/repository/PackRepository.java` and `DataPackCommand.java`.
