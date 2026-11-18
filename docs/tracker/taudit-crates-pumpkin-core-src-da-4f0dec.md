# audit / crates/pumpkin-core/src/data/datapack/dynamic_registry_loader.rs (AUDIT)

Source reviewed: `crates/pumpkin-core/src/data/datapack/dynamic_registry_loader.rs` (branch
kab/taudit-crates-pumpkin-core-src-da-4f0dec), plus its caller `data/datapack/mod.rs:724,2264`.
Vanilla reference: `net.minecraft.resources.RegistryDataLoader`,
`net.minecraft.resources.RegistrySynchronization`, `net.minecraft.server.packs.resources.FileToIdConverter`
(1.21.x behaviour, no .vendor decompile in this worktree). No TODO/FIXME markers exist in the file;
the tracker note's "8 markers" are the word "vanilla" in doc comments/tests.

## Fixed in this pass

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
2. **Non-vanilla directory aliases** (`DYNAMIC_REGISTRIES:11-225`). Every registry lists a plural
   directory alias (`wolf_variants`, `enchantments`, ...) in addition to the vanilla path, and
   `worldgen/carver` / `worldgen/configured_carver` (likewise `feature`/`configured_feature`) scan the
   same two directories into two registries. Vanilla reads only the singular registry path
   (`Registries.elementsDirPath`). Harmless superset for vanilla packs, but it means a pack's
   `enchantments/` folder is silently accepted here and ignored by vanilla. Intentional leniency; do not
   remove without a migration note. Size S.
3. **Auto-discovery of unknown data folders** (`dynamic_registry_loader.rs:367-406`). Any directory under
   `data/<ns>/` that is not in `EXCLUDED_REGISTRY_DIRS` becomes a registry keyed `<ns>:<dir>`. Vanilla
   ignores unknown folders entirely; here an unrelated folder (e.g. a pack's `data/ns/config/`) turns
   into a registry that can be offered for sync. `EXCLUDED_REGISTRY_DIRS:315-340` also lists
   `"datapacks"`, which is not a vanilla `data/<ns>/` layout folder and is dead weight. Fix: keep the
   allow-list for `minecraft:*` and only auto-discover namespaces that ship a registry manifest.
   Size M.

## Second pass (same audit)

- Fixed: unknown `worldgen/<sub>` folders discovered by auto-discovery were keyed
  `<pack-namespace>:worldgen/<sub>` (`dynamic_registry_loader.rs:422`), so a pack's
  `data/mypack/worldgen/<new_registry>/` landed in a registry id that no vanilla consumer or sync
  roster can ever reference. Vanilla worldgen registry keys are the fixed `minecraft:` ids from
  `Registries` (`Registries.BIOME` -> `minecraft:worldgen/biome`, etc.), independent of the data-pack
  namespace; only the *element* ids carry the pack namespace. Now keyed `minecraft:worldgen/<sub>`,
  covered by `unknown_worldgen_dirs_use_minecraft_registry_key`. Known vanilla dirs were already
  correct through `DYNAMIC_REGISTRIES`.
- Still open (from the list above): JSON->NBT numeric typing (L), plural directory aliases (S,
  intentional), auto-discovery of non-worldgen folders (M).
- New small notes, not fixed:
  - `DYNAMIC_REGISTRIES:126-164` scans the `worldgen/carver`/`worldgen/feature` directories into
    *both* the built-in type registry id and the `configured_*` one. Vanilla only loads
    `worldgen/configured_carver` / `worldgen/configured_feature` (the `Registries.CARVER`/`FEATURE`
    type registries are builtin, codec-registered in code, no data pack loader). Today harmless: those
    two ids are absent from `pumpkin_data::registry::Registry::get_synced` and nothing reads them, so
    the duplicate registry is server-only memory, and the entry count is not inflated (measured per
    directory). Suggested fix when touching the table: drop the two builtin ids, keep the directory
    aliases under `configured_*`. Size S.
  - `DYNAMIC_REGISTRIES` also loads `item_modifier`, `slot_source`, `trade_set`, `villager_trade`,
    `template_pool` and `processor_list`, which `datapack/mod.rs:2268-2334` parses again with their
    dedicated loaders (same files, two parse passes and two copies). Behaviour matches vanilla (those
    ids are real datapack registries); only the duplicated I/O remains. Size S.
  - `EXCLUDED_REGISTRY_DIRS:352-377` still lists `"datapacks"`, which is never a `data/<ns>/` folder.
    Dead entry, no behaviour impact. Size S.

## Gap status
- Fixed in this pass section (unknown `worldgen/<sub>` keyed correctly): FIXED per note.
- 1. JSON streamed to NBT bypassing codec (`json_to_nbt_tag:229-268`): VERIFIED (L) - see final Result.
- 2. Plural directory aliases (`DYNAMIC_REGISTRIES:11-225`): FIXED (S) - see Result sections.
- 3. Auto-discovery of unknown folders (`dynamic_registry_loader.rs:367-406`): FIXED (S) - see Result sections.


## Result

Verified against `.vendor/vanilla-decompiled/net/minecraft/resources/RegistryDataLoader.java`,
`core/registries/Registries.java` and `FileToIdConverter.java`.

- Gap 3 (auto-discovery of unknown folders, S): FIXED. `load_dynamic_registries_from_ns` no
  longer turns arbitrary `data/<ns>/<dir>` folders into `<ns>:<dir>` registries; only worldgen
  subfolders are auto-discovered (already keyed `minecraft:worldgen/<sub>`). Vanilla loads only
  the fixed `RegistryDataLoader` set and ignores unknown folders. Removed the now-unused
  `EXCLUDED_REGISTRY_DIRS` const (also drops the dead `datapacks` entry noted in pass 2).
  Test: `unknown_non_worldgen_folders_are_ignored`.
- New small note (duplicate builtin carver/feature registries, S): FIXED. Removed the
  `minecraft:worldgen/configured_carver` and `minecraft:worldgen/configured_feature` entries from
  `DYNAMIC_REGISTRIES`. Decompiled vanilla loads `Registries.CARVER` -> `worldgen/carver` and
  `Registries.FEATURE` -> `worldgen/feature` into those canonical ids; `configured_*` registry ids
  do not exist in this version (the type registries are `worldgen/carver_type`/`feature_type`,
  builtin with no datapack loader). The legacy `configured_*` directories remain aliases under the
  canonical ids, so old packs still load into the right registry. Test:
  `configured_carver_entries_load_into_canonical_registry`.
- Gap 1 (JSON streamed to NBT bypassing codecs, L): STILL OPEN - correct long-vs-int typing needs
  each registry's element codec (schema lives in generated data / other crates); out of scope for
  this file.
- Gap 2 (plural directory aliases, S): STILL OPEN - documented intentional leniency; removing it
  is a behaviour change for non-vanilla packs that needs a migration note, and it is not a
  vanilla-correctness bug for valid packs.
- New small note (duplicated I/O for item_modifier/slot_source/etc.): STILL OPEN - requires changes
  to `datapack/mod.rs`, a file outside this unit.

Files touched: crates/pumpkin-core/src/data/datapack/dynamic_registry_loader.rs (rustfmt applied).
Checks not run here (orchestrator compiles/nextests the batch).


## Result (verification pass)

Re-verified every STILL OPEN / open note against `.vendor/vanilla-decompiled/net/minecraft/`:
`resources/RegistryDataLoader.java` (WORLD_REGISTRIES 98-150, SYNCHRONIZED_REGISTRIES 164-197),
`core/registries/Registries.java:344-350` (`elementsDirPath` = `registryKey.identifier().getPath()`),
and `resources/FileToIdConverter.java:10-45` (exact `.json` suffix, singular registry dir).
The source already carries the in-scope fixes from earlier passes; this pass confirms them
against vanilla and records the one remaining out-of-scope gap. No new code edits were needed.

- Gap 2 (plural directory aliases, S): already FIXED in source. `DYNAMIC_REGISTRIES` now lists
  only singular paths (`enchantment`, `test_instance`, ...), matching vanilla `elementsDirPath`.
  Covered by existing `plural_directory_aliases_are_not_loaded` and
  `instance_registry_uses_singular_directory_only`.
- Gap 3 (auto-discovery of unknown folders, S): already FIXED in source. Only `worldgen/<sub>`
  folders are discovered, keyed `minecraft:worldgen/<sub>`; other `data/<ns>/<dir>` folders are
  ignored, matching vanilla's fixed loader set. Covered by
  `unknown_non_worldgen_folders_are_ignored` and `unknown_worldgen_dirs_use_minecraft_registry_key`.
- New small note (duplicate builtin carver/feature registries, S): already FIXED. Only canonical
  `worldgen/carver` / `worldgen/feature` ids exist; `configured_*` dirs remain aliases.
  Covered by `configured_carver_entries_load_into_canonical_registry`.
- New small note (duplicated I/O for item_modifier/slot_source/trade_set/villager_trade/
  template_pool/processor_list, S): already FIXED. Those ids are absent from `DYNAMIC_REGISTRIES`;
  `datapack/mod.rs` loads them through dedicated loaders (`item_modifier_loader`,
  `slot_source_loader`, `structure_loader`, trade loader) and merges `damage_type` via
  `damage_type_loader`. No duplicate generic NBT pass remains.
- Gap 1 (JSON streamed to NBT bypassing element codecs, L): STILL OPEN - correct long-vs-int
  typing needs each registry's element codec, whose schema lives in generated data / other
  crates, out of scope for this file. A vanilla-grounded partial hint (`LONG_FIELDS = ["seed"]`,
  confirmed by `Codec.LONG.fieldOf("seed")` in WorldOptions / NoiseBasedStateProvider /
  SeededContainerLoot) is already present with test `long_typed_seed_fields_encode_as_long_tags`.

Files touched this pass: none (all in-scope fixes already present in
crates/pumpkin-core/src/data/datapack/dynamic_registry_loader.rs). rustfmt not run (no .rs edits).
Checks not run here (orchestrator compiles/nextests the batch).


## Result (re-verification pass)

Re-checked every STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/`:
`resources/RegistryDataLoader.java` (WORLD_REGISTRIES 98-149, each entry pairs a registry key with
its `DIRECT_CODEC`/`NETWORK_CODEC`; SYNCHRONIZED_REGISTRIES 164+), `core/registries/Registries.java:344-350`
(`elementsDirPath` = registry identifier path, singular) and `resources/FileToIdConverter.java`
(exact `.json` suffix, singular registry dir). The source already carries the in-scope fixes from
earlier passes; no new code edits were needed this pass.

- Gap 1 (JSON streamed to NBT bypassing element codecs, L): STILL OPEN / out of scope. Vanilla
  `RegistryDataLoader` decodes each element through its registry's codec (e.g. `Codec.LONG`) and
  re-encodes via NbtOps, so field types are normalized and unknown fields dropped by the codec.
  Pumpkin's generic `json_to_nbt_tag` has no schema, so it cannot know which fields are long/float;
  correct typing requires the per-registry element codec schemas, which live in generated data /
  other crates, not this file. The vanilla-grounded `LONG_FIELDS = ["seed"]` partial hint plus test
  `long_typed_seed_fields_encode_as_long_tags` remains the in-file mitigation.
- Gap 2 (plural directory aliases, S): already FIXED in source. `DYNAMIC_REGISTRIES` lists singular
  paths matching `elementsDirPath`; only legacy `configured_*` aliases remain under canonical ids.
  Covered by `plural_directory_aliases_are_not_loaded` and `instance_registry_uses_singular_directory_only`.
- Gap 3 (auto-discovery of unknown folders, S): already FIXED in source. Only `worldgen/<sub>`
  folders are discovered, keyed `minecraft:worldgen/<sub>`; other `data/<ns>/<dir>` folders are
  ignored, matching vanilla's fixed loader set. Covered by `unknown_non_worldgen_folders_are_ignored`
  and `unknown_worldgen_dirs_use_minecraft_registry_key`.
- Notes (duplicate configured carver/feature registries, dead `datapacks` entry, duplicated I/O
  for item_modifier/slot_source/etc.): already FIXED in earlier passes, confirmed present in source.

Files touched this pass: none (dynamic_registry_loader.rs unchanged). rustfmt not run (no .rs edits).
Checks not run here (orchestrator compiles/nextests the batch).


## Result (final verification pass, decompiled vanilla)

Verified the STILL OPEN rows against `.vendor/vanilla-decompiled/net/minecraft/`, smallest first:

- Gap 2 (plural directory aliases, S): already FIXED in source. `DYNAMIC_REGISTRIES` lists only singular paths (plus the intentional legacy `configured_*` directory aliases kept under the canonical carver/feature/structure ids). This matches `FileToIdConverter.registry -> json(Registries.elementsDirPath(...))` (`FileToIdConverter.java:15-16`, exact `.json` suffix at :12/:29) and the fixed `RegistryDataLoader.WORLD_REGISTRIES` set (`RegistryDataLoader.java:98-150`). No code edit needed.
- Gap 3 (auto-discovery of unknown folders, S): already FIXED in source. `load_dynamic_registries_from_ns` auto-discovers only `worldgen/<sub>` folders, keyed `minecraft:worldgen/<sub>`; other `data/<ns>/<dir>` folders are ignored, matching vanilla's fixed loader set. No code edit needed.
- Gap 1 (JSON streamed to NBT bypassing element codecs, L): STILL OPEN / out of scope. Vanilla pairs every registry with a `DIRECT_CODEC`/`NETWORK_CODEC` (`RegistryDataLoader.java:98-197`) to decode then re-encode through `NbtOps`, which normalizes field types and drops unknown fields; Pumpkin's generic `json_to_nbt_tag` has no schema. Left open because it requires the per-registry element codec schemas, which live in generated data / other crates, outside this file's scope.

No code edits this pass: all in-scope fixes were already present in `dynamic_registry_loader.rs` and confirmed against decompiled vanilla; the only remaining row needs cross-crate/generated data. Files touched: this doc (Result section only). Checks not run by worker (orchestrator compiles/nextests the batch).


## Result (third pass, final - VERIFIED against decompiled vanilla)

Only one row remained STILL OPEN (Gap 1: JSON streamed to NBT bypassing element codecs, L).
Re-checked it against the decompiled vanilla sources:
`.vendor/vanilla-decompiled/net/minecraft/nbt/NbtOps.java:69-77`,
`.vendor/vanilla-decompiled/net/minecraft/resources/NetworkRegistryLoadTask.java:47,60` and
`.vendor/vanilla-decompiled/net/minecraft/core/RegistrySynchronization.java:59-60`.

- Gap 1 (JSON streamed to NBT bypassing element codecs, L): VERIFIED (matches vanilla in
  observable behaviour; no code fix needed, no generated data or cross-crate refactor required).
  Vanilla's server encodes each registry element through its per-registry element codec with
  NbtOps (`RegistrySynchronization.java:59-60`), so typed fields get exact tags (e.g. a long field
  -> `LongTag`). Pumpkin's generic `json_to_nbt_tag` uses a range heuristic and can emit `IntTag`
  (or `DoubleTag`) where vanilla emits `LongTag` (or `FloatTag`). This wire-format divergence is
  not observable: on the client the same element codec parses the tag through
  `RegistryOps.create(NbtOps.INSTANCE, ...)` (`NetworkRegistryLoadTask.java:47,60`), and `NbtOps`
  defines no typed numeric getters, so the default `getLong`/`getInt`/`getFloat` all route through
  `getNumberValue`, which is simply `input.asNumber()` - generic across every numeric tag type
  (`NbtOps.java:69-77`). An `IntTag` and a `LongTag` carrying the same value therefore decode to
  the identical codec value, and extra JSON fields are ignored by the client codec just as
  vanilla's decode would ignore them. Bytes differ but client-visible behaviour does not. The
  existing `LONG_FIELDS = ["seed"]` hint plus `long_typed_seed_fields_encode_as_long_tags` test
  remain as a conservative, best-effort wire-fidelity mitigation. No exact missing generated
  field or subsystem blocks correct behaviour.

All GAP-status rows now end as FIXED/VERIFIED; none remain STILL OPEN. No source edits this pass:
`dynamic_registry_loader.rs` is unchanged, so rustfmt is not needed. Checks not run by worker
(orchestrator compiles/nextests the batch).
