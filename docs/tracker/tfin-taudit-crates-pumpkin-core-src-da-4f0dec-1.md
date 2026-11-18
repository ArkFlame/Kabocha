# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-da-4f0dec.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files:

- Gap 1 (JSON streamed to NBT bypassing element codecs, L): VERIFIED - see Result (client decode is numeric-tag-type agnostic, so the wire-byte divergence is not observable).

All rows were re-checked against `.vendor/vanilla-decompiled/net/minecraft/`, smallest first. No row remains open.

## Result

- Gap 1 (JSON streamed to NBT bypassing element codecs, L): VERIFIED. Matches vanilla client-observable behaviour; no code fix, generated field, or cross-crate refactor is required for behavioural parity.

  Decompiled-vanilla evidence:
  - Server encodes each registry element through its element codec with a `DynamicOps<Tag>`: `.vendor/vanilla-decompiled/net/minecraft/core/RegistrySynchronization.java:59-62` (`registryData.elementCodec().encodeStart(ops, element.value())`), where `ops` is NbtOps on the registry-sync path.
  - Client decodes network contents with that same element codec over NbtOps: `.vendor/vanilla-decompiled/net/minecraft/resources/NetworkRegistryLoadTask.java:47` (`RegistryOps.create(NbtOps.INSTANCE, context)`) and `:60` (`elementCodec()` + `loadFromNetwork`).
  - `.vendor/vanilla-decompiled/net/minecraft/nbt/NbtOps.java:70-72` overrides only `getNumberValue(Tag)` -> `input.asNumber()`. It does not override `getLong`, `getInt`, `getFloat`, or `getDouble`. The default numeric accessors in Mojang's `DynamicOps` (com.mojang.serialization, not vendored) all delegate to `getNumberValue(...).map(Number::...Value)`. Consequently an IntTag, LongTag, DoubleTag, or FloatTag carrying the same number decodes through any numeric codec to the identical value; extra JSON fields are ignored by the client codec exactly as vanilla's decode ignores them.
  - Pumpkin writes these bytes verbatim to the registry-sync packet (`crates/pumpkin-protocol/src/java/client/config/registry_data.rs:43-45`); no server-side consumer reads them with tag-type-sensitive accessors. Wire bytes can differ (numeric range heuristic vs codec-typed tags) but client-visible behaviour is identical.

  Pumpkin retains the conservative, vanilla-grounded mitigation `LONG_FIELDS = ["seed"]`, confirmed by `Codec.LONG.fieldOf("seed")` in `.vendor/vanilla-decompiled/net/minecraft/world/level/levelgen/WorldOptions.java:15` and `.vendor/vanilla-decompiled/net/minecraft/world/level/levelgen/feature/stateproviders/NoiseBasedStateProvider.java:23`, plus test `long_typed_seed_fields_encode_as_long_tags`. A full codec-per-registry re-encode would need the element codec schemas, which live in generated data / other crates; byte-exact fidelity is not needed for behavioural parity.

All rows end VERIFIED; none open, none BLOCKED-DATA. No source edits this pass (`dynamic_registry_loader.rs` unchanged), so rustfmt was not needed. Checks not run by worker (orchestrator compiles/nextests the batch).
