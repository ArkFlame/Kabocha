# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-da-4f0dec.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- Gap 1 (JSON streamed to NBT bypassing codecs, L): VERIFIED - client decode via NbtOps is numeric-type-generic; no generated field or cross-crate codec schema needed for correct behaviour (see Result).
- Gap 2 (plural directory aliases, S): FIXED - DYNAMIC_REGISTRIES lists only singular paths (see Result).
- New small note (duplicated I/O for item_modifier/slot_source/etc.): FIXED - dedicated loaders in datapack/mod.rs, ids absent from DYNAMIC_REGISTRIES (see Result).
Re-verified every STILL OPEN / open note against `.vendor/vanilla-decompiled/net/minecraft/`:

## Result
Re-verified all three rows against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/`. No source edits were needed: `dynamic_registry_loader.rs` already carries every in-scope fix.

- Gap 1 (JSON streamed to NBT bypassing codecs, L): VERIFIED. Vanilla encodes each synced element through its element codec with `NbtOps` (`core/RegistrySynchronization.java:59-62`, `ops` is NbtOps), so a `Codec.LONG` field becomes a `LongTag`; the client decodes with `RegistryOps.create(NbtOps.INSTANCE, ...)` + `elementCodec()` (`resources/NetworkRegistryLoadTask.java:47,60`). `NbtOps` overrides only `getNumberValue` as `input.asNumber()` (`nbt/NbtOps.java:69-72`) and every numeric accessor derives from it, so an `IntTag`/`LongTag`/`DoubleTag`/`FloatTag` carrying the same number decodes to the identical codec value. Wire bytes differ but client-visible behaviour is identical, so no generated field or cross-crate codec schema is missing for correctness. The existing `LONG_FIELDS = ["seed"]` hint plus `long_typed_seed_fields_encode_as_long_tags` test keep best-effort wire fidelity.
- Gap 2 (plural directory aliases, S): FIXED. `DYNAMIC_REGISTRIES` lists only singular paths. This matches vanilla `FileToIdConverter.registry(...)` = `json(Registries.elementsDirPath(...))` (`resources/FileToIdConverter.java:11-16`) with exact `.json` suffix (`:28-30`) and `elementsDirPath` = registry identifier path (`core/registries/Registries.java:344-350`). Covered by `plural_directory_aliases_are_not_loaded` and `instance_registry_uses_singular_directory_only`.
- New small note (duplicated I/O for item_modifier/slot_source/etc., S): FIXED. Those ids are absent from `DYNAMIC_REGISTRIES`; `datapack/mod.rs` loads them through dedicated modules (`damage_type_loader`, `item_modifier_loader`, `slot_source_loader`, `trade_loader`, `structure_loader`) into gameplay registries. This mirrors vanilla's separate `RELOADABLE_REGISTRIES` loaders for `ITEM_MODIFIER`/`SLOT_SOURCE` (`resources/RegistryDataLoader.java:154-163`) and the `DAMAGE_TYPE`/`VILLAGER_TRADE`/`TRADE_SET` entries in `WORLD_REGISTRIES` (`:133,145-146`). `DEDICATED_WORLDGEN_DIRS = ["template_pool", "processor_list"]` keeps those two folders out of generic auto-discovery (vanilla `WORLD_REGISTRIES` lines 107-108).

All rows end FIXED/VERIFIED; none BLOCKED-DATA, none STILL OPEN. No source edits this pass, so rustfmt not needed. Checks not run by worker (orchestrator compiles/nextests the batch).
