# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-co-ee4197.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: datapack.rs mod.rs

- 1. `datapack.rs:455` (before/after builders): OPEN. Not re-verified this pass.
- 2. Duplicate enable/disable logic: OPEN. Two separate world-info update blocks remain; a refactor only, no parity change.
- 3. `before`/`after` anchor not enabled: FIXED (DatapackError::UnknownAnchor and NotEnabled return before world info mutates, data/datapack/mod.rs enable_pack_checked; vanilla getPack ERROR_PACK_ALREADY_* in .vendor/vanilla-decompiled/net/minecraft/server/commands/DataPackCommand.java:236-256).
- 4. `reload_datapacks` result discarded: FIXED. enable and disable now return the reload result and send commands.reload.failure on error (vanilla ReloadCommand.reloadPacks, .vendor/vanilla-decompiled/net/minecraft/server/commands/ReloadCommand.java:18-24).

## Result
Results: rows 3 and 4 FIXED (see rows above). Rows 1 and 2 remain OPEN as stated.
