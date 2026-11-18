# blocks / Beds (RE-REVIEW)

Target: JavaMinecraftVersion::V_26_3. Reference: decompiled Paper/Canvas tree in
`/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft`
(`world/level/block/AbstractBedBlock.java`, `BedBlock.java`, `StrawBedBlock.java`,
`world/entity/LivingEntity.java#stopSleeping`, `server/level/ServerPlayer.java#getBedResult`).

Re-reviewed after `1f7ea3c59`/`d05a53dc8` (use flow) and the new bunk-bed stand-up code in
`crates/pumpkin-core/src/block/blocks/bed.rs`.

Verified correct and left alone: use flow partner/occupied/range/obstruction/canSetSpawn/
canSleep/monster order matches `getBedResult` (spawn point is set before the canSleep
failure, as in `ServerPlayer.java:2469-2477`); explosion-first Paper ordering; straw-bed
stat; `destroy_after_use` on player leave; drop/flags in `broken`; `updateShape` mirroring;
`find_stand_up_position` matches `AbstractBedBlock.java:226-284` (right/side selection,
bunk branch, 6-step offset search order, offsets at 313-330); `is_bunk_bed` tag check equals
`instanceof AbstractBedBlock`.

Fixed in this pass:

- `bed.rs:301-306` - `ServerPlayer#getBedResult:2458` returns `OTHER_PROBLEM` for a player
  already sleeping or dead; now guarded before range/spawn/sleep checks.
- `player.rs:2824-2850` - `LivingEntity#stopSleeping:4818` repositions the waking player at
  `findStandUpPosition` (or above+0.1) and turns them toward the bed; previously the player
  woke at the old sleeping position with unchanged yaw.
- `villager/sleep.rs:66-72` - vanilla `Villager#stopSleeping` calls
  `AbstractBedBlock#onStopSleeping`, so `destroyOnLeave` straw beds are removed; the villager
  path only cleared `occupied`.

Remaining gaps:

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| `bed.rs:263-264`, `straw_bed.rs:268-269` | `AbstractBedBlock#getBedRule` evaluates `EnvironmentAttributes.BED_RULE` / `STRAW_BED_RULE` at the bed position | Uses `world.dimension.bed_rule` (dimension-level); `world/environment.rs` ignores `pos` | Per-position environment attribute lookup for biome overrides | M | STILL OPEN (per-position environment attribute lookup; M, not attempted) |
| `crates/pumpkin-data/src/generated/dimension.rs:13-18` | `BedRule` record has `canSleep`, `canSetSpawn`, `destroyOnUse`, `destroyOnLeave`, `errorMessage` | Generated `BedRule` has `can_sleep`, `can_set_spawn`, `explodes` only; no `destroy_on_leave`/`error_message` | Extend `pumpkin-codegen` + regenerate from `assets/` | M | STILL OPEN (needs pumpkin-codegen change; generated code not hand-edited) |
| `bed.rs:352-361` | `ServerPlayer#getBedResult:2476` returns `rule.asProblem()`, the per-dimension message | Hardcoded `BLOCK_MINECRAFT_BED_NO_SLEEP`; nether/respawn-anchor style messages and `BedRule.errorMessage` ignored | Needs the `error_message` field above | M | STILL OPEN (blocked on BedRule error_message codegen; M) |
| `bed.rs` failure paths (307-386) | Paper `AbstractBedBlock:112-135` fires `PlayerBedFailEnterEvent` for each `BedSleepingProblem` and allows plugins to override/explode | Only `PlayerBedEnterEvent` is fired, and only on the success path | Add the fail event on each early return | M | STILL OPEN (PlayerBedFailEnterEvent not attempted; M) |
| `player.rs:2658` | `LivingEntity#setPosToBed:4801` uses `AbstractBedBlock#getSleepHeight`; `StrawBedBlock#getSleepHeight:63` resolves head->foot and returns the straw surface height | Player sleep height is hardcoded `0.6875`; villager `get_sleep_height` uses the raw state's max collision height without head->foot resolution | Shared `BedBlock::get_sleep_height(state, world, pos)` helper | S | FIXED 8c2003a3e (player and villager use bed collision top + 0.125; straw head->foot resolution UNVERIFIED) |


## Result

Verified all four STILL OPEN rows against decompiled vanilla 26.3 (`AbstractBedBlock.java`,
`BedRule.java`, `BedBlock.java`) and Paper's `PlayerBedFailEnterEvent`. Every one needs work
outside this unit (generated data / plugin API), so no source file was changed this pass.

- `bed.rs:263-264` / `straw_bed.rs:268-269` (per-position bed rule): OPEN - vanilla
  `AbstractBedBlock#getBedRule` calls `environmentAttributes().getValue(BED_RULE, pos)`, but the
  generated `Biome` stores only `boolean_attributes` (no BedRule compound), so supporting a biome
  override needs a pumpkin-codegen change plus regenerated data. No vanilla biome overrides
  `bed_rule`, so the current dimension-level value is behaviourally equivalent for vanilla data.
- `crates/pumpkin-data/src/generated/dimension.rs` (BedRule fields): OPEN - generated file is
  off-limits; adding `destroy_on_leave`/`error_message` requires extending pumpkin-codegen and
  regenerating from `assets/`.
- `bed.rs:352-361` (rule error message): OPEN - vanilla `BedRule#asProblem` sends
  `BedRule.errorMessage()`; blocked on adding the `error_message` field to generated `BedRule`
  (codegen), so the hardcoded `BLOCK_MINECRAFT_BED_NO_SLEEP` stays.
- `bed.rs` failure paths (`PlayerBedFailEnterEvent`): OPEN - no such event exists in Pumpkin;
  adding it requires a new WIT event type plus plugin-api/host/SDK bindings (cross-crate,
  outside this unit).

Rechecked row 38 (already FIXED): `player_sleep_height`/`sleep_shape_pos` in `bed.rs` and the
`straw_bed_sleep_shape_is_the_lower_foot_surface` unit test already cover the straw head->foot
resolution; no change needed.

## Result

Re-review pass 2 against decompiled vanilla 26.3 (`AbstractBedBlock.java`,
`BedRule.java`, `StrawBedBlock.java`, `EnvironmentAttributes.java:139-142`,
`ServerPlayer.java:2455-2498`) and Paper's `PlayerBedFailEnterEvent`. No source file was
changed this pass: every remaining gap needs generated data / WIT bindings / the off-limits
generated file, all outside this unit.

- Row 1 (`bed.rs:267,284-285` / `straw_bed.rs:272-273` / `environment.rs:228`): OPEN -
  vanilla evaluates `BED_RULE`/`STRAW_BED_RULE` per position, but generated `Biome` stores
  only `boolean_attributes`; biome overrides need pumpkin-codegen + regenerated data. No
  vanilla biome overrides either rule, so the dimension-level value is behaviourally
  equivalent; `get_value_bed_rule(pos)` is the correct seam for a future codegen change.
- Row 2 (`pumpkin-data/src/generated/dimension.rs:14-18`): OPEN - generated file is off-limits;
  `destroy_on_leave` / `error_message` need pumpkin-codegen and regeneration from `assets/`.
- Row 3 (`bed.rs:389-402`): OPEN - vanilla sends `BedRule.errorMessage`, which in vanilla data
  is only ever `block.minecraft.bed.no_sleep`, already hardcoded as
  `BLOCK_MINECRAFT_BED_NO_SLEEP`; full fidelity blocked on adding `error_message` via codegen.
- Row 4 (`bed.rs` failure paths): FIXED in current tree - `fire_bed_fail_event` fires
  `PlayerBedFailEnterEvent` on TooFarAway/Obstructed/NoSleep/NotSafe and honours `cancelled()`
  via `resolve_fail_message`; explode-first ordering preserved (bed.rs:267). Verified against
  `AbstractBedBlock.java:112-135` and `ServerPlayer.java:2455-2498`. Verified, no change.

Also rechecked row 38 (already FIXED): `sleep_height` / `player_sleep_height` in bed.rs plus
`straw_bed_sleep_shape_is_the_lower_foot_surface` cover the head->foot straw resolution;
sleep.rs consumes the shared helper. No change.
