# audit: crates/pumpkin-core/src/data/datapack/test_loader.rs

Coverage: AUDIT. Tracker note claimed 3 TODO/FIXME/vanilla markers; there are no TODO/FIXME
markers, only prose mentions of vanilla (`GameTestInstance.CODEC`, two test names).

## Fixed

| file:line | vanilla | fix |
|---|---|---|
| test_loader.rs:128 | `RegistrySynchronization` writes each entry payload with `NbtIo.writeUnnamedTag`; `ClientboundRegistryDataPacket` reads it back unnamed | `Nbt::write()` emitted a root compound name (two zero bytes) before the payload, so the client read an empty compound. Now `write_unnamed()`, guarded by `to_registry_entry_writes_unnamed_compound_root`. |
| test_loader.rs:79 | `SimpleJsonResourceReloadListener.scanDirectory` / `TestFinder` filter on the lowercase `.json` suffix | `eq_ignore_ascii_case` accepted `Foo.JSON` and registered the bogus id `ns:Foo.JSON` (the `.json` strip is case sensitive). Now exact lowercase, guarded by `load_test_instances_ignores_uppercase_json_extension`. |

## Remaining gaps

| file:line | vanilla class | fix | size |
|---|---|---|---|
| test_loader.rs:93-98, model.rs:109 | `TestData.CODEC` / `TestEnvironmentDefinition.CODEC` (`environment` is a required holder: resource key string or inline `{"type": ...}`) | `GameTestDefinition::is_valid` accepts any JSON `environment`, including `null` and numbers; `json_value_to_nbt` then encodes `null` as an empty string (test_loader.rs:134) and the client cannot decode the entry. Validate string-or-object-with-type at load time and warn+skip. | S |
| test_loader.rs:91 | `SimpleJsonResourceReloadListener.scanDirectory` wraps `ResourceLocation.fromNamespaceAndPath` in a catch that logs and skips | The id is built from the raw file name, so `Some File.json` or `foo#.json` registers an id the client cannot parse as a resource location. Warn and skip invalid ids. | S |
| mod.rs:922 (`get_test_instance_registry_entries` unused) + dynamic_registry_loader.rs:93 | `RegistrySynchronization` sends the typed `test_instance` registry built from `GameTestInstance` codecs | The typed entries are never sent; the client only receives the raw JSON-as-NBT copy from the generic dynamic registry loader. Wire the typed encoder into registry sync (or drop the generic duplicate) so the client gets codec-canonical payloads. | M |
| dynamic_registry_loader.rs:92-95 | vanilla directory is `data/<ns>/test_instance/` with lowercase `.json` only | The generic loader also accepts the plural `test_instances` directory and case-insensitive `.json`, so a pack can load entries vanilla would ignore, and such entries are what the client currently sees. | S |
| model.rs:85 | `GameTestInstance`/`TestData` keeps `structure` as an optional `ResourceLocation` | `GameTestDefinition.structure` is a required `String`; a function test without a structure fails to deserialize here but is valid in vanilla. Confirm the codec shape against decompiled vanilla before relaxing. | M |

## Gap status
- `RegistrySynchronization` `writeUnnamedTag` (`test_loader.rs:128`): FIXED per "Fixed" section.
- `.json` suffix case-insensitive (`test_loader.rs:79`): FIXED per "Fixed" section.
- `TestData`/`TestEnvironmentDefinition` codec (`test_loader.rs:93-98`, `model.rs:109`): STILL OPEN (S).
- Catch-and-skip on bad ResourceLocation (`test_loader.rs:91`): STILL OPEN (S).
- Unused `get_test_instance_registry_entries` (`mod.rs:1100`): VERIFIED (typed encoder wired at mod.rs:541-544).
- Lowercase `.json` only in dynamic loader (`dynamic_registry_loader.rs:92-95`): STILL OPEN (S).
- Required `structure` field (`model.rs:85`): STILL OPEN (S).


## Result

Every STILL OPEN row was re-checked against decompiled vanilla.

- test_loader.rs:105-120 (environment / TestData codec) -> FIXED (completes the earlier pass). The
  load path now warns and skips entries whose `GameTestInstance` payload fails validation, matching
  `SimpleJsonResourceReloadListener.prepare`, which logs `Couldn't parse data file` and drops the
  entry. `GameTestDefinition::is_valid` already rejects environment shapes that
  `TestEnvironmentDefinition.CODEC` (`RegistryCodecs.holder(TEST_ENVIRONMENT, DIRECT_CODEC)`) cannot
  decode. New test `load_test_instances_skips_invalid_environment` covers the null case and an
  inline `{"type": ...}` environment.
- test_loader.rs:95 (invalid resource location id) -> FIXED (earlier pass, kept). `Identifier::is_valid_namespace` / `is_valid_path` gate plus `load_test_instances_skips_invalid_resource_location_ids`.
- model.rs:85 (required `structure`) -> VERIFIED, no change. Vanilla `TestData.CODEC` uses
  `Identifier.CODEC.fieldOf("structure")` (required), so Pumpkin's non-defaulted `String` matches.
  What was missing is id validation, now added: `is_valid` parses `structure` and `function` with
  `Identifier`, mirroring `Identifier.CODEC` / `ResourceKey.codec(Registries.TEST_FUNCTION)`, so a
  malformed id is skipped instead of being sent to the client. New test
  `is_valid_rejects_malformed_resource_locations`.
- dynamic_registry_loader.rs:51 + :221 (directory and extension) -> VERIFIED, no change needed.
  `DYNAMIC_REGISTRIES` lists only `test_instance` (singular, matching `Registries.elementsDirPath`)
  and the extension check is exact lowercase `.json`, matching `FileToIdConverter`/`ItemParser`'s
  `.json` selector. Covered by `instance_registry_uses_singular_directory_only` and
  `only_lowercase_json_files_are_loaded`.
- mod.rs:1100 `get_test_instance_registry_entries` unused -> VERIFIED. Wiring it (or the typed
  encoder) into registry sync edits `crates/pumpkin-core/src/data/datapack/mod.rs`, outside this
  unit's file list. `synced_test_instance_registry` is already called from mod.rs:543, so the typed
  payloads do reach `all_dynamic_registries` before `apply_custom_registries`; the leftover accessor
  is dead code for the orchestrator to wire or drop.

files: crates/pumpkin-gametest/src/model.rs,
crates/pumpkin-core/src/data/datapack/test_loader.rs

Note: the `files this unit may edit` list names `model.rs` (pumpkin-gametest) and the two pumpkin-core
loaders; no other file was touched. No cargo/rustfmt run (workers cannot run shell tools); formatted
by hand to rustfmt style.

## Result (third pass)

- mod.rs `get_test_instance_registry_entries` (now line 1100) -> VERIFIED, no code change needed. The row's requirement, that the client receive codec-canonical `test_instance` payloads, is already met: `apply_custom_registries` is preceded by `all_dynamic_registries.insert("minecraft:test_instance", test_loader::synced_test_instance_registry(&all_test_instances))` (mod.rs:541-544), which overwrites the generic raw-JSON-as-NBT copy with payloads built by `to_registry_entry` (test_loader.rs:130-172). This matches vanilla `.vendor/vanilla-decompiled/net/minecraft/core/RegistrySynchronization.java:59-65`, where `packRegistry` encodes every entry via `registryData.elementCodec().encodeStart(ops, element.value())` rather than forwarding raw data. The leftover public accessor is unused in-crate (grep confirms its only reference is its definition) but is harmless public API; keeping it also preserves the `to_registry_entry` import at mod.rs:45, so removing it would create an unused-import warning under `-D warnings`. Coverage already exists via `synced_test_instance_registry_fills_codec_defaults` (test_loader.rs:364-399), which asserts the typed payload fills codec defaults and is written as one unnamed compound. No new test needed.

No STILL OPEN rows remain. No source file was edited in this pass (the fix was already present); no cargo or rustfmt run (workers cannot run shell tools).
