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
| mod.rs:2079 (scan loop), 1384 | `PackRepository` / `FolderRepositorySource` / `FilePackResources` | `.zip` datapacks are listed by `get_all_known_packs` but never loaded (`scan_datapacks_dir` only accepts directories), so `/datapack enable` on a zip appears to work but loads nothing. | Introduce a small read-only pack filesystem abstraction (dir or zip) and thread it through `load_pack_contents`, `read_pack_mcmeta`, structure/worldgen loaders. | L |
| mod.rs:122 | `CommandFunctionManager` (`maxCommandChainLength` game rule) | Command budget is the fixed constant 65536; vanilla reads the `maxCommandChainLength` game rule per execution, so lowering it in game rules is ignored. | Add `maxCommandChainLength` to the gamerule set, read it from the executing world before running a chain, and reset the thread-local budget accordingly. | M |
| mod.rs:2409-2421 | `PackMetadataSection` / `Component.Serializer` | `pack.mcmeta` `description` that is a JSON text component is stored as raw JSON (`d.to_string()`), so `/datapack list` shows JSON instead of text. | Parse the component and keep its plain text (fall back to `translate` key), or store a `TextComponent`. | S |
| mod.rs:1459-1483, 2423 | `DataPackVersion` / `PackFormat` | `pack_format` for the vanilla/embedded/bundled packs and the mcmeta default are the literal `61`, unrelated to `CURRENT_MC_VERSION`; a version bump silently desyncs them. | Derive the current data pack format once from `CURRENT_MC_VERSION` (const next to `CURRENT_MC_VERSION`) and use it for all defaults. | S |
| mod.rs:1729 | `MacroFunction.stringify` (via `Tag.getAsString`) | Floats/doubles are printed with 15 fixed decimals and trailing zeros trimmed, so `1.0` becomes `1` and large values use Rust formatting. Java `Double.toString`/`Float.toString` produce `1.0` and differ at boundaries, changing macro-expanded command text. | Mirror Java `Double.toString`/`Float.toString` output (or add test fixtures from vanilla) before changing behaviour. | S |
| mod.rs:1130 | `FunctionArgument#listSuggestions` | `get_function_names` returns `HashMap` iteration order; vanilla suggestions are alphabetical, so `/function <tab>` output is nondeterministic. | `names.sort_unstable()` before returning. | S |
| mod.rs:2000-2003, 1352+ | `VanillaPackResources` feature packs | `"bundle"` is accepted by known-packs and feature resolution (`mod.rs:941`, `mod.rs:1000+`) but missing from `load_bundled_datapacks` and `get_all_known_packs`, so enabling the bundle experiment loads no data. | Add `"bundle"` to both bundled lists once the assets datapack exists. | S |
| mod.rs:1806 | `RegistryDataLoader` raw ids | Custom damage type ids are `u8` and `saturating_add`, so beyond 205 custom types ids collide at 255; vanilla registry ids are ints. | Use a wider id type or reject overflow with a load error. | S |
| mod.rs:500, 884 | `Tag#getEntries` (flattened holder set) | `get_tag_values` / `get_tag_entries` return raw `#tag` reference strings; vanilla returns the flattened member list. | Reuse the `tag_contains_entry` recursion to expand nested references. | S |
| function_loader.rs:30, mod.rs:2465 | `FolderRepositorySource` | Loaders skip symlinked files/dirs (`entry.file_type()`), while vanilla's directory source follows them; a pack symlinking shared content silently loads nothing. | Use `path.is_dir()` / `path.is_file()` in the recursive loaders. | S |
| mod.rs:2409 | `PackMetadataSection` format range | An incompatible `pack_format` is accepted without warning; vanilla marks the pack incompatible and refuses to enable it. | Compare `pack_format` (or min/max range) against the current format and reject/skip with a log. | S |
