# blocks / Beds (RE-REVIEW)

Target: JavaMinecraftVersion::V_26_3. Reference: decompiled Paper/Canvas tree in
`/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft`
(`world/level/block/AbstractBedBlock.java`, `BedBlock.java`, `StrawBedBlock.java`,
`world/entity/player/Player.java#freeAt`, `server/level/ServerPlayer.java#getBedResult`).

Reviewed source change: `1f7ea3c59` and `d05a53dc8` (bed use flow). Re-review against
`AbstractBedBlock#useWithoutItem` + `ServerPlayer#getBedResult`.

Behaviour verified correct and left alone: explosion/destroyOnUse runs first (Paper note on
line 101), occupied villager kick, `SLEEP_IN_STRAW_BED` stat for straw beds,
`destroy_after_use` on leave, drop/flags handling in `broken`, `updateShape` mirroring.

Fixed in this pass:

- `bed.rs:241`, `straw_bed.rs:264` - `AbstractBedBlock#useWithoutItem` returns `CONSUME`
  when the partner position is not the same block with the opposite `part`. Previously a
  lone half (`/setblock`) was sleepable and reported messages.
- `bed.rs:246`, `straw_bed.rs:268` - occupied check moved before range/obstruction/sleep
  rule checks; vanilla handles `OCCUPIED` before calling `startSleepInBed`.
- `bed.rs:299-319`, `straw_bed.rs:298-318` - range (3/2/3) now runs before the obstruction
  check, matching `ServerPlayer#getBedResult`; the combined canSleep/canSetSpawn early
  return was dropped (vanilla reports range/obstruction first, then `rule.asProblem()`).
- `bed.rs:322`, `straw_bed.rs:320` - obstruction uses `BlockState::is_suffocating`
  (`Player#freeAt`) instead of `is_solid`.
- `bed.rs:402`, `straw_bed.rs:398` - `SLEPT_IN_BED` advancement only fires when
  `can_set_spawn` (`ServerPlayer#startSleepInBed:2525`).
- `straw_bed.rs:334` - straw beds now set the respawn point like `getBedResult` does.
- `straw_bed.rs` - monster scan skips creative and scans the head only, like `bed.rs` and
  `getBedResult`.
- `bed.rs:73`, `straw_bed.rs:69` - placement requires the head position inside the world
  border (`AbstractBedBlock#getStateForPlacement`).

Remaining gaps:

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| `bed.rs:263-264`, `straw_bed.rs:268-269` | `AbstractBedBlock#getBedRule` evaluates `EnvironmentAttributes.BED_RULE` / `STRAW_BED_RULE` at the bed position | Uses `world.dimension.bed_rule` (dimension-level); `world/environment.rs` ignores `pos` | Per-position environment attribute lookup for biome overrides | M |
| `crates/pumpkin-data/src/generated/dimension.rs:13-18` | `BedRule` record: `canSleep`, `canSetSpawn`, `destroyOnUse`, `destroyOnLeave`, `errorMessage` | Generated `BedRule` has `can_sleep`, `can_set_spawn`, `explodes` only; no `destroy_on_leave`/`error_message` | Extend `pumpkin-codegen` + regenerate from `assets/` | M |
| `straw_bed.rs:268-269` | `StrawBedBlock#getBedEnvironmentAttribute` returns `STRAW_BED_RULE` | Straw bed reads the dimension bed rule | Needs the per-position attribute work above | M |
| `straw_bed.rs` (`getSleepHeight`) | `StrawBedBlock#getSleepHeight` lowers the sleeping height to the straw surface | Not modelled | Cosmetic/client; only relevant if sleep height is sent | S |
| `bed.rs:212`, `straw_bed.rs:235` | `ServerPlayer#getBedResult` returns `OTHER_PROBLEM` when already sleeping or dead | No explicit guard before `player.sleep` | Confirm `Player::sleep` rejects sleeping/dead players; add guard if not | S |

Note: `destroy_on_leave` for straw beds is already handled in
`crates/pumpkin-core/src/entity/player.rs:2807` via `StrawBedBlock::destroy_after_use`.
