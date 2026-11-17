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
| `buttons.rs:52-58`, `buttons.rs:60-67`, `buttons.rs:172-206` | Paper fires `BlockRedstoneEvent` on press (`ButtonBlock.java:80-84, 94-96, 178-182`) and `EntityInteractEvent` when an arrow turns a wooden button on (`ButtonBlock.java:168-176`) | Pumpkin's plugin API has no equivalent redstone-change/entity-interact hook, so plugins cannot veto a button press | Add the plugin events and fire them at the same points | M |
| `buttons.rs:185-192` (`press`) | `press` order is setBlockAndUpdate, updateNeighbours, scheduleTick, sound, game event (`ButtonBlock.java:102-108`) | Pumpkin schedules the tick before updating neighbours; only visible if a neighbour inspects the tick queue during the update | Move `update_neighbors` above `schedule_block_tick` | S |
