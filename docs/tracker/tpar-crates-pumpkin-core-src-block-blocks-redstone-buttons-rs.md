# Parity rows for crates/pumpkin-core/src/block/blocks/redstone/buttons.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-bl-f6470f** [partial/OWN-OPEN] own audit: 10 open, 27 fixed, 1 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-bl-f6470f.md) docs/tracker/taudit-crates-pumpkin-core-src-bl-f6470f.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)


## Result (third pass, final)

The single tracker row `own/taudit-crates-pumpkin-core-src-bl-f6470f` points at the buttons.rs
audit. All five STILL OPEN gap rows are closed against the decompiled vanilla source under
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/` (`ButtonBlock.java`,
`AbstractCandleBlock.java`, `state/properties/BlockSetType.java`):

- arrow bounds from shape (on_scheduled_tick): VERIFIED. The state's
  `get_block_outline_shapes_at(pos)` boxes are unioned via `.at_pos(pos)` to the button's thin
  shape bounds, matching vanilla checkPressed (`ButtonBlock.java:162-165`,
  `state.getShape(level, pos).bounds().move(pos)`). API checked: `get_block_outline_shapes_at`
  (pumpkin-data/src/block_state.rs:339) returns block-local boxes; `BoundingBox::at_pos`
  (pumpkin-util/src/math/boundingbox.rs:187) adds the block position.
- game-event source and player (press_with_source): VERIFIED. A hand press passes the player; an
  arrow trigger passes the colliding arrow via `emit_game_event_with_context` with
  `GameEventContext { source_entity }`; release emits BlockDeactivate with no source, matching
  `ButtonBlock.java:107,186` (`gameEvent(player/firstArrow, ...)`, firstArrow null on release).
- onExplosionHit before removal: VERIFIED (outside this unit's editable files). explosion.rs calls
  `BlockBehaviour::explode` before `set_block_state(pos, AIR)`; the buttons.rs `explode` hook guards
  on the placed block and `can_trigger_blocks`, matching `ButtonBlock.java:90-100`.
- Paper events: VERIFIED. `BlockRedstoneEvent` fires 0 to 15 on press and 15 to 0 on release;
  `EntityInteractEvent` fires on the arrow rising edge in `on_entity_collision`, matching
  `ButtonBlock.java:80-84,94-96,168-182`. A cancelled event cancels the press/release.
- press ordering: VERIFIED. set_block_state, update_neighbors, schedule_block_tick, sound, game
  event, matching vanilla press (`ButtonBlock.java:102-108`).

New fix this pass (normal_use, buttons.rs): FIXED. `normal_use` previously returned `Consume` both
when the button was already powered and when `BlockRedstoneEvent` cancelled the press. Vanilla
`useWithoutItem` returns CONSUME only for an already-powered button and SUCCESS when the redstone
event cancels (`ButtonBlock.java:76-87`). `normal_use` now early-returns `Consume` for a powered
button and returns `Success` otherwise. No new pure seam exists (needs a live World), so no new
unit test was added; the pure `button_click_sounds_match_block_set_types` and
`stone_button_classification_matches_vanilla` tests remain.

Worker compile-pitfall re-check: no unwrap/expect/panic/todo, no println, no em dashes; no new
imports, helpers, struct initializers, moved values or unstable std; `Self` used inside the impl.
Files changed this pass: buttons.rs (one edit) and this doc. Checks not run (worker rule;
orchestrator batch-compiles); verified by source and API review only.
