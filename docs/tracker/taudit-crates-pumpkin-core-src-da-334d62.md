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
