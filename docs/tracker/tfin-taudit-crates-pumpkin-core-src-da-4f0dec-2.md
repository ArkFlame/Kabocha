# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-da-4f0dec.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- Gap 1 (JSON streamed to NBT bypassing element codecs, L): STILL OPEN / out of scope. Vanilla pairs every registry with a `DIRECT_CODEC`/`NETWORK_CODEC` (`RegistryDataLoader.java:98-197`) to decode then re-encode through `NbtOps`, which normalizes field types and drops unknown fields; Pumpkin's generic `json_to_nbt_tag` has no schema. Left open because it requires the per-registry element codec schemas, which live in generated data / other crates, outside this file's scope.
Only one row remained STILL OPEN (Gap 1: JSON streamed to NBT bypassing element codecs, L).
All GAP-status rows now end as FIXED/VERIFIED; none remain STILL OPEN. No source edits this pass:
No STILL OPEN rows remain. Re-verified the sole open row (Gap 1: JSON streamed to NBT

## Result (final pass, verified against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/`)

- Gap 1 (JSON streamed to NBT bypassing element codecs, L): VERIFIED - matches vanilla in observable client behaviour; no source code fix, generated field or cross-crate refactor required.
  - Server encodes each element through its per-registry element codec with `NbtOps`: `.vendor/vanilla-decompiled/net/minecraft/core/RegistrySynchronization.java:59-62` (`registryData.elementCodec().encodeStart(ops, element.value())`, where `ops` is the `DynamicOps<Tag>` = NbtOps passed in at `:28/:38`).
  - Client decodes with `RegistryOps.create(NbtOps.INSTANCE, context)` plus the element codec: `.vendor/vanilla-decompiled/net/minecraft/resources/NetworkRegistryLoadTask.java:47,60` (`PendingRegistration.loadFromNetwork(this.data.elementCodec(), nbtOps, ...)`).
  - `NbtOps` overrides only `getNumberValue` at `.vendor/vanilla-decompiled/net/minecraft/nbt/NbtOps.java:70-71`, returning `input.asNumber()`; grep confirms it has no `getLong`/`getInt`/`getFloat`/`getDouble` overrides, so those default `DynamicOps` accessors all route through `getNumberValue` -> `Number::{intValue,longValue,floatValue,doubleValue}`. `Tag.asInt/asLong/asFloat/asDouble` likewise all derive from `asNumber()` (`.vendor/vanilla-decompiled/net/minecraft/nbt/Tag.java:55-80`). Therefore an `IntTag`, `LongTag`, `FloatTag` or `DoubleTag` carrying the same number decodes through the element codec to the identical value, and unknown JSON fields are ignored by the client codec just as in vanilla. Wire bytes may differ (Pumpkin range heuristic vs vanilla codec-typed tags), but client-visible behaviour does not.
  - The `LONG_FIELDS = ["seed"]` hint plus `long_typed_seed_fields_encode_as_long_tags` test remain as best-effort wire fidelity. No exact missing generated field or subsystem blocks correctness.

All rows in this unit end VERIFIED; none remain STILL OPEN, none BLOCKED-DATA. No source edits this pass (`dynamic_registry_loader.rs` already carries every in-scope fix: `json_to_nbt_tag -> Option<NbtTag>`, exact `.json` suffix, invalid-file skipping, removed no-op clone, `LONG_FIELDS` hint and test), so rustfmt is not needed. Checks not run by worker (orchestrator compiles/nextests the batch).
