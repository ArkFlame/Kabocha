# DEEP AUDIT: crates/pumpkin-core/src/data/datapack/mod.rs

Scope: datapack discovery/loading, function execution, tags, pack listing.
No literal TODO/FIXME markers exist in the file; the tracker's "vanilla markers" are
comments referencing vanilla behaviour. Behaviour gaps found are listed below.

## Fixed in this pass

- Enabled pack order was ignored. `scan_datapacks_dir` iterated `fs::read_dir` order, so
  which pack won an override was arbitrary and `/datapack enable ... before|after` had no
  effect on load order. Now packs load in level order (`enabled_world_pack_dirs`).
- `is_in_tag` did not follow nested `#tag` references (vanilla flattens tags when loading).
  Now resolved recursively with cycle protection (`tag_contains_entry`).
- Function tags containing `#other_tag` silently dropped those members. Now expanded with
  cycle protection (`collect_function_tag_members`).

## Remaining gaps

| file:line | Vanilla class | Gap | Suggested fix | Size |
|---|---|---|---|---|
| mod.rs:2079 (scan loop), 1384 | `PackRepository` / `FolderRepositorySource` / `FilePackResources` | `.zip` datapacks are listed by `get_all_known_packs` but never loaded (`scan_datapacks_dir` only accepts directories), so `/datapack enable` on a zip appears to work but loads nothing. | Introduce a small read-only pack filesystem abstraction (dir or zip) and thread it through `load_pack_contents`, `read_pack_mcmeta`, structure/worldgen loaders. | L, STILL OPEN: size L, skipped per wave brief |
| mod.rs:122 | `CommandFunctionManager` (`maxCommandChainLength` game rule) | Command budget is the fixed constant 65536; vanilla reads the `maxCommandChainLength` game rule per execution, so lowering it in game rules is ignored. | Add `maxCommandChainLength` to the gamerule set, read it from the executing world before running a chain, and reset the thread-local budget accordingly. | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| mod.rs:2409-2421 | `PackMetadataSection` / `Component.Serializer` | `pack.mcmeta` `description` that is a JSON text component is stored as raw JSON (`d.to_string()`), so `/datapack list` shows JSON instead of text. | Parse the component and keep its plain text (fall back to `translate` key), or store a `TextComponent`. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| mod.rs:1459-1483, 2423 | `DataPackVersion` / `PackFormat` | `pack_format` for the vanilla/embedded/bundled packs and the mcmeta default are the literal `61`, unrelated to `CURRENT_MC_VERSION`; a version bump silently desyncs them. | Derive the current data pack format once from `CURRENT_MC_VERSION` (const next to `CURRENT_MC_VERSION`) and use it for all defaults. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| mod.rs:1729 | `MacroFunction.stringify` (via `Tag.getAsString`) | Floats/doubles are printed with 15 fixed decimals and trailing zeros trimmed, so `1.0` becomes `1` and large values use Rust formatting. Java `Double.toString`/`Float.toString` produce `1.0` and differ at boundaries, changing macro-expanded command text. | Mirror Java `Double.toString`/`Float.toString` output (or add test fixtures from vanilla) before changing behaviour. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| mod.rs:1130 | `FunctionArgument#listSuggestions` | `get_function_names` returns `HashMap` iteration order; vanilla suggestions are alphabetical, so `/function <tab>` output is nondeterministic. | `names.sort_unstable()` before returning. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| mod.rs:2000-2003, 1352+ | `VanillaPackResources` feature packs | `"bundle"` is accepted by known-packs and feature resolution (`mod.rs:941`, `mod.rs:1000+`) but missing from `load_bundled_datapacks` and `get_all_known_packs`, so enabling the bundle experiment loads no data. | Add `"bundle"` to both bundled lists once the assets datapack exists. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| mod.rs:1806 | `RegistryDataLoader` raw ids | Custom damage type ids are `u8` and `saturating_add`, so beyond 205 custom types ids collide at 255; vanilla registry ids are ints. | Use a wider id type or reject overflow with a load error. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| mod.rs:500, 884 | `Tag#getEntries` (flattened holder set) | `get_tag_values` / `get_tag_entries` return raw `#tag` reference strings; vanilla returns the flattened member list. | Reuse the `tag_contains_entry` recursion to expand nested references. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| function_loader.rs:30, mod.rs:2465 | `FolderRepositorySource` | Loaders skip symlinked files/dirs (`entry.file_type()`), while vanilla's directory source follows them; a pack symlinking shared content silently loads nothing. | Use `path.is_dir()` / `path.is_file()` in the recursive loaders. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| mod.rs:2409 | `PackMetadataSection` format range | An incompatible `pack_format` is accepted without warning; vanilla marks the pack incompatible and refuses to enable it. | Compare `pack_format` (or min/max range) against the current format and reject/skip with a log. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |


## Result

This unit may edit only `function_loader.rs`. Every STILL OPEN gap whose fix lives in
`mod.rs` or generated data is out of scope and left open with a one-line reason below.

- function_loader.rs:30 (symlinks skipped by `entry.file_type()`): FIXED. Verified against
  vanilla `PackDetector.detectPackResources`
  (`.vendor/vanilla-decompiled/net/minecraft/server/packs/repository/PackDetector.java:31-49`),
  which resolves `Files.readSymbolicLink` and tests the target with `Files.isDirectory`, so
  symlinked packs/content are followed. Replaced `entry.file_type()` with `path.is_dir()` /
  `path.is_file()` in `load_functions_recursive` and `load_tags_recursive` (matching
  `load_block_tags_recursive`, which already used `path.is_dir()`). Updated the two tests that
  asserted the old skip behaviour to assert symlinked functions and function-tag directories
  are now loaded. Ran `rustfmt --edition 2024` on the file (exit 0). No compile/test run this
  pass: per unit instructions the orchestrator compiles once after all workers.

Out of scope (left STILL OPEN, one-line reasons):

- mod.rs:2079, 1384 (zip datapacks listed but never loaded): needs a read-only pack filesystem
  abstraction threaded through many loaders in `mod.rs`; outside the editable file.
- mod.rs:122 (`maxCommandChainLength` game rule ignored): needs gamerule plumbing in `mod.rs`;
  outside the editable file.
- mod.rs:2409-2421 (`pack.mcmeta` description stored as raw JSON): parsing fix lives in `mod.rs`;
  outside the editable file.
- mod.rs:1459-1483, 2423 (hard-coded `pack_format` 61): needs generated-data/`CURRENT_MC_VERSION`
  wiring in `mod.rs`; outside the editable file.
- mod.rs:1729 (Java `Double`/`Float.toString` for macro strings): fix lives in `mod.rs`; outside
  the editable file.
- mod.rs:1130 (function suggestions in HashMap order): fix lives in `mod.rs`; outside the file.
- mod.rs:2000-2003, 1352+ ("bundle" feature pack missing from bundled lists): needs the assets
  datapack plus `mod.rs`; outside the editable file.
- mod.rs:1806 (u8 saturating custom damage type ids): fix lives in `mod.rs`; outside the file.
- mod.rs:500, 884 (`get_tag_values`/`get_tag_entries` return raw refs): fix lives in `mod.rs`;
  outside the editable file.
- mod.rs:2409 (incompatible `pack_format` accepted): fix lives in `mod.rs`; outside the file.


## Result - second pass (cross-file)

Verified every STILL OPEN row against the current code in
`crates/pumpkin-core/src/data/datapack/mod.rs` (this shared checkout has already advanced
that file well past the line numbers in the table above).

Resolved already in `mod.rs` (no new edit needed this pass):

- mod.rs:122 (maxCommandChainLength ignored): `chain_budget()` reads the
  `max_command_sequence_length` game rule per chain (`mod.rs:1538-1544`), and
  `enter_with_budget` threads the value into the thread-local budget.
- mod.rs:2409-2421 (description stored as raw JSON): `read_pack_mcmeta` parses the
  description into `pumpkin_util::text::TextComponent` and keeps `get_text()`
  (`mod.rs:2841-2853`).
- mod.rs:1459-1483, 2423 (hard-coded pack_format 61): all defaults now use
  `CURRENT_MC_VERSION.data_pack_format()` (`mod.rs:1721,1728,1748,2854`).
- mod.rs:1729 (Java Double/Float.toString): verified against decompiled vanilla
  `MacroFunction` (`net/minecraft/commands/functions/MacroFunction.java:31-33,84-94`):
  `DECIMAL_FORMAT` is `new DecimalFormat("#")` with
  `setMaximumFractionDigits(15)`, NOT `Double.toString`. Vanilla therefore stringifies
  `1.0` to `1`, so the tracker premise was outdated. `stringify_macro_argument`
  (`mod.rs:2002-2021`) already mirrors this format and
  `stringify_macro_matches_vanilla_decimal_format` (`mod.rs:3034`) locks it in. No change.
- mod.rs:1130 (function suggestions in HashMap order): `get_function_names` calls
  `names.sort_unstable()` (`mod.rs:1349`).
- mod.rs:500,884 (tag values return raw refs): `get_tag_values`/`get_tag_entries` flatten
  nested references via `collect_generic_tag_values` (`mod.rs:700`).
- mod.rs:2409 (incompatible pack_format accepted): enabling a pack checks
  `pack_format_compatibility` and warns on a mismatch (`mod.rs:1823-1830`).

Still BLOCKED (require generated data or a whole missing subsystem):

- mod.rs:2079,1384 (zip datapacks listed but never loaded): no zip/`ZipArchive` reader
  exists; `scan_datapacks_dir` and `find_bundled_datapack_dir` are directory-only.
  Requires a new read-only pack filesystem abstraction threaded through every loader, i.e.
  a whole missing subsystem.
- mod.rs:2000-2003,1352+ ("bundle" feature pack): "bundle" is absent from both bundled
  lists (`mod.rs:1588-1592,2396-2400`) and no bundled assets datapack exists; adding the
  name alone would load nothing. Needs generated assets data plus the load path.
- mod.rs:1806 (u8 saturating custom damage ids): `pumpkin_data::damage::DamageType.id` is
  `u8` in `crates/pumpkin-data/src/generated/damage_type.rs` (generated, forbidden to
  edit); widening the id requires regenerating generated data.

## Result - third pass

Every remaining STILL OPEN row closed as FIXED, VERIFIED or BLOCKED-DATA. Files touched:
`crates/pumpkin-core/src/data/datapack/mod.rs`,
`crates/pumpkin-core/src/data/datapack/test_loader.rs`,
`crates/pumpkin-core/src/data/datapack/function_loader.rs` (second pass).

- mod.rs:2079,1384 (zip datapacks listed but never loaded): BLOCKED-DATA / whole subsystem.
  `pumpkin-core` has no `zip` dependency (`crates/pumpkin-core/Cargo.toml`), and
  `scan_datapacks_dir`, `read_pack_mcmeta`, `pack_metadata_error` and `load_pack_contents`
  are `Path`-only. Vanilla needs a `PackResources` abstraction (`FilePackResources`,
  `net/minecraft/server/packs/FilePackResources.java:54-89`) threaded through every
  structure/worldgen/advancement loader, so this is a missing subsystem, not a row fix.
- mod.rs:122 (maxCommandChainLength ignored): VERIFIED. `chain_budget`
  (`mod.rs:1538-1545`) reads `game_rules.max_command_sequence_length` with
  `Math.max(1, ...)`, matching vanilla `Commands#executeCommandInContext`
  (`.vendor/vanilla-decompiled/net/minecraft/commands/Commands.java:458-463`).
- mod.rs:2409-2421 (description stored as raw JSON): VERIFIED. `read_pack_mcmeta`
  decodes the component into `pumpkin_util::text::TextComponent` and keeps `get_text()`
  (`mod.rs:2840-2856`), so `/datapack list` shows plain text; the round trip is covered by
  `read_pack_mcmeta_flattens_component_descriptions`.
- mod.rs:1459-1483, 2423 (hard-coded pack_format 61): VERIFIED. All defaults use
  `CURRENT_MC_VERSION.data_pack_format()` (`mod.rs:1721,1728,1748,2920`).
- mod.rs:1729 (Java Double/Float.toString for macro strings): VERIFIED as tracker
  premise outdated. Vanilla `MacroFunction` uses `new DecimalFormat("#")` with
  `setMaximumFractionDigits(15)`, not `Double.toString`
  (`.vendor/vanilla-decompiled/net/minecraft/commands/functions/MacroFunction.java:31-33,84-94`),
  which is exactly what `stringify_macro_argument` (`mod.rs:2002-2021`) does.
- mod.rs:1130 (suggestions in HashMap order): VERIFIED. `get_function_names` sorts with
  `names.sort_unstable()` (`mod.rs:1349`).
- mod.rs:2000-2003,1352+ ("bundle" feature pack): VERIFIED as non-existent in this version.
  Vanilla `FeatureFlags` registers only `vanilla`, `trade_rebalance`,
  `redstone_experiments` and `minecart_improvements`
  (`.vendor/vanilla-decompiled/net/minecraft/world/flag/FeatureFlags.java:32-41`), and
  `data/Main.java:172-183` generates only those three builtin datapacks. Bundles are ordinary
  vanilla items now, so there is nothing to add to the bundled lists.
- mod.rs:1806 (u8 saturating custom damage type ids): FIXED without generated data.
  `next_custom_id` is now `u16` and the id is converted with `u8::try_from`; when the
  `u8` space is exhausted the entry is skipped with a `tracing::error!` instead of being
  given a colliding saturated id (`mod.rs:2126-2154`). Widening `DamageType.id` itself
  would need regenerated pumpkin-data, which stays out of scope.
- mod.rs:500,884 (tag values returning raw refs): VERIFIED. `get_tag_values` expands
  nested `#tag` members through `collect_generic_tag_values` (`mod.rs:686-711, 700`) and
  `get_tag_entries` delegates to it (`mod.rs:1060-1064`).
- mod.rs:2409 (incompatible pack_format accepted): VERIFIED. `enable_pack_checked` compares
  the declared range with `CURRENT_MC_VERSION.data_pack_format()` via
  `pack_format_compatibility` and returns `DatapackError::Incompatible` with a warning
  (`mod.rs:1820-1833`), mirroring `PackCompatibility.forVersion`.
- mod.rs:2937 (recipe loader skipped symlinks): FIXED. `load_recipes_recursive` now tests
  `path.is_dir()`/`path.is_file()` like the other loaders; vanilla lists packs with
  `Files.find` which follows links
  (`.vendor/vanilla-decompiled/net/minecraft/server/packs/PathPackResources.java:102-119`).
  `load_recipes_follows_symlinks_and_loads_regular_files` asserts the symlinked directory and
  file are both loaded.
- test_loader.rs:72 (test instance loader skipped symlinks): FIXED the same way, and
  `load_test_instances_follows_symlinks_and_loads_regular_files` now asserts the symlinked
  directory and file are registered.
- function_loader.rs:30, mod.rs:2465 (symlinked pack content skipped): FIXED in the earlier
  passes; no `entry.file_type()` calls remain in `crates/pumpkin-core/src/data/datapack/`.

No cargo/rustfmt run by this worker (shell denied); the orchestrator compiles the batch.


## Result - third pass final verification

Re-read the previous Result and re-verified each row against the current checkout; every
row is terminal (FIXED, VERIFIED or BLOCKED-DATA) and no new source fix was required this
pass. Files named for this unit (`function_loader.rs`, `test_loader.rs`, and the damage-type
loader referenced by the unit) were re-checked and remain correct:

- function_loader.rs:30 (symlinks skipped): FIXED. `load_functions_recursive` and
  `load_tags_recursive` use `path.is_dir()` / `path.is_file()` (function_loader.rs:28,30,156,158),
  matching vanilla `PackDetector.detectPackResources` which resolves symbolic links and tests
  the target (`PackDetector.java:31-49`). Tests `load_functions_follows_symlinks_and_loads_regular_files`
  and `load_function_tags_follows_symlink_directories` are present.
- test_loader.rs:72 (symlinks skipped): FIXED. `load_test_instances_recursive` uses
  `path.is_dir()` / `path.is_file()` (test_loader.rs:76,78), matching vanilla pack walking with
  `Files.find` (`PathPackResources.java:102-119`). Test
  `load_test_instances_follows_symlinks_and_loads_regular_files` is present.
- mod.rs:1806 (u8 saturating custom damage ids): FIXED. `next_custom_id` is `u16` and the id is
  narrowed with `u8::try_from`, skipping with `tracing::error!` on exhaustion
  (mod.rs:2129,2146-2148); widening `DamageType.id` itself needs regenerated pumpkin-data and
  stays out of scope.

Spot-checked the remaining VERIFIED claims in code and decompiled vanilla; all hold:
macro stringify uses `new DecimalFormat("#")` + `setMaximumFractionDigits(15)`
(`MacroFunction.java:31-33,84-94`, matching `stringify_macro_argument` at mod.rs:2002);
`chain_budget` reads `max_command_sequence_length` (mod.rs:1538-1543 vs `Commands.java:458-463`);
`get_function_names` sorts (mod.rs:1349); tag values flatten nested refs
(`collect_generic_tag_values` mod.rs:2273); pack format uses `CURRENT_MC_VERSION.data_pack_format()`
(mod.rs:1721,1728,1748,2925). The zip-datapack row stays BLOCKED-DATA (no zip reader / whole
`PackResources` subsystem). No STILL OPEN rows remain.

No cargo or rustfmt run by this worker (shell denied); no source files changed this pass, so
there is nothing to format. The orchestrator compiles the batch.


## Result - final re-verification (CODER second/third pass)

Re-read the last Result and re-checked every STILL OPEN row against the current
checkout. All rows are terminal; no further source edit was needed this pass.

Named files re-verified:
- function_loader.rs (symlinks): FIXED. `load_functions_recursive` and
  `load_tags_recursive` use `path.is_dir()` / `path.is_file()`, matching vanilla
  `PackDetector.detectPackResources` which resolves symbolic links and tests the
  target (`.vendor/vanilla-decompiled/net/minecraft/server/packs/repository/PackDetector.java:31-49`).
  Tests `load_functions_follows_symlinks_and_loads_regular_files` and
  `load_function_tags_follows_symlink_directories` present.
- test_loader.rs (symlinks): FIXED. `load_test_instances_recursive` uses
  `path.is_dir()` / `path.is_file()`, matching vanilla `Files.find` pack walking
  (`.vendor/vanilla-decompiled/net/minecraft/server/packs/PathPackResources.java:102-119`).
  Test `load_test_instances_follows_symlinks_and_loads_regular_files` present.
- damage_type.rs: only exists as generated `pumpkin-data` data (forbidden to edit).
  The u8 saturating custom-id row is FIXED in `mod.rs` (`next_custom_id: u16` with
  `u8::try_from`, skip + `tracing::error!` on exhaustion).

Spot-checked remaining VERIFIED rows in `mod.rs`: `chain_budget` reads
`max_command_sequence_length` (vs `Commands.java:458-463`); `read_pack_mcmeta`
flattens component descriptions to text; all format defaults use
`CURRENT_MC_VERSION.data_pack_format()`; `stringify_macro_argument` mirrors
vanilla `new DecimalFormat("#")` + `setMaximumFractionDigits(15)`
(`MacroFunction.java:31-33,84-94`); `get_function_names` sorts; tag values flatten
nested refs via `collect_generic_tag_values`; `enable_pack_checked` rejects
incompatible formats via `pack_format_compatibility`; `load_recipes_recursive`
follows symlinks. The zip-datapack row stays BLOCKED-DATA (no zip reader / whole
`PackResources` subsystem; no `zip` dependency in `pumpkin-core/Cargo.toml`).

No source files changed this pass, so nothing to rustfmt. Shell denied; the
orchestrator compiles the batch.
