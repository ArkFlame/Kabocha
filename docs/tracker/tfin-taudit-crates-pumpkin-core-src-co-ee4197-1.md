# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-co-ee4197.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: datapack.rs mod.rs

- 5. `pumpkin-unit-test` exposed (`data/datapack/mod.rs:1399`): VERIFIED FIXED (S; see Result).
- 6. Suggestion ids are Pumpkin-internal (`datapack.rs:73-122`): VERIFIED FIXED (S; see Result).
needed: every in-scope STILL OPEN row is already resolved in the checkout and matches vanilla.
- 5. `pumpkin-unit-test` exposed: VERIFIED FIXED (see Result; fix lives in `data/datapack/mod.rs`).

## Result

Both in-scope rows were already resolved in the checkout and were re-verified against the decompiled vanilla source under `.vendor/vanilla-decompiled/net/minecraft/`. No further source edits were needed.

- 5. `pumpkin-unit-test` exposed: VERIFIED FIXED. `DatapackManager::is_hidden_test_pack` in `crates/pumpkin-core/src/data/datapack/mod.rs` filters the compile-time test pack out of both `get_all_known_packs` and `compute_enabled_packs` in release builds, so `/datapack list enabled` only shows real repository packs. Vanilla `listEnabledPacks` lists only `PackRepository.getSelectedPacks()` (`.vendor/vanilla-decompiled/net/minecraft/server/commands/DataPackCommand.java:218-234`). Debug builds keep the pack visible for gametests, which is the dev/test mode the row allows. Covered by `enabled_pack_list_filters_hidden_test_pack_and_disabled_packs` and `only_vanilla_pack_is_required`.

- 6. Suggestion ids are Pumpkin-internal: VERIFIED FIXED. Both providers in `crates/pumpkin-core/src/command/commands/datapack.rs` suggest each pack id once and quote only ids containing a space or `/`, matching vanilla `SELECTED_PACKS`/`UNSELECTED_PACKS`, which map ids through `StringArgumentType::escapeIfRequired` (`.vendor/vanilla-decompiled/net/minecraft/server/commands/DataPackCommand.java:80-96`). The old internal `file/<name>` duplicates and double-spellings are gone; `file/<name>` is itself vanilla-correct for world packs (`FolderRepositorySource.createDiscoveredFilePackInfo`, `.vendor/vanilla-decompiled/net/minecraft/server/packs/repository/FolderRepositorySource.java:64-67`). No pure unit-test seam exists because the providers require a `CommandContext`; existing pure seams cover the rest of the file's logic.
