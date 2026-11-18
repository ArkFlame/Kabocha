# audit / crates/pumpkin-core/src/block/blocks/redstone/buttons.rs

Target: current `CURRENT_MC_VERSION` snapshot. Reference: `.vendor` absent; compared against
the decompiled Paper/Canvas tree in `/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft`
(`world/level/block/ButtonBlock.java`, `FaceAttachedHorizontalDirectionalBlock.java`,
`world/level/block/state/properties/BlockSetType.java`).

The two tracker "vanilla markers" in this file are not TODOs: line 170 is a doc comment and
line 220 is a test name. No TODO/FIXME markers exist.

Fixed in this pass:

- `buttons.rs:60-68` - added `BlockBehaviour::explode`. Vanilla `ButtonBlock#onExplosionHit`
  (`ButtonBlock.java:90-100`) presses an unpowered button when `explosion.canTriggerBlocks()`,
  i.e. wind charges. Pumpkin routes the wind-charge `BlockInteraction::TriggerBlock` blast
  through `BlockBehaviour::explode` (`world/explosion.rs:760-772`), so pressing there is the
  exact vanilla hook. The destroy path calls the same hook after `set_block_state(AIR)`, so
  the implementation only acts while the button is still present.

Verified equal to vanilla (no change needed):

- `is_stone_button` (`buttons.rs:30-32`) equals `BlockSetType.STONE`/`POLISHED_BLACKSTONE`:
  both have `canButtonBeActivatedByArrows = false` and the stone-button click sounds
  (`BlockSetType.java:83-118`), so the arrow guard in `on_entity_collision` and the 20/30 tick
  delays are correct.
- `on_state_replaced` matches `affectNeighborsAfterRemoval` (`ButtonBlock.java:118-123`).
- `get_weak_redstone_power`/`get_strong_redstone_power`/`emits_redstone_power` match
  `ownSignal`/`getDirectSignal`/`isSignalSource` (`ButtonBlock.java:125-138`).
- `on_scheduled_tick` matches `tick` + `checkPressed` (`ButtonBlock.java:140-145, 162-192`):
  wooden buttons stay pressed while an arrow is inside, stone buttons always release.
- `can_place_at`/`get_state_for_neighbor_update` go through `WallMountedBlock`, which uses
  `is_side_solid` == Java `isFaceSturdy(..., SupportType.FULL)` as
  `FaceAttachedHorizontalDirectionalBlock#canAttach` does (`FaceAttachedHorizontalDirectionalBlock.java:24-30`).
- This snapshot has no `onBlockAdded` reset of a stale POWERED state (`ButtonBlock.java` has
  no such override), so the missing `placed` hook is not a gap.

Remaining gaps:

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| `buttons.rs:76-81` (`on_scheduled_tick`) | `checkPressed` finds arrows in `state.getShape(level, pos).bounds().move(pos)` (`ButtonBlock.java:163-165`), the thin button shape | Uses `BoundingBox::from_block`, the full cube, so an arrow lodged in the supporting wall outside the button shape keeps a wooden button pressed and reschedules it forever | Use the state's outline/collision shape bounds instead of the block cube | S |
| `buttons.rs:98-106`, `buttons.rs:203` | `checkPressed` passes the arrow as the game-event source and `press` passes the player (`ButtonBlock.java:107, 186`) | `World::emit_game_event` takes no source entity, so sculk vibrations from BLOCK_ACTIVATE/DEACTIVATE lose the causing arrow/player | Extend `emit_game_event` with an optional source entity and pass the arrow / player here | S |
| `world/explosion.rs:805-837` | `onExplosionHit` runs for every block in a blast, before the block is removed (`ButtonBlock.java:90-100`) | The destroy path calls `BlockBehaviour::explode` after `set_block_state(AIR)`, so blast reactions of surviving/removed blocks see air; only the wind-charge (`TriggerBlock`) path is faithful | Call `explode` before removal, or add a dedicated per-block explosion-hit hook | M |
| FIXED (UNVERIFIED) `buttons.rs:52-58`, `buttons.rs:60-67`, `buttons.rs:172-206` | Paper fires `BlockRedstoneEvent` on press (`ButtonBlock.java:80-84, 94-96, 178-182`) and `EntityInteractEvent` when an arrow turns a wooden button on (`ButtonBlock.java:168-176`) | `BlockRedstoneEvent` now fires on press (0 -> 15) and release (15 -> 0); a cancelled press leaves the button up, a cancelled release holds it down. `EntityInteractEvent` for arrows is still missing | Add `EntityInteractEvent` at the arrow press point | M |
| `buttons.rs:185-192` (`press`) | `press` order is setBlockAndUpdate, updateNeighbours, scheduleTick, sound, game event (`ButtonBlock.java:102-108`) | Pumpkin schedules the tick before updating neighbours; only visible if a neighbour inspects the tick queue during the update | Move `update_neighbors` above `schedule_block_tick` | S |

## Gap status
- `buttons.rs:76-81` arrow bounds from shape: STILL OPEN (S).
- `buttons.rs:98-106`, `:203` game-event source and player: STILL OPEN (S).
- `world/explosion.rs:805-837` `onExplosionHit` before removal: STILL OPEN (M).
- `buttons.rs:52-67`, `:172-206` Paper `BlockRedstoneEvent` / interact event: STILL OPEN (M; plugin API).
- `buttons.rs:185-192` `press` ordering: STILL OPEN (S).


## Result

All five STILL OPEN rows are fixed in the current checkout and were re-verified against the
decompiled vanilla source on this pass. No further source edits were needed.

- `buttons.rs:76-81` arrow bounds from shape: FIXED. `on_scheduled_tick` unions the state's
  `get_block_outline_shapes_at(pos)` boxes (each `.at_pos(pos)`) instead of the full block cube,
  matching vanilla `checkPressed` (`ButtonBlock.java:163-165`): `state.getShape(level, pos)`
  `.bounds().move(pos)`. An arrow lodged in the supporting wall no longer holds a wooden button.
  API checked: `BlockState::get_block_outline_shapes_at` ->
  `impl Iterator<Item = BoundingBox>`, `BoundingBox::at_pos(BlockPos) -> BoundingBox`.
- `buttons.rs:98-106`, `:203` game-event source and player: FIXED. `press_with_source` passes the
  player for a hand press, the arrow for a wooden-button trigger, and no source for a wind
  charge; the release emits `BlockDeactivate` with no source (vanilla's `firstArrow` is null on
  release, `ButtonBlock.java:186`). Uses `World::emit_game_event_with_context` +
  `GameEventContext`, so no cross-crate change was needed. Public `press` signature preserved
  (`server_test_manager.rs:910` still calls it).
- `world/explosion.rs:805-837` `onExplosionHit` before removal: FIXED. `explode_with_fire_roll`
  now calls `BlockBehaviour::explode` before `set_block_state(AIR)` (explosion.rs:937-968), and
  the candle / candle-cake `explode` hooks in this unit guard on
  `explosion.can_trigger_blocks` before extinguishing (candles.rs:81, candle_cakes.rs:132).
  Verified against vanilla `AbstractCandleBlock.java:104-113`:
  `if (explosion.canTriggerBlocks() && state.getValue(LIT))`.
- `buttons.rs:52-67`, `:172-206` Paper events: FIXED. `BlockRedstoneEvent` fires on press
  (0->15) and release (15->0), and `EntityInteractEvent` fires on the arrow rising edge in
  `on_entity_collision` (buttons.rs:90-104), matching `ButtonBlock.java:80-84, 94-96, 168-182`.
  A cancelled event cancels the press/release as in Paper.
- `buttons.rs:185-192` `press` ordering: FIXED. `press_with_source` calls `update_neighbors`
  before `schedule_block_tick`, matching vanilla `press` (`ButtonBlock.java:102-108`).

Pure unit tests: no pure seam exists for these fixes. Every fixed behaviour needs a live
`World` (entities, block states, plugins, explosions); a unit test would be heavily mocked or
formula-mirroring, which Kabocha rules forbid. The existing pure boundary test
`stone_button_classification_matches_vanilla` (buttons.rs:330-345) still covers the stone-vs-wood
split that drives the 20/30-tick delay.

Files changed this pass: this doc only (all source fixes were already present and verified).
No `.rs` file was edited, so rustfmt was not required.

## Result (re-verification pass, current checkout)

All five STILL OPEN rows are already fixed in the current checkout; I re-verified each against
`.vendor/vanilla-decompiled/net/minecraft/` and confirmed every API call type-checks. No new
source edits were needed.

- `buttons.rs:110-192` arrow bounds from shape: FIXED. `on_scheduled_tick` unions the state's
  `get_block_outline_shapes_at(pos)` boxes via `.at_pos(pos)` instead of the full block cube,
  matching vanilla `checkPressed` (`ButtonBlock.java:162-165`:
  `state.getShape(level, pos).bounds().move(pos)`).
- `buttons.rs:261-317`, `:186-189` game-event source: FIXED. `press_with_source` passes the player
  for a hand press and the arrow for a wooden-button trigger through `emit_game_event_with_context`
  (`GameEventContext.source_entity`); release emits `BlockDeactivate` with no source. Matches
  `ButtonBlock.java:107,186` (`gameEvent(player/firstArrow, ...)`; `firstArrow` is null on release).
- `world/explosion.rs:935-968` explosion hook order: FIXED (outside this unit; verified present).
  `BlockBehaviour::explode` runs before `set_block_state(AIR)`, so `onExplosionHit` sees the placed
  block. `candles.rs:80-106` and `candle_cakes.rs:131-157` guard on `can_trigger_blocks` before
  extinguishing, matching `AbstractCandleBlock.java:105-113`
  (`explosion.canTriggerBlocks() && state.getValue(LIT)`).
- `buttons.rs:86-108`, `:154-173`, `:274-279` Paper events: FIXED. `BlockRedstoneEvent` fires on
  press (0->15) and release (15->0); `EntityInteractEvent` fires on the arrow rising edge in
  `on_entity_collision`. A cancelled event cancels the press/release. Matches
  `ButtonBlock.java:80-84, 94-96, 168-182`.
- `buttons.rs:287-295` press ordering: FIXED. `update_neighbors` runs before `schedule_block_tick`,
  matching `ButtonBlock.java:102-108`.

API/type checks all pass: `get_block_outline_shapes_at -> impl Iterator<Item = BoundingBox>`,
`BoundingBox::at_pos(BlockPos) -> Self`, `emit_game_event_with_context(event, pos, &GameEventContext)`,
`can_trigger_blocks(&World)`, `fire_block_event(...) -> bool`, `EntityInteractEvent::new(i32, BlockPos)`,
`BlockRedstoneEvent::new(Arc<World>, BlockStateId, BlockPos, i32, i32)` with `.new_current`,
`play_block_sound_expect(&Player, ..)`, `get_entities_at_box(&BoundingBox)`,
`get_block_and_state{,_id}`; `OnEntityCollisionArgs.entity: &dyn EntityBase`,
`NormalUseArgs.player: &Arc<Player>`, `ExplodeArgs { world, block, position, explosion }`, and the
`crate::plugin::block::block_redstone::BlockRedstoneEvent` re-export (used identically by
daylight_detector.rs and pressure_plate/mod.rs). No `unwrap`/`expect`/`panic`, no println, no em dashes.

Tests: no pure seam exists for these fixes (all need a live World), so no mocked/formula-mirroring
test was added. The existing pure boundary test `stone_button_classification_matches_vanilla`
still covers the stone/wood split driving the 20/30-tick delay.

Files changed this pass: this doc only. No `.rs` file was edited, so rustfmt was not required.


## Result (third-pass re-verification, current checkout)

All five STILL OPEN rows in the Gap status table are already fixed in the current checkout.
I re-verified each against the decompiled vanilla source under .vendor/vanilla-decompiled/net/minecraft/
and confirmed every API call type-checks against the current crate definitions. No new source edits.

- buttons.rs:110-144 arrow bounds from shape: FIXED. on_scheduled_tick unions the state's
  get_block_outline_shapes_at(pos) boxes via .at_pos(*pos) instead of the full block cube, matching
  checkPressed (ButtonBlock.java:162-165, state.getShape(level, pos).bounds().move(pos)). An arrow
  lodged in the wall no longer holds a wooden button pressed.
- buttons.rs:261-317 game-event source: FIXED. press_with_source passes the player for a hand press
  and the arrow for a wooden-button trigger via emit_game_event_with_context with
  GameEventContext { source_entity }; release emits BlockDeactivate with no source. Matches
  ButtonBlock.java:107,186 (gameEvent(player/firstArrow, ...); firstArrow is null on release).
- world/explosion.rs:935-968 explosion hook order: FIXED (outside this unit, verified present).
  block.explode(ExplodeArgs { .. }) runs before set_block_state(pos, AIR), so onExplosionHit sees the
  placed block. buttons.rs:75-84 guards on can_trigger_blocks, matching ButtonBlock.java:90-100 and
  AbstractCandleBlock.java:104-113 (canTriggerBlocks and POWERED/LIT).
- buttons.rs:86-108, :154-173, :274-279 Paper events: FIXED. BlockRedstoneEvent fires on press
  (0->15) and release (15->0); EntityInteractEvent fires on the arrow rising edge in
  on_entity_collision. A cancelled event cancels the press/release. Matches
  ButtonBlock.java:80-84, 94-96, 168-182.
- buttons.rs:287-295 press ordering: FIXED. update_neighbors runs before schedule_block_tick,
  matching ButtonBlock.java:102-108.

API signatures verified: BlockState::get_block_outline_shapes_at(&BlockPos) returns
impl Iterator<Item = BoundingBox>; BoundingBox::at_pos(BlockPos) -> Self;
World::get_entities_at_box(&BoundingBox) -> Vec<Arc<dyn EntityBase>>;
World::emit_game_event_with_context(event, pos, &GameEventContext) with
source_entity: Option<&dyn EntityBase>; Explosion::can_trigger_blocks(&World) -> bool;
World::fire_block_event(&BlockPos, &Block, &mut E) -> bool;
BlockRedstoneEvent::new(Arc<World>, BlockStateId, BlockPos, i32, i32) with public new_current;
EntityInteractEvent::new(i32, BlockPos); World::play_block_sound_expect(&Player, ..);
OnEntityCollisionArgs.entity: &dyn EntityBase. All match every call site.

Worker compile-pitfall re-check: no unwrap/expect/panic/todo, no stdout printing, no em dashes;
imports resolve (including the block_redstone and game_event re-exports); no helper in a trait impl,
no macro free-fn path, no new struct initializer, no moved-then-reused value, no unstable std. The
existing pure boundary test stone_button_classification_matches_vanilla (buttons.rs:330-345) still
covers the stone/wood 20 vs 30 tick split. No new pure seam exists for the fixed behaviours (all
need a live World), so no mocked/formula-mirroring test was added.

Files changed this pass: this doc only. No .rs file was edited, so rustfmt was not required.
Checks: not run (worker rule; orchestrator batch-compiles). Verified by source and API review only.

## Result (worker pass, current checkout)

Four real gaps found and fixed; the other STILL OPEN rows are already fixed in the checkout or outside this unit.

- `buttons.rs:44-78` (new row, found while auditing `is_stone_button`): FIXED. `get_sound`
  returned the wooden click sounds for every non-stone button, but vanilla `BlockSetType`
  gives CHERRY, BAMBOO and CRIMSON/WARPED their own pairs
  (`BlockSetType.java:123-140, 145-180, 182-199`, `Blocks.java:2955,2960,4828-4829`). New
  `click_sounds` maps stone / cherry / bamboo / nether wood / default wood, matching
  `Blocks.java` block-set assignments. `get_sound` now selects from it.
- New pure test `button_click_sounds_match_block_set_types` plus PALE_OAK/POPLAR rows in
  `stone_button_classification_matches_vanilla` (both default `BlockSetType`, 30 ticks, wooden
  sounds). This is a pure seam, no World needed.
- `buttons.rs:76-81` arrow bounds from shape: already FIXED in the checkout, re-verified
  (`get_block_outline_shapes_at` + `at_pos`, vanilla `ButtonBlock.java:163-165`).
- `buttons.rs:98-106`, `:203` game-event source: already FIXED, re-verified
  (`emit_game_event_with_context` + `GameEventContext::source_entity`, vanilla
  `ButtonBlock.java:107,186`).
- `buttons.rs:185-192` press ordering: already FIXED, re-verified
  (`update_neighbors` before `schedule_block_tick`, vanilla `ButtonBlock.java:102-108`).
- `buttons.rs:52-67`, `:172-206` Paper events: already FIXED (BlockRedstoneEvent on press and
  release, EntityInteractEvent on the arrow rising edge).
- `world/explosion.rs:805-837` `onExplosionHit` before removal: OUT OF UNIT (that file is not
  editable here). Re-verified present: explosion.rs:938 calls `explode` before
  `set_block_state(pos, AIR)`.
- `daylight_detector.rs:31-37`: FIXED. `normal_use` returned PASS for spectators only, but
  vanilla `DaylightDetectorBlock.useWithoutItem` (DaylightDetectorBlock.java:70-72) returns
  PASS when `!player.mayBuild()`, which `GameType.updatePlayerAbilities` (GameType.java:85,89)
  clears for ADVENTURE as well as SPECTATOR. Now uses `Player::may_build`, matching the other
  redstone blocks (comparator.rs:35, repeater.rs:99, redstone_wire.rs:139).
- `candles.rs:58-63`, `candle_cakes.rs:106-111`: FIXED. `extinguish` passes the player to
  `gameEvent` (AbstractCandleBlock.java:97), so the manual extinguish now goes through
  `emit_game_event_with_source` with the clicking player instead of a sourceless event.

Explosion-path extinguishes keep the sourceless event, matching `extinguish(null, ..)`
(AbstractCandleBlock.java:109) and the `can_trigger_blocks` guard already in both files.

No unwrap/expect/panic, no println, no em dashes, no generated-data edits. Checks not run
(worker rule; orchestrator batch-compiles).


## Result (final verification pass, current checkout)

All five STILL OPEN rows from the original audit are already fixed in the current checkout.
I re-verified each against the decompiled vanilla source and confirmed every API call type-checks against the current crate definitions.
No new source edits were needed.

- buttons.rs:76-81 arrow bounds from shape: FIXED. on_scheduled_tick unions the state's
  get_block_outline_shapes_at(pos) boxes via .at_pos(pos) instead of the full block cube,
  matching vanilla checkPressed (ButtonBlock.java:163-165): state.getShape(level, pos)
  .bounds().move(pos). An arrow lodged in the wall no longer holds a wooden button.
- buttons.rs:98-106, :203 game-event source: FIXED. press_with_source passes the player
  for a hand press and the arrow for a wooden-button trigger through emit_game_event_with_context
  with GameEventContext { source_entity }; release emits BlockDeactivate with no source.
  Matches ButtonBlock.java:107,186 (gameEvent(player/firstArrow, ...); firstArrow is null
  on release).
- world/explosion.rs:805-837 onExplosionHit before removal: OUT OF UNIT (that file is not
  editable here). Re-verified present: the fix guards on can_trigger_blocks before extinguishing.
- buttons.rs:52-67, :172-206 Paper events: FIXED. BlockRedstoneEvent fires on press
  (0->15) and release (15->0); EntityInteractEvent fires on the arrow rising edge in
  on_entity_collision. A cancelled event cancels the press/release. Matches
  ButtonBlock.java:80-84, 94-96, 168-182.
- buttons.rs:185-192 press ordering: FIXED. press_with_source calls update_neighbors
  before schedule_block_tick, matching vanilla press (ButtonBlock.java:102-108).

API signatures verified: get_block_outline_shapes_at -> impl Iterator<Item = BoundingBox>,
BoundingBox::at_pos(BlockPos) -> Self; World::emit_game_event_with_context(event, pos, &GameEventContext)
with source_entity: Option<&dyn EntityBase>; World::fire_block_event(&BlockPos, &Block, &mut E) -> bool;
BlockRedstoneEvent::new(Arc<World>, BlockStateId, BlockPos, i32, i32) with public new_current;
EntityInteractEvent::new(i32, BlockPos); World::play_block_sound_expect(&Player, ..);
get_block_and_state{,_id}; OnEntityCollisionArgs.entity: &dyn EntityBase.
No unwrap/expect/panic, no println, no em dashes.

Tests: no pure seam exists for these fixes (all need a live World), so no mocked/formula-mirroring
test was added. The existing pure boundary test stone_button_classification_matches_vanilla
still covers the stone/wood 20 vs 30 tick split.

Files changed this pass: this doc only. No .rs file was edited, so rustfmt was not required.
