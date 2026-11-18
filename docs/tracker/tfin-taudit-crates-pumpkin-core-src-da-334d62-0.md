# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-da-334d62.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Every STILL OPEN row was re-checked against decompiled vanilla.
No STILL OPEN rows remain. No source file was edited in this pass (the fix was already present); no cargo or rustfmt run (workers cannot run shell tools).
No STILL OPEN rows remain. No source files were edited; no cargo/rustfmt run (workers cannot run shell tools).

## Result
## Result (final pass)

Re-read every source file named by the unit and re-checked each row of
`docs/tracker/taudit-crates-pumpkin-core-src-da-334d62.md` against decompiled vanilla. All fixes
are already present and correct in the checkout; no further source edit was required. Cited vanilla
source lives under `.../vanilla-decompiled/net/minecraft/`.

- test_loader.rs:93-98 / model.rs:109 (TestData / TestEnvironmentDefinition codec) -> FIXED.
  `GameTestDefinition::is_valid` (model.rs:110-139) rejects every environment shape the holder codec
  cannot decode: a String must parse as `Identifier`, an Object must carry `type`, all other shapes
  fail. `load_test_instances_recursive` (test_loader.rs:108-119) warns+skips such entries, matching
  vanilla `SimpleJsonResourceReloadListener.prepare` ("Couldn't parse data file", skip at
  `.../server/packs/resources/SimpleJsonResourceReloadListener.java:49-51`). Cited
  `.../gametest/framework/TestData.java:31` (`TestEnvironmentDefinition.CODEC.fieldOf("environment")`)
  and `.../gametest/framework/TestEnvironmentDefinition.java:35-38`
  (`RegistryCodecs.holder(Registries.TEST_ENVIRONMENT, DIRECT_CODEC)`). Covered by
  `load_test_instances_skips_invalid_environment`.

- test_loader.rs:91 (invalid resource location id) -> FIXED. The
  `Identifier::is_valid_namespace` / `is_valid_path` gate at test_loader.rs:94-101 skips ids the
  client cannot parse. Vanilla builds the id from the file path via `FileToIdConverter`
  (`.../resources/FileToIdConverter.java:12`). Covered by
  `load_test_instances_skips_invalid_resource_location_ids`.

- mod.rs:922 / mod.rs:1100 + dynamic_registry_loader.rs:93 (typed test_instance registry sync) ->
  VERIFIED, no change. `all_dynamic_registries.insert("minecraft:test_instance",
  test_loader::synced_test_instance_registry(&all_test_instances))` (mod.rs:541-543) overwrites the
  generic raw-JSON-as-NBT copy with codec-canonical `to_registry_entry` payloads
  (test_loader.rs:129-171) before `apply_custom_registries` (mod.rs:564). Matches vanilla
  `.../core/RegistrySynchronization.java:59-61` (`registryData.elementCodec().encodeStart(ops,
  element.value())`). The leftover `get_test_instance_registry_entries` accessor (mod.rs:1100) is
  harmless public API and keeps the `to_registry_entry` import (mod.rs:45) live.

- dynamic_registry_loader.rs:92-95 (directory + extension) -> VERIFIED, no change. `DYNAMIC_REGISTRIES`
  lists singular `test_instance` (dynamic_registry_loader.rs:51) and the extension check is exact
  lowercase `json` (:221-223) with the same resource-location gate (:231-239). Vanilla's
  `FileToIdConverter.json` uses the `.json` suffix (`.../resources/FileToIdConverter.java:11-12`).
  Covered by `instance_registry_uses_singular_directory_only` and
  `only_lowercase_json_files_are_loaded`.

- model.rs:85 (required `structure`) -> VERIFIED, no change. `TestData.CODEC` uses
  `Identifier.CODEC.fieldOf("structure")` (required) at
  `.../gametest/framework/TestData.java:33`; Pumpkin's non-defaulted `String` plus the `Identifier`
  parse in `is_valid` (model.rs:113) matches. Covered by `is_valid_rejects_malformed_resource_locations`.

Every row is terminal (FIXED or VERIFIED); no BLOCKED-DATA. No source files were edited in this pass;
no cargo/rustfmt run (workers cannot run shell tools). Files verified:
crates/pumpkin-core/src/data/datapack/test_loader.rs,
crates/pumpkin-core/src/data/datapack/dynamic_registry_loader.rs,
crates/pumpkin-core/src/data/datapack/mod.rs,
crates/pumpkin-gametest/src/model.rs.
