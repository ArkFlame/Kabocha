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
| `bed.rs:263-264`, `straw_bed.rs:268-269` | `AbstractBedBlock#getBedRule` evaluates `EnvironmentAttributes.BED_RULE` / `STRAW_BED_RULE` at the bed position | Uses `world.dimension.bed_rule` (dimension-level); `world/environment.rs` ignores `pos` | Per-position environment attribute lookup for biome overrides | M | **BLOCKED-DATA** |
| `crates/pumpkin-data/src/generated/dimension.rs:13-18` | `BedRule` record: `canSleep`, `canSetSpawn`, `destroyOnUse`, `destroyOnLeave`, `errorMessage` | Generated `BedRule` has `can_sleep`, `can_set_spawn`, `explodes` only; no `destroy_on_leave`/`error_message` | Extend `pumpkin-codegen` + regenerate from `assets/` | M | **BLOCKED-DATA** |
| `straw_bed.rs:268-269` | `StrawBedBlock#getBedEnvironmentAttribute` returns `STRAW_BED_RULE` | Straw bed reads the dimension bed rule | Needs the per-position attribute work above | M | **BLOCKED-DATA** |
| `straw_bed.rs` (`getSleepHeight`) | `StrawBedBlock#getSleepHeight` lowers the sleeping height to the straw surface | Modelled in `bed.rs::player_sleep_height` / `sleep_shape_pos` + `player.rs` wiring | Cosmetic/client; height is sent via `set_pos`; covered by pure unit tests | S | **VERIFIED** |
| `bed.rs:212`, `straw_bed.rs:235` | `ServerPlayer#getBedResult` returns `OTHER_PROBLEM` when already sleeping or dead | No explicit guard before `player.sleep` | Confirm `Player::sleep` rejects sleeping/dead players; add guard if not | S | **FIXED** (bed.rs already guarded; straw_bed guard added)

Note: `destroy_on_leave` for straw beds is already handled in
`crates/pumpkin-core/src/entity/player.rs:2807` via `StrawBedBlock::destroy_after_use`.

## h6 gap status
- Row 5 (sleep rejected for sleeping or dead player): FIXED in tree at HEAD (`bed.rs:303` returns the problem for `is_sleeping() || !is_alive()`).
- Row 4 (straw bed sleep height): VERIFIED (see final Result); modelled in `bed.rs::player_sleep_height` + `player.rs` wiring, height is sent via `set_pos`.

## Result

- Row: straw bed getSleepHeight (h6 row 4) - FIXED. Verified against decompiled
  StrawBedBlock.java#getSleepHeight: the head half redirects to the connected
  foot's shape (4/16 = 0.25) instead of the head's pillow shape (5/16 = 0.3125).
  player.rs#sleep does send the height via set_pos, so this was observable.
  Added bed.rs::player_sleep_height (uses the foot's shape for straw beds, the
  head's for other beds) and wired player.rs#sleep to it. Pure unit test added.
- Rows 39-41 (per-position bed rule / generated BedRule record) were not marked
  STILL OPEN and were left alone; they need generated-data/codegen changes.
- Files touched: crates/pumpkin-core/src/block/blocks/bed.rs,
  crates/pumpkin-core/src/entity/player.rs. Ran rustfmt --edition 2024 on both.
- Did not compile (per task: no cargo); orchestrator verifies.


## Result (re-review verification)

- Row 42 / h6 row 4 (straw bed getSleepHeight): verified STILL OPEN fix is already
  present and correct in the tree, no new edit needed. The head-half redirect in
  bed.rs::sleep_shape_pos + player_sleep_height matches vanilla
  StrawBedBlock.java#getSleepHeight (head -> connected foot via
  getConnectedDirection = facing.getOpposite; super uses the foot shape). The
  foot shape max Y (4/16 = 0.25) is lower than the head pillow shape (5/16 =
  0.3125) and is observable because player.rs#sleep sends the height via set_pos.
- SLEEP_HEIGHT_OFFSET = 0.125 verified against vanilla LivingEntity.java#setPosToBed
  (adjustHeight = 0.125). Pure unit tests already present in bed.rs.
- Row 43 / h6 row 5 (sleeping/dead guard) remains FIXED as previously recorded.
- Rows 39-41 stay open (not marked STILL OPEN): need pumpkin-data BedRule codegen
  and per-position environment attribute lookup; generated files are off-limits.
- No source edits this pass (fix pre-existing). Rustfmt previously applied to
  bed.rs; no .rs file touched this pass. Did not compile (no cargo per task).

## Result (fresh verification pass)

- Only row marked STILL OPEN is row 42 (straw bed `getSleepHeight`). Verified it is
  already fixed and correct in the tree; no new source edit needed.
- Vanilla evidence: `StrawBedBlock.java#getSleepHeight:63-78` redirects a HEAD half
  to the connected FOOT (`getConnectedDirection(state)`, i.e. facing.getOpposite),
  returns empty if that state is not the same block's FOOT, then calls
  `super.getSleepHeight(footState)`; FOOT shape max Y = 4/16 = 0.25, HEAD pillow
  shape max Y = 5/16 = 0.3125. `AbstractBedBlock.java#getSleepHeight:71-77` returns
  `state.getShape(level, pos).max(Y)`. `LivingEntity.java#setPosToBed:4812-4813`
  adds `adjustHeight = 0.125`.
- Pumpkin: `bed.rs:581` `SLEEP_HEIGHT_OFFSET = 0.125`, `sleep_height()` (shape max
  Y), `sleep_shape_pos()` (straw HEAD -> opposite/foot), `player_sleep_height()`,
  and pure unit tests `sleep_height_plus_offset_is_the_bed_top` and
  `straw_bed_sleep_shape_is_the_lower_foot_surface`. Wiring at
  `entity/player.rs:2819-2824` sends the height via `set_pos`. All present.
- Rows 39-41 (per-position bed rule / generated BedRule record / straw STRAW_BED_RULE)
  are not marked STILL OPEN: they need pumpkin-data codegen + regeneration and a
  cross-crate per-position environment-attribute lookup (generated files off-limits,
  outside this unit). Left open.
- No source files edited this pass; only this doc. Did not compile (no cargo per task).


## Result (third pass - final resolution)

- Row 42 / h6 row 4 (straw bed `getSleepHeight`): VERIFIED, matches vanilla. The fix is
  already present in the tree: `bed.rs::player_sleep_height` (lines 606-613) redirects a
  straw-bed HEAD to the connected FOOT via `sleep_shape_pos` (`facing.opposite()`), then
  reads the foot's collision-shape top (4/16 = 0.25); non-straw beds use the head shape
  (9/16 = 0.5625). `entity/player.rs:2832-2837` sends the height via `set_pos` with
  `SLEEP_HEIGHT_OFFSET = 0.125`. Vanilla evidence:
  `.vendor/vanilla-decompiled/net/minecraft/world/level/block/StrawBedBlock.java:63-78`
  (HEAD -> `pos.relative(getConnectedDirection(state))` FOOT, else empty, then
  `super.getSleepHeight`), `AbstractBedBlock.java:71-78` (`shape.max(Direction.Axis.Y)`),
  and `LivingEntity` setPosToBed adds `adjustHeight = 0.125`. Pure unit tests
  `sleep_height_plus_offset_is_the_bed_top` and
  `straw_bed_sleep_shape_is_the_lower_foot_surface` in `bed.rs` cover it. No source edit
  needed this pass.
- Row 39 (per-position bed rule / biome overrides): BLOCKED-DATA. Requires a per-position
  `EnvironmentAttribute<BedRule>` lookup with biome overrides;
  `world/environment.rs::get_value_bed_rule` ignores `pos`, and generated
  `crates/pumpkin-data/src/generated/biome.rs` exposes only `boolean_attributes` (no
  bed-rule attribute value to read at a position). Vanilla: `AbstractBedBlock.java:63-65`
  `getBedRule` calls `level.environmentAttributes().getValue(attr, pos)`.
- Row 40 (BedRule record fields): BLOCKED-DATA. Generated
  `crates/pumpkin-data/src/generated/dimension.rs:14-18` `BedRule` has only `can_sleep`,
  `can_set_spawn`, `explodes`; missing fields `destroy_on_leave` and `error_message` plus
  `asProblem()` from vanilla
  `.vendor/vanilla-decompiled/net/minecraft/world/attribute/BedRule.java:12` (record
  `BedRule(canSleep, canSetSpawn, destroyOnUse, destroyOnLeave, errorMessage)`). Needs
  codegen extension and regeneration from `assets/`.
- Row 41 (straw bed STRAW_BED_RULE): BLOCKED-DATA. Generated
  `crates/pumpkin-data/src/generated/environment_attribute.rs` has `GameplayBedRule` but
  no `GameplayStrawBedRule` variant; vanilla registers `STRAW_BED_RULE` at
  `.vendor/vanilla-decompiled/net/minecraft/world/attribute/EnvironmentAttributes.java:141-142`.
  Also depends on the per-position lookup from row 39.
- Row 43 (sleeping/dead guard): unchanged FIXED as previously recorded.
- No source files edited this pass (row 42 fix pre-existing and verified against
  `.vendor/vanilla-decompiled`); only this doc was updated, so rustfmt is not applicable.
  Did not compile (no cargo per task); orchestrator verifies.
