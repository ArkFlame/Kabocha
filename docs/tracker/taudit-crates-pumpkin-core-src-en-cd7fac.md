# audit / crates/pumpkin-core/src/entity/ai/brain/memory/types.rs

Reference: `.vendor` absent; compared against the decompiled tree in
`/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft/world/entity/ai/memory/MemoryModuleType.java`
(115 registered types). Every entry in `types.rs` was diffed by name, declaration order
and codec presence against that file.

Markers: none left. The tracker's "2 TODO/FIXME/vanilla markers" are false positives: both
hits are test names (`:180` `registry_has_every_vanilla_type_in_declaration_order`,
`:210` `persistent_type_count_matches_vanilla`). The old `// TODO` at `:120` was already
removed.

## Verified this pass (mechanical diff against the reference file)

Script parsed every `register("name", codec)` call in the vanilla file (balanced-paren
scan, so the multi-line `xmap` / `Codec.list` entries are included) and compared it with
the `memory_module_types!` table by position, name and codec:

- 115 vs 115 entries, identical order and identical resource names, first to last.
- Identical persistent set: the same 52 types carry a codec
  (`GLOBAL_POS`, `GLOBAL_POS_SET`, `GLOBAL_POS_LIST`, `BLOCK_POS`, `UUID`, `INT`, `LONG`,
  `BOOL`, `UNIT`); every codec-less vanilla entry is `None` here.
  `VISITED_BLOCK_POSITIONS`/`UNREACHABLE_TRANSPORT_BLOCK_POSITIONS` really are
  `GlobalPos.CODEC.listOf().xmap(Sets::newHashSet, Lists::newArrayList)` in vanilla,
  `SNIFFER_EXPLORED_POSITIONS` is `Codec.list(GlobalPos.CODEC)`, `BREEZE_JUMP_TARGET` is
  `BlockPos.CODEC`, `ANGRY_AT`/`LIKED_PLAYER` are `UUIDUtil.CODEC`.
- Declared Rust types line up with the codec they carry (each has a `MemoryValue` impl in
  `value.rs`), and `codec_of`/`name_of`/`from_name` are bounds-safe and mutually consistent.

No change was needed in `types.rs`; nothing committed for it.

## Remaining gaps (outside this file, unchanged)

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| `types.rs:121-122` (`VISIBLE_ADULT_PIGLIN_COUNT`, `VISIBLE_ADULT_HOGLIN_COUNT`) | `PiglinSpecificSensor.java:107-108` writes both counts from the visible adult lists; `PiglinAi.java:780-781` and `HoglinAi.java:179-180` read them to make the outnumbered side retreat | Nothing writes or reads the counts (grep over `crates/pumpkin-core/src` finds only the declarations) | Port `PiglinSpecificSensor` and the two `retreatFromNearestTarget` call sites | M |
| `types.rs:117-121` (`NEARBY_ADULT_PIGLINS`, `NEAREST_VISIBLE_ADULT_PIGLINS`, `NEAREST_VISIBLE_ADULT_HOGLINS`, `NEAREST_VISIBLE_ADULT_PIGLIN`) | Written by `PiglinSpecificSensor`/`HoglinSpecificSensor` | Declared but never populated; `crates/pumpkin-core/src/entity/ai/sensing.rs` has no piglin/hoglin specific sensor | Port the two species sensors together with the counts above | L |
