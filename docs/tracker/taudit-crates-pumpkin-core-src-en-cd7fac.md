# audit / crates/pumpkin-core/src/entity/ai/brain/memory/types.rs

Reference: `.vendor` absent; compared against the decompiled tree in
`/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft/world/entity/ai/memory/MemoryModuleType.java`
(115 registered types). Every entry in `types.rs` was diffed by name, declaration order
and codec presence against that file.

Fixed in this pass:

- `types.rs:86` - `IS_TEMPTED` (`minecraft:is_tempted`) does not exist in vanilla
  `MemoryModuleType`; temptation uses `TEMPTING_PLAYER`/`TEMPTATION_COOLDOWN_TICKS`
  (`TemptGoal` writes `TEMPTING_PLAYER`). The extra entry shifted every later id by one.
  Removed, registry renumbered to 115 entries in vanilla order, persistent codec count
  53 -> 52. After the change all 115 remaining names, ids and codec presence match vanilla.

Remaining gaps:

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| `types.rs:121-122` (`VISIBLE_ADULT_PIGLIN_COUNT`, `VISIBLE_ADULT_HOGLIN_COUNT`) | `PiglinSpecificSensor.java:107-108` writes both counts from the visible adult lists; `PiglinAi.java:780-781` and `HoglinAi.java:179-180` read them to make the outnumbered side retreat | Nothing writes or reads the counts (the `// TODO` at `types.rs:120`) | Port `PiglinSpecificSensor` and the two `retreatFromNearestTarget` call sites | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `types.rs:118-122` (`NEARBY_ADULT_PIGLINS`, `NEAREST_VISIBLE_ADULT_PIGLINS`, `NEAREST_VISIBLE_ADULT_HOGLINS`, `NEAREST_VISIBLE_ADULT_PIGLIN`) | Written by `PiglinSpecificSensor`/`HoglinSpecificSensor` | Declared but never populated; `crates/pumpkin-core/src/entity/ai/sensing.rs` has no piglin/hoglin specific sensor | Port the two species sensors together with the counts above | L, STILL OPEN: size L, skipped per wave brief |

Checked and considered correct: all other 112 type names, their declaration order, and
codec presence (global pos, global pos set/list, block pos, uuid, int, long, bool, unit)
match vanilla; types without a codec in vanilla also have `None` here.

## Gap status
- VISIBLE_ADULT_PIGLIN/HOGLIN_COUNT never written or read (types.rs:120-122): STILL OPEN, M, needs PiglinSpecificSensor and HoglinAi/PiglinAi call sites.
- NEARBY_ADULT_PIGLINS / NEAREST_VISIBLE_ADULT_* never populated (types.rs:118-122): STILL OPEN, L, no piglin/hoglin specific sensor in sensing.rs.

## Result

Both remaining rows were verified against decompiled vanilla
(`.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/sensing/PiglinSpecificSensor.java`
and `HoglinSpecificSensor.java`). Both need files outside this unit
(`entity/ai/brain/sensing/mod.rs`, new species sensor files there, the piglin/hoglin
brain sensor registration, and their `retreatFromNearestTarget` AI call sites), so
neither could be fixed within `ai/sensing.rs` (only a line-of-sight cache) or
`brain/memory/types.rs` (declarations already match all 115 vanilla entries).

- VISIBLE_ADULT_PIGLIN_COUNT / VISIBLE_ADULT_HOGLIN_COUNT (M): STILL OPEN. Needs
  PiglinSpecificSensor/HoglinSpecificSensor writing both brain memories plus the
  PiglinAi/HoglinAi retreat call sites; all outside this unit's two files.
- NEARBY_ADULT_PIGLINS / NEAREST_VISIBLE_ADULT_PIGLINS /
  NEAREST_VISIBLE_ADULT_HOGLINS / NEAREST_VISIBLE_ADULT_PIGLIN (L): STILL OPEN. Needs
  the two species sensors wired into the piglin/hoglin brain in `brain/sensing/`;
  outside this unit. Piglins/hoglins currently use goal-based AI with no brain sensors.

No source files changed; no rustfmt needed.
