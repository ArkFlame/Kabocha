# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-data-datap.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- remaining datapack rows: VERIFIED (note body in this worktree is unreadable; rows not reviewed in this wave; prior fix 722e94305)
Verified remaining rows against decompiled vanilla at
- zip packs (scan_datapacks_dir): BLOCKED-DATA. Needs a zip reader crate in
Sub-gap VERIFIED obsolete: DatapackInfo carries no incompatible marker as

## Result
Terminal results (FINAL PASS). Every STILL OPEN row in this unit is now closed; none remain STILL OPEN.

| Row from this unit | Terminal status | Evidence |
|---|---|---|
| remaining datapack rows (generic catch-all) | VERIFIED | All rows of the source table in `docs/tracker/taudit-crates-pumpkin-core-src-data-datap.md` are terminal in its authoritative final Result: 8 FIXED, 1 VERIFIED obsolete, 2 BLOCKED-DATA whole subsystem. Spot-checked in source: `PackFormatRange`/`pack_format_compatibility` (mod.rs:152-221), `requested_features` union (mod.rs:2900-2914), enabled-order `scan_datapacks_dir` (mod.rs:2455-2516), gamerule budget (mod.rs:328-344). Vanilla refs: PackCompatibility.forVersion (`.vendor/vanilla-decompiled/net/minecraft/server/packs/repository/PackCompatibility.java:9-33`), FeatureFlagsMetadataSection (`.vendor/vanilla-decompiled/net/minecraft/server/packs/FeatureFlagsMetadataSection.java:9-13`). |
| zip packs (`scan_datapacks_dir`) | BLOCKED-DATA (whole subsystem) | Directory-only loader: `enabled_world_pack_dirs` accepts only `is_dir()` (mod.rs:2533). `get_all_known_packs` lists `.zip` files (mod.rs:1610-1613), so an enabled zip is listed but never loaded. Vanilla loads zips through `FilePackResources` backed by `ZipFile` (`.vendor/vanilla-decompiled/net/minecraft/server/packs/FilePackResources.java:25-62,108-130`). `crates/pumpkin-core/Cargo.toml` has no `zip` dependency and Pumpkin has no `PackResources` abstraction, so this needs a new zip-backed resource subsystem wired across every loader, not a generated-data field. |
| `DatapackInfo` incompatible marker (`displayPackList`) | VERIFIED obsolete | Vanilla `/datapack list` formats packs via `Pack.getChatLink`, not `PackRepository.displayPackList` (`.vendor/vanilla-decompiled/net/minecraft/server/commands/DataPackCommand.java:191-233`; `displayPackList` only at PackRepository.java:33-34). Pumpkin does the same with plain names (`send_enabled_list`/`send_available_list`/`format_pack`, `crates/pumpkin-core/src/command/commands/datapack.rs:66-76,154-198`), and incompatible packs are already refused at enable (`DatapackError::Incompatible` -> `ERROR_ENABLE_FAILED`, datapack.rs:58-63; mod.rs:1825-1827). Adding the field also requires WIT changes (`DatapackInfo` is exported via pumpkin-plugin-api and both wasm hosts). No behaviour gap, no change. |

No source files were changed this pass: every row was already FIXED/VERIFIED in the shared tree or is a legitimate BLOCKED-DATA whole-subsystem case. No cargo/git run (shell denied to this worker); no rustfmt needed as no source was edited.
