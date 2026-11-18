# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-da-734ab8.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files:

Re-read the last Result and re-checked every open row against the current

## Result

Final pass. All 11 tracker rows were re-verified against the current checkout in
`crates/pumpkin-core/src/data/datapack/` and decompiled vanilla under
`.vendor/vanilla-decompiled/net/minecraft/`. No source edit was needed this pass:
every row was already terminal from earlier passes. Per-row results:

- mod.rs:2079,1384 (zip datapacks listed but never loaded): BLOCKED-DATA. `scan_datapacks_dir`
  and `load_pack_contents` are `Path`-only and `crates/pumpkin-core/Cargo.toml` has no zip
  dependency. Vanilla loads zips through `FilePackResources` using `ZipFile`
  (`.vendor/vanilla-decompiled/net/minecraft/server/packs/FilePackResources.java:54-89`) and
  `PackDetector.detectPackResources` calls `createZipPack` for `.zip`
  (`.vendor/vanilla-decompiled/net/minecraft/server/packs/repository/PackDetector.java:41-50`).
  The exact missing pieces are a zip reader dependency and a read-only pack filesystem
  abstraction threaded through every structure/worldgen loader, i.e. a whole missing
  subsystem, not a row-level fix.

- mod.rs:122 (maxCommandChainLength ignored): VERIFIED. `chain_budget`
  (`mod.rs:1538-1545`) reads `game_rules.max_command_sequence_length` with `Math.max(1, ...)`,
  matching vanilla `Commands#executeCommandInContext`
  (`.vendor/vanilla-decompiled/net/minecraft/commands/Commands.java:458-463`).

- mod.rs:2409-2421 (pack.mcmeta description stored as raw JSON): VERIFIED. `read_pack_mcmeta`
  decodes a component object into `pumpkin_util::text::TextComponent` and keeps `get_text()`
  (`mod.rs:2846-2858`), so `/datapack list` shows plain text.

- mod.rs:1459-1483,2423 (hard-coded pack_format 61): VERIFIED. All defaults use
  `CURRENT_MC_VERSION.data_pack_format()` (`mod.rs:1721,1728,1748,2925`).

- mod.rs:1729 (Java Double/Float.toString for macro strings): VERIFIED as outdated premise.
  Vanilla `MacroFunction` uses `new DecimalFormat("#", ...)` with
  `setMaximumFractionDigits(15)`
  (`.vendor/vanilla-decompiled/net/minecraft/commands/functions/MacroFunction.java:31-33,84-94`),
  not `Double.toString`, so `1.0` stringifies to `1`. `stringify_macro_argument`
  (`mod.rs:2002-2021`) mirrors this, including the `-0` case; the round trip is locked by
  `stringify_macro_matches_vanilla_decimal_format`.

- mod.rs:1130 (function suggestions in HashMap order): VERIFIED. `get_function_names` calls
  `names.sort_unstable()` (`mod.rs:1349`).

- mod.rs:2000-2003,1352+ ("bundle" feature pack missing): VERIFIED as non-existent in this
  version. Vanilla `FeatureFlags` registers only `vanilla`, `trade_rebalance`,
  `redstone_experiments` and `minecart_improvements`
  (`.vendor/vanilla-decompiled/net/minecraft/world/flag/FeatureFlags.java:32-41`); bundles are
  ordinary vanilla items, so there is nothing to add to the bundled lists.

- mod.rs:1806 (u8 saturating custom damage ids): FIXED. `next_custom_id` is `u16`, narrowed with
  `u8::try_from`, and an exhausted id space skips the entry with `tracing::error!`
  (`mod.rs:2129,2146-2149`) instead of colliding at 255. Widening `DamageType.id` itself would
  need regenerated `pumpkin-data` (generated, forbidden to edit).

- mod.rs:500,884 (tag values return raw refs): VERIFIED. `get_tag_values` expands nested
  `#tag` members through `collect_generic_tag_values` (`mod.rs:686-711,2273`) and
  `get_tag_entries` delegates to it (`mod.rs:1060-1064`).

- function_loader.rs:30, mod.rs:2465 (symlinked pack content skipped): FIXED.
  `load_functions_recursive` and `load_tags_recursive` use `path.is_dir()` / `path.is_file()`
  (`function_loader.rs:28,30,156,158`), matching vanilla `PackDetector.detectPackResources`
  which resolves symlinks and tests the target
  (`.vendor/vanilla-decompiled/net/minecraft/server/packs/repository/PackDetector.java:31-49`).
  Tests `load_functions_follows_symlinks_and_loads_regular_files` and
  `load_function_tags_follows_symlink_directories` are present.

- mod.rs:2409 (incompatible pack_format accepted): VERIFIED. `enable_pack_checked` compares the
  declared range with `CURRENT_MC_VERSION.data_pack_format()` via `pack_format_compatibility`
  and returns `DatapackError::Incompatible` with a warning (`mod.rs:168-179,1820-1833`),
  mirroring `PackCompatibility.forVersion`.

No source files changed this pass, so nothing to rustfmt. Shell/cargo/git not run; the
orchestrator compiles the batch. No open rows remain.
