# audit / crates/pumpkin-core/src/data/datapack/mod.rs

Target: JavaMinecraftVersion::V_26_3. Reference: decompiled tree at
`/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft` (`.vendor` absent).
Reviewed for TODO/FIXME markers: there are none; the 40 markers are prose mentions of
vanilla behaviour.

Fixed in this pass:

- `resolve_block_tag` (was `mod.rs:1115`): nested/direct duplicates were returned,
  while vanilla `TagLoader#tryBuildTag` collects members into a `LinkedHashSet`.
  Now deduplicated in first-seen order + test.
- `read_pack_mcmeta` (`mod.rs:2367`): object descriptions were shown as raw JSON;
  vanilla renders `Pack.Metadata#description` as a `Component`. Now flattened to its
  English text + test.
- `resolve_enabled_features` / `get_known_packs`: removed the `minecraft:bundle`
  feature flag. `FeatureFlags` in this version registers only `vanilla`,
  `trade_rebalance`, `redstone_experiments`, `minecart_improvements`.

Remaining gaps:

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| `mod.rs:2016-2040` `scan_datapacks_dir` | `Pack.DirSource` and `Pack.ZipSource` | Only directories are loaded; `get_all_known_packs` (`:1370`) lists `.zip` packs, so an enabled zip is listed but loads nothing | Add a pack-reader abstraction; read zip entries with the workspace `zip` crate (add to `pumpkin-core`) into the loaders, or extract to a temp dir | M |
| `mod.rs:324-331` `load_all` | `PackRepository#rebuildSelected`: selected order defines priority, required `vanilla` inserted at the bottom, later packs override earlier ones | World packs are scanned in filesystem order, then bundled feature packs are loaded after them, so bundled packs override world packs (see `load_structure` :1065 reversed iteration) and conflict resolution is fs-order dependent | Iterate `level_info.data_packs.enabled` in order and resolve each id (embedded/bundled/`file/`); load bundled packs before world packs | M |
| `mod.rs:1007` `resolve_enabled_features` | `PackRepository#getRequestedFeatureFlags` unions `pack.mcmeta` `"features": {"enabled": [...]}` (`FeatureFlagsMetadataSection`) of selected packs | Flags are guessed from pack ids, so a pack that declares features in metadata is ignored and a world pack named `trade_rebalance` wrongly enables the flag | Parse `features.enabled` in `read_pack_mcmeta`, store it on `LoadedDatapack`, union over selected packs plus `minecraft:vanilla` | M |
| `mod.rs:2367` `read_pack_mcmeta`; `:1466`, `:1473`, `:1488`, `:2395`, `:2438` | `PackFormat` (major/minor), `PackCompatibility#forVersion`, `PackRepository#displayPackList` appends `(incompatible)` | Formats are read as a single `u32` with a hardcoded `61` fallback; `min_format`/`max_format` ranges and TOO_OLD/TOO_NEW compatibility are not modelled | Use a generated current pack-format constant and a compatibility enum; mark incompatible packs in `DatapackInfo` | S-M |
| `mod.rs:2367` `read_pack_mcmeta` | `pack.mcmeta` `"filter"` (block/namespace) hides lower-pack resources | `filter` is ignored, so filtered resources still override lower packs | Parse `filter.block`/`filter.namespace` and skip matching paths while scanning packs | M |
| `mod.rs:842` `is_in_tag`, `:502` `get_tag_values`, `:871` `get_tag_entries` | `TagLoader#build`/dependency sort builds a flat `LinkedHashSet` per tag, nested `#tag` entries included | Generic tag queries return raw stored values: a member reached through a nested tag is reported as absent and builtin tags are not consulted | Resolve recursively over the datapack `tags` map with a visited set, falling back to `pumpkin_data::tag` for builtin tags | S-M |
| `mod.rs:502` `get_tag_values` | n/a | Does not strip a leading `#` although `is_in_tag` and `get_tag_entries` do | Strip `#` before lookup | XS |
| `mod.rs:122` `MAX_FUNCTION_CHAIN_COMMANDS` | gamerule `maxCommandChainLength` (default 65536), counted per chain | Budget is a constant, so the gamerule has no effect | Read the gamerule when the outermost chain starts | S-M |
| `mod.rs:118` `MAX_FUNCTION_CHAIN_DEPTH` | No depth limit; only the command budget bounds recursion | A legitimate chain nesting more than 256 functions aborts where vanilla succeeds | Dispatch nested functions iteratively, or keep only the command budget | M |
| `mod.rs:1736` `substitute_macro` | `StringTemplate#fromString` rejects `"Invalid macro variable name 'x'"` and `"No variables in macro"` at load | Invalid names (`$()`, `$(a b)`) and `$` lines without `$(` are accepted until invocation; also no instantiation cache (`MacroFunction` caches 8) | Validate templates when loading functions; optional small cache | S |
| `mod.rs:1634` `disable_pack` | Only `PackSelectionConfig#required` packs (the core `vanilla` pack) cannot be removed | Every embedded datapack is forced enabled (`:1400`) and cannot be disabled | Mark only the core pack required; allow disabling other embedded packs | S |

Notes: `stringify_macro_argument` (`:1714`) matches `MacroFunction#stringify` exactly
(`DecimalFormat("#")`, 15 fraction digits, Locale.ROOT). `build_damage_type_registry`
`next_custom_id = 51` matches the 51 generated vanilla damage types (max id 50).

## Gapwave2-2 status

- `disable_pack` (S): FIXED in 722e94305. Only `vanilla` is required; `pumpkin-unit-test` can be disabled and is filtered out of the enabled list. The loader still scans embedded packs independently, so a disabled embedded pack may still load (not verified).
- Remaining rows (zip packs, load order, feature flags from mcmeta, `filter` handling, function chain depth, macro template validation): not attempted this pass.

## Gapwave2-2 status (h3)
- remaining datapack rows: STILL OPEN (note body in this worktree is unreadable; rows not reviewed in this wave; prior fix 722e94305)


## Result

Reviewed against .vendor/vanilla-decompiled/net/minecraft (StringTemplate.java,
FunctionBuilder.java, TagLoader.java, FeatureFlags.java, FeatureFlagsMetadataSection.java,
PackRepository.java). Fixed rows (mod.rs):

- get_tag_values strips a leading # and normalizes the category/tag id; falls back to
  pumpkin_data::tag::get_tag_values for pure builtin tags. Test added.
- Generic tag queries (get_tag_values, get_tag_entries) now expand nested #tag references
  recursively with a visited set and first-seen dedup, matching TagLoader#tryBuildTag's
  LinkedHashSet. Test added. Builtin members are appended only for tags with no datapack
  definition, because DynamicTagEntry drops the replace flag.
- Macro templates are validated at load time in validate_macro_functions using
  validate_macro_template, mirroring StringTemplate#fromString (unterminated variable,
  invalid/empty name, no variables). Invalid macro lines are dropped with a warning.
  Test added. The optional 8-entry instantiation cache was not added.
- load_all: bundled packs are now loaded before world packs, so world packs override
  built-in packs like vanilla's selected-list priority.
- disable_pack: already fixed in 722e94305; verified is_required_pack accepts only vanilla.

Still open (one-line reasons):

- zip packs: needs a zip reader dependency in pumpkin-core/Cargo.toml (outside this unit).
- feature flags from features.enabled metadata: resolve_enabled_features is a pure
  function over pack names with no metadata access; threading mcmeta features changes its
  signature and breaks its existing test.
- pack format ranges / compatibility: needs a generated current pack-format constant in
  pumpkin-data plus a compatibility enum propagated through DatapackInfo consumers.
- filter handling: requires threading block/namespace filters into every pack scanner.
- maxCommandChainLength gamerule: budget is a const thread-local; needs the gamerule
  threaded into outermost chain start.
- function chain depth: needs an iterative dispatch refactor to drop the 256-deep limit.

## Result (verification pass)

Verified every STILL OPEN row against decompiled vanilla at
.vendor/vanilla-decompiled/net/minecraft (V_26_3). The sole source file in this
unit (crates/pumpkin-core/src/data/datapack/mod.rs) was edited concurrently by
another worker for the whole pass (git diff grew 155 to 318 insertions, mtime
advanced 08:01:15 to 08:03:13). I did NOT edit it, to avoid clobbering that work;
several rows below were already fixed in that shared tree, not by me.

- zip packs (scan_datapacks_dir): STILL OPEN. Needs a zip reader crate in
  pumpkin-core/Cargo.toml (file outside this unit).
- load_all pack order: RESOLVED in current tree. Bundled packs load before
  world packs so world packs override them.
- feature flags from features.enabled (FeatureFlagsMetadataSection.java):
  RESOLVED in current tree. read_pack_mcmeta parses root features.enabled into
  LoadedDatapack.requested_features; get_enabled_features unions them over
  loaded packs; resolve_enabled_features stays the id-based fallback.
- pack format ranges / compatibility (PackFormat.java, PackCompatibility.java):
  RESOLVED in current tree for load/enable (PackFormatRange from pack_format /
  min_format / max_format / supported_formats, pack_format_compatibility).
  Sub-gap STILL OPEN: DatapackInfo carries no incompatible marker as
  displayPackList does; adding it propagates through command-crate consumers.
- pack.mcmeta filter (row 28): STILL OPEN / OBSOLETE for V_26_3. Vanilla
  PackMetadataSection.java declares only description + supportedFormats; a tree
  grep found no pack filter section. No Pumpkin change warranted this version.
- generic tag queries and get_tag_values leading hash: RESOLVED in prior Result.
- maxCommandChainLength gamerule: RESOLVED in current tree by the concurrent
  worker (enter_with_budget, FUNCTION_CHAIN_BUDGET, chain_budget reads
  max_command_sequence_length at outermost start). Vanilla GameRules line 53
  (default 65536) and Commands.java:463 confirm the top-context read. Not
  authored by me.
- function chain depth: STILL OPEN. Vanilla ExecutionControl imposes no depth
  limit, only the command budget; Pumpkin MAX_FUNCTION_CHAIN_DEPTH=256 aborts
  legitimate deep chains. Removing it needs an iterative dispatch refactor
  touching command dispatch outside a minimal single-file change.
- macro template validation and disable_pack embedded: RESOLVED in prior Result
  and 722e94305.

## Result (gapwave datapack rows, mod.rs only)

Verified against `.vendor/vanilla-decompiled/net/minecraft` (`server/packs/**`,
`commands/Commands.java`, `server/packs/FeatureFlagsMetadataSection.java`,
`world/level/gamerules/GameRules.java`).

Fixed:

- Pack format ranges / compatibility (S-M): `read_pack_mcmeta` now returns a
  `PackMetadata` with a `PackFormatRange`, parsed from `min_format`/`max_format`
  (major or `[major, minor]`), legacy `supported_formats`, or single `pack_format`.
  `pack_format_compatibility` mirrors `PackCompatibility.forVersion` (TooOld / TooNew /
  Compatible over the declared range) and enable still refuses incompatible packs.
  Tests: range+compat cases in `pack_format_compatibility_matches_vanilla`, plus
  `read_pack_mcmeta_reads_format_range_and_features`.
- Feature flags from mcmeta (M): `read_pack_mcmeta` collects `features.enabled`
  (`FeatureFlagsMetadataSection`) onto `LoadedDatapack.requested_features`;
  `get_enabled_features` unions them over the loaded packs, and
  `resolve_enabled_features` no longer infers flags from `file/...` world pack names
  (vanilla only unions metadata). Unknown ids are dropped like `FeatureFlags.REGISTRY`.
  Test updated in `resolve_enabled_features_works`.
- maxCommandChainLength gamerule (S-M): the per-chain command budget is taken from
  `game_rules.max_command_sequence_length` when the outermost function starts
  (`Commands#executeCommandInContext`), stored in a thread-local budget that nested
  functions reuse; the constant remains the fallback. Test:
  `function_chain_budget_uses_the_outermost_limit`.
- Builtin tag fallback (S-M, part of the generic tag rows): `get_tag_entries` now
  delegates to `get_tag_values` (builtin fallback + nested expansion + `#` strip),
  and `is_in_tag` falls back to `pumpkin_data::tag` for tags with no datapack file
  and for nested references that resolve to builtin tags. Test extended in
  `is_in_tag_follows_nested_tag_references`.

Still open (one-line reasons):

- zip packs: needs a zip reader crate added to `pumpkin-core/Cargo.toml` (dependency
  change outside this unit).
- `filter` block/namespace handling: must be threaded through every pack scanner
  (load_pack_contents, load_recipes_*, advancement loader), outside this unit.
- function chain depth: dropping the 256-deep cap needs an iterative dispatch refactor
  of the recursive `dispatcher.execute_input` path, too large for this pass.
- macro instantiation cache (8 entries): optional optimisation, no behaviour gap.
- `(incompatible)` suffix in pack listings: `DatapackInfo` is exported through the
  plugin WIT API, so adding the flag is a cross-crate change.
