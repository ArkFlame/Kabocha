# audit: crates/pumpkin-core/src/data/datapack/dynamic_registry_loader.rs (AUDIT)

Audited 2026-11-17 against vanilla `net.minecraft.resources.RegistryDataLoader`,
`Registries.elementsDirPath`, `RegistrySynchronization` and
`net.minecraft.server.packs.resources.FileToIdConverter` (1.21.x behaviour, no
`.vendor` decompile in this worktree), plus `MappedRegistry` merge semantics
(datapack entries replace built-in values in place, new ones are appended) and
`PathPackResources` listing (no symlink following).
Entry ids are `<ns>:<path below the registry dir>`; unknown `data/<ns>/` folders are
ignored by vanilla. No TODO/FIXME markers exist in the file: the nine "vanilla"
grep hits are doc comments that describe parity and were verified.

## Fixed on branch kab/taudit-crates-pumpkin-core-src-da-4f0dec (2026-11-17)

- New `normalize_numeric_list` (line 234) used by `json_to_nbt_tag`'s array arm: a
  JSON array mixing numeric widths (`[1, 3000000000]`, `[1, 2.5]`) became a
  heterogeneous `NbtTag::List`, which `pumpkin-nbt` encodes as a `COMPOUND` list of
  `{"": value}` wrapper compounds (`crates/pumpkin-nbt/src/tag.rs:65-119`) instead
  of plain numbers. Numeric elements are now widened to the widest type present,
  the same shape a homogeneous JSON array already produced.
- `load_registry_dir_recursive` (line 334-346): directory symlinks were followed.
  Vanilla's pack listing (`Files.find` / `PathPackResources`) does not follow them,
  and a symlink loop in `data/<ns>/<registry>/` recursed until the stack blew.
  Symlinked *files* are still read so dev packs keep working.
- `load_dynamic_registries_from_ns` (line 430-455): a registry map was materialised
  for every one of the ~60 known registry ids for every namespace, even when the
  namespace had no such directory, so `apply_custom_registries` rebuilt every
  vanilla registry and registered empty registries vanilla does not have. Empty
  namespaces now contribute nothing.

## Fixed in the earlier pass on this branch

- JSON `null` was converted to `NbtTag::End`, which the compound/list writer emits as a type-0 entry.
  A reader stops at the first End, so any key after a null field (and the rest of the registry entry
  payload sent to the client) was lost, i.e. a single `"field": null` truncated the whole element.
  `json_to_nbt_tag` now returns `Option<NbtTag>` and drops null object keys / list elements, matching
  NBT having no null and vanilla codecs never encoding one (`json_to_nbt_tag`, `json_to_registry_nbt_bytes`).
- Files were matched with `eq_ignore_ascii_case("json")` but the suffix stripped with a case-sensitive
  `strip_suffix(".json")`, so `Foo.JSON` loaded under the id `ns:Foo.JSON` (invalid resource location).
  Vanilla `FileToIdConverter` only accepts the exact `.json` suffix; the loader now does the same
  (`load_registry_dir_recursive`).
- Invalid input is now skipped with a warning instead of stored or mangled: `load_registry_dir_recursive`
  rejects ids that are not valid resource locations, unreadable or unparsable JSON, and non-object JSON
  roots (vanilla `FileToIdConverter` + registry codec behaviour); the `json_to_registry_nbt_bytes` doc
  now states a non-object root never reaches it. Covered by the `invalid_registry_files_are_skipped` test.
- Deleted the no-op `reg_entry.entry_id.clone_from(&entry.entry_id)` in
  `merge_dynamic_registry_entries` (the struct literal already sets that field).

## Remaining gaps

1. **JSON is streamed to NBT, bypassing the vanilla codec** (`json_to_nbt_tag:229-268`). Vanilla decodes
   each element with its registry codec and re-encodes through `NbtOps`, so field types are normalized
   (`Codec.LONG` -> `LongTag`) and unknown fields are dropped only if the codec does. Here every JSON
   integer becomes `Int` unless it exceeds `i32`, so a datapack that writes a long-typed field
   (e.g. structure/placement seeds, `worldgen/noise_settings` long values) sends `IntTag` where vanilla
   sends `LongTag`. Most client codecs read numbers generically, so this is latent, not yet observed
   broken. Fix: per-registry codecs, or a value-range hint table. Size L.

2. **Auto-discovery of unknown data folders** (`load_dynamic_registries_from_ns:459-499`). Any directory
   under `data/<ns>/` that is not in `EXCLUDED_REGISTRY_DIRS` becomes a registry keyed `<ns>:<dir>`, and
   any unknown `worldgen/<sub>` becomes `<ns>:worldgen/<sub>`. Vanilla ignores unknown folders entirely;
   here an unrelated folder (e.g. a pack's `data/ns/config/`) turns into a registry that can be offered
   for sync, and a mis-spelled plural for a known registry (`data/ns/trial_spawners/`) becomes
   `<ns>:trial_spawners` instead of loading into `minecraft:trial_spawner`, because `visited_paths` only
   holds directories listed in `DYNAMIC_REGISTRIES`. `EXCLUDED_REGISTRY_DIRS:397-422` also lists
   `"datapacks"`, which is not a vanilla `data/<ns>/` layout folder and is dead weight. Fix: keep the
   allow-list for `minecraft:*`, canonicalise dir names, and only auto-discover namespaces that ship a
   registry manifest. Size M.

3. **Non-numeric heterogeneous JSON arrays** (`normalize_numeric_list:234-264`). `["a", 1]`,
   `[true, 5]`, `[{...}, 1]` still fall through to the `{"": value}` wrapper-compound encoding of
   `pumpkin-nbt`. Vanilla `JsonOps` hands each element to the field codec, so such a file either decodes
   (per-codec leniency) or fails the whole load with an error; Pumpkin stores NBT no codec can read.
   Fix: reject the file at load time with a warning, matching vanilla's failed load. Size S once the
   policy is picked.

4. **Non-compound fallback writes a typed tag** (`json_to_registry_nbt_bytes:311-325`). The `other =>`
   arm uses `NbtTag::serialize`, which writes the type id where the unnamed format expects a payload
   only. Unreachable from the loader (line 379 rejects non-object roots) but it is `pub` and used as an
   encoding helper. Vanilla fails the file instead of encoding it. Fix: `serialize_data` or an explicit
   `Result`. Size S.

5. **Silent filesystem errors** (`load_registry_dir_recursive:334` `fs::read_dir` early return, `:358`
   `entries.flatten()`). Permission/I/O failures drop files without a log line; vanilla logs the pack
   read failure. Fix: `warn!` the path. Size S.

6. **`merge_dynamic_registry_entries` trusts its input** (`:509-566`). Entry ids coming from
   `custom_entries` are never validated as resource locations (only file names are validated in the
   loader), and a `RegistryEntryData { data: None }` would override a built-in entry with an empty
   payload. Vanilla validates keys at pack load and always has decoded values. Fix:
   `debug_assert`/skip `None` payloads. Size S.

## Notes (intentional, not gaps)

- **Non-vanilla directory aliases** (`DYNAMIC_REGISTRIES:13-227`). Every registry lists a plural
  directory alias (`wolf_variants`, `enchantments`, ...) in addition to the vanilla path, and
  `worldgen/carver` / `worldgen/configured_carver` (likewise `feature`/`configured_feature`) scan the
  same two directories into two registries, in fixed `dir_names` order so last-wins is deterministic.
  Vanilla reads only the singular registry path (`Registries.elementsDirPath`). Harmless superset for
  vanilla packs, but a pack's `enchantments/` folder is silently accepted here and ignored by vanilla.
  Intentional leniency; do not remove without a migration note. Size S.
- Custom-only entries are appended after sorting lexicographically (`merge_dynamic_registry_entries`
  line 552-563). Vanilla's order is pack iteration order; the numeric registry ids are server-defined
  and sent with `SynchronizedRegistries`, so clients follow our order. Left as is.
- `RegistryEntryData.data` is cloned on every merge (full payload copy per reload). Sharing
  `Arc<[u8]>` would avoid it. Size S, perf only.
- `damage_type` is not in `DYNAMIC_REGISTRIES`; it is loaded by `damage_type_loader` +
  `build_damage_type_registry`, which then injects `minecraft:damage_type` into the registry map
  (`data/datapack/mod.rs:382-394`). Correct, but two loaders must stay in sync.
