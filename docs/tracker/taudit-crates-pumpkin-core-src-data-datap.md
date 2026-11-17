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
