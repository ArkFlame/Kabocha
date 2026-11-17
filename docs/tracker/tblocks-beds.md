# blocks / Beds (RE-REVIEW)

Target: JavaMinecraftVersion::V_26_3. Reference: `.vendor` absent, compared against the
decompiled Paper/Canvas tree in `/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft`
(`world/level/block/AbstractBedBlock.java`, `BedBlock.java`, `StrawBedBlock.java`,
`server/level/ServerPlayer.java#getBedResult`, `world/attribute/BedRule.java`).

Reviewed source change: `912463aed` (continue sleeping after kicking a villager out of a
bed, issue #3581). Re-review found that vanilla returns after the kick, so the fix was
corrected.

Fixed in this pass:

- `bed.rs:289-302` - after kicking a sleeping villager, vanilla
  `AbstractBedBlock#useWithoutItem` returns `SUCCESS_SERVER`; the player must use the bed
  again. Previously the player slept in the same interaction.
- `bed.rs:307-315` - sleep range is vanilla `ServerPlayer#isReachableBedBlock`: `|dx| <= 3`,
  `|dy| <= 2`, `|dz| <= 3`. Was `3/3/3`.
- `bed.rs:350-362` - monster scan is skipped in creative and is centred on the head only
  (`ServerPlayer#getBedResult`); previously it also scanned around the foot.

Remaining gaps:

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| `bed.rs:212-222` (`use_bed`) | `AbstractBedBlock#useWithoutItem` returns `CONSUME` unless the partner block is the same block with the opposite `part` | A lone bed half (e.g. `/setblock`) can still be slept in | Bail out when the other half does not match before evaluating the rule | S |
| `bed.rs:256-268` | `ServerPlayer#bedBlocked` uses `freeAt` (`!isSuffocating`) above head and foot | Uses `BlockState::is_solid()` instead of suffocation | Port `is_suffocating` check once available on `BlockState` | S |
| `bed.rs:270-302` vs `bed.rs:256` | `AbstractBedBlock#useWithoutItem` handles `OCCUPIED` before any range/obstruction check | Obstructed check runs first, so an occupied bed with a block above reports "obstructed" and never kicks the villager | Move the occupied/kick block above the obstruction check | S |
| `bed.rs:224,242-243` | `AbstractBedBlock#getBedRule` evaluates `EnvironmentAttributes.BED_RULE` at the bed position | `world.dimension.bed_rule` only (dimension level); `world/environment.rs:238` ignores `pos` | Per-position environment attribute lookup for dimensions with biome overrides | M |
| `crates/pumpkin-data/src/generated/dimension.rs:14-18` | `BedRule` record: `canSleep`, `canSetSpawn`, `destroyOnUse`, `destroyOnLeave`, `errorMessage` | Generated `BedRule` has only `can_sleep`, `can_set_spawn`, `explodes`; no `destroy_on_leave`/`error_message` | Extend codegen and regenerate from `assets/` (codegen change, not hand edit) | M |
| `bed.rs:388-398` | `ServerPlayer#startSleepInBed` fires `SLEPT_IN_BED` only when `canSetSpawn` | Advancement fired unconditionally | Gate the trigger on `can_set_spawn` | S |
| `bed.rs:73-86` (`can_place_at`) | `AbstractBedBlock#getStateForPlacement` also requires `level.getWorldBorder().isWithinBounds(relative)` | No world border check for the head position | Add border check when `CanPlaceAtArgs` can reach the world border | S |
| `straw_bed.rs:229-288` | `StrawBedBlock#getSleptInBedStatType` is `Stats.SLEEP_IN_STRAW_BED`; `getBedRule` uses `STRAW_BED_RULE` | Straw bed reports `SleepInBed` and reuses `dimension.bed_rule` | Add `CustomStatistic::SleepInStrawBed` and a per-block bed rule | M |

Note: `bed.rs:224-239` explosion flow matches `BedBlock#destroyOnUse` (remove both halves,
5.0 power, fire, bad respawn point damage, head centre), and it runs before the occupied
check as Paper does.
