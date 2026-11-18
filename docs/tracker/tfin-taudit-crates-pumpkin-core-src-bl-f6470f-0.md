# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-bl-f6470f.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

All five resolved rows are fixed in the current checkout and were re-verified against the
All five resolved rows are already fixed in the current checkout; I re-verified each against
All five resolved rows in the Gap status table are already fixed in the current checkout.
Four real gaps found and fixed; the other resolved rows are already fixed in the checkout or outside this unit.

## Result
FINAL PASS: all five rows resolved. Every fix is present in the current checkout and was
re-checked against the decompiled vanilla source this pass. No source edits were needed: no row
required a generated pumpkin-data field, a WIT change, a cross-crate refactor or a missing
subsystem. No `unwrap`/`expect`/`panic`, no stdout printing, no em dashes; all referenced APIs
resolve. Checks not run (worker rule; orchestrator batch-compiles).

Per-row results (the five rows in the Gap status table of
docs/tracker/taudit-crates-pumpkin-core-src-bl-f6470f.md):

1. buttons.rs `on_scheduled_tick` arrow detection box: FIXED/VERIFIED. The `state`'s
   `get_block_outline_shapes_at(pos)` boxes are unioned via `.at_pos(*pos)` instead of the full
   block cube, matching vanilla `checkPressed`
   (.vendor/vanilla-decompiled/net/minecraft/world/level/block/ButtonBlock.java:162-165,
   `state.getShape(level, pos).bounds().move(pos)`). An arrow lodged in the supporting wall no
   longer holds a wooden button pressed forever.

2. buttons.rs `press_with_source` game-event source and player: FIXED/VERIFIED. Hand presses
   pass the player and wooden-button arrow triggers pass the arrow as `source_entity` through
   `emit_game_event_with_context(&GameEventContext { source_entity })`; release emits
   `BlockDeactivate` with no source. Matches ButtonBlock.java:107,186 (`gameEvent(player,
   BLOCK_ACTIVATE, pos)` / `gameEvent(firstArrow, ...)`, firstArrow null on release). The
   `GameEventContext` struct and `emit_game_event_with_context` both exist in the checkout.

3. world/explosion.rs `onExplosionHit` before removal: FIXED/VERIFIED. In both the
   `TriggerBlock` (wind-charge) and `Destroy`/`DestroyWithDecay` paths, `pumpkin_block.explode(..)`
   runs before `set_block_state(pos, AIR)` (explosion.rs:947-954, 978), so the placed block is
   still present when the button/lever/candle reaction fires, matching vanilla `onExplosionHit`
   (ButtonBlock.java:90-100). File is outside this unit's original editable set but the fix is
   present and verified.

4. buttons.rs Paper events: FIXED/VERIFIED. `BlockRedstoneEvent(0->15)` fires on press and
   `(15->0)` on release, with a cancelled event (or plugin-forced current) keeping the button in
   its prior state; `EntityInteractEvent` fires only on the rising edge when an arrow turns on an
   unpowered wooden button, and cancelling it skips the press. Matches ButtonBlock.java:80-84,
   94-96, 168-182.

5. buttons.rs `press_with_source` ordering: FIXED/VERIFIED. Order is `set_block_state`,
   `update_neighbors`, `schedule_block_tick`, sound, game event, matching vanilla `press`
   (ButtonBlock.java:102-108).

Retained pure unit tests: `button_click_sounds_match_block_set_types` and
`stone_button_classification_matches_vanilla` in buttons.rs.

Files changed this pass: this doc only (Result section finalized).
