# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-da-734ab8.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files).

Rows from the parent tracker's "Remaining gaps" table, closed in table order. The shared checkout had already resolved every row in earlier passes; this pass re-verified each against current code and decompiled vanilla and made no new source edits. No open rows remain.

## Result

1. mod.rs:2079,1384 (zip datapacks listed but never loaded): BLOCKED-DATA. Whole missing subsystem, not a row fix. `pumpkin-core/Cargo.toml` has no `zip` dependency; `scan_datapacks_dir` (mod.rs:2455), `read_pack_mcmeta`, `pack_metadata_error` and `load_pack_contents` (mod.rs:2543) are `Path`-only. Vanilla loads zip packs through `FilePackResources.SharedZipFileAccess` (`.vendor/vanilla-decompiled/net/minecraft/server/packs/FilePackResources.java:169-187`) behind a `PackResources` abstraction threaded through every loader. Exact missing subsystem: read-only zip/`PackResources` pack filesystem.

2. mod.rs:122 (maxCommandChainLength game rule ignored): VERIFIED. `chain_budget` (mod.rs:1538-1545) reads `game_rules.max_command_sequence_length` with `Math.max(1, ..)` each chain, matching vanilla `.vendor/vanilla-decompiled/net/minecraft/commands/Commands.java:463` (`int chainLimit = Math.max(1, gameRules.get(GameRules.MAX_COMMAND_SEQUENCE_LENGTH))`).

3. mod.rs:2409-2421 (pack.mcmeta description stored as raw JSON): VERIFIED. `read_pack_mcmeta` (mod.rs:2838-2858) parses a component object into `pumpkin_util::text::TextComponent` and keeps `get_text()` (plain string kept as-is), so `/datapack list` shows plain text.

4. mod.rs:1459-1483,2423 (hard-coded pack_format 61): VERIFIED. All defaults use `CURRENT_MC_VERSION.data_pack_format()` (mod.rs:1721,1728,1748,2925); no literal `61` remains for these defaults.

5. mod.rs:1729 (Java Double/Float.toString for macro strings): VERIFIED, tracker premise outdated. Vanilla `MacroFunction` uses `new DecimalFormat("#", ...)` with `setMaximumFractionDigits(15)` (`.vendor/vanilla-decompiled/net/minecraft/commands/functions/MacroFunction.java:31-32`), not `Double.toString`, so `1.0` renders as `1`. `stringify_macro_argument` (mod.rs:2002-2021) mirrors this and `stringify_macro_matches_vanilla_decimal_format` locks it in. No change.

6. mod.rs:1130 (function suggestions in HashMap order): VERIFIED. `get_function_names` calls `names.sort_unstable()` (mod.rs:1349), giving alphabetical suggestions.

7. mod.rs:2000-2003,1352+ ("bundle" feature pack missing from bundled lists): VERIFIED as non-existent in this version. Vanilla `FeatureFlags` registers only `vanilla`, `trade_rebalance`, `redstone_experiments`, `minecart_improvements` (`.vendor/vanilla-decompiled/net/minecraft/world/flag/FeatureFlags.java:32-41`); `known_feature_flag` (mod.rs:2298-2307) matches exactly that set. Bundles are ordinary items now, so there is nothing to add.

8. mod.rs:1806 (u8 saturating custom damage type ids): FIXED. `next_custom_id` is `u16` and narrowed with `u8::try_from`, skipping with `tracing::error!` on exhaustion (mod.rs:2126-2157) instead of assigning colliding saturated ids. Widening `DamageType.id` itself needs regenerated pumpkin-data (generated, forbidden).

9. mod.rs:500,884 (get_tag_values/get_tag_entries return raw refs): VERIFIED. `collect_generic_tag_values` (mod.rs:2273-2294) recursively flattens nested `#tag` members with cycle protection; `get_tag_values` (mod.rs:686-700) and `get_tag_entries` (mod.rs:1060) both use it.

10. function_loader.rs:30, mod.rs:2465 (symlinked pack content skipped): FIXED. `load_functions_recursive`/`load_tags_recursive` (function_loader.rs:28,30) use `path.is_dir()`/`path.is_file()`; same pattern in `load_test_instances_recursive` (test_loader.rs:76,78) and `load_recipes_recursive` (mod.rs:2931-2952). Vanilla follows symlinks (`.vendor/vanilla-decompiled/net/minecraft/server/packs/repository/PackDetector.java:21-41`, resolves `Files.readSymbolicLink` then tests `isDirectory`). Symlink tests present in each loader.

11. mod.rs:2409 (incompatible pack_format accepted): VERIFIED. `enable_pack_checked` (mod.rs:1806-1833) compares the declared range with `CURRENT_MC_VERSION.data_pack_format()` via `pack_format_compatibility` (mod.rs:168-179) and returns `DatapackError::Incompatible` with a warning, mirroring vanilla `PackCompatibility.forVersion`.

No cargo or rustfmt run by this worker (shell denied); no source files changed this pass, so there is nothing to format. The orchestrator compiles the batch.
