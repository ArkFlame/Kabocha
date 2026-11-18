# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-bl-f6470f.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

All five resolved rows from the original audit are already fixed in the current checkout.
All five resolved rows from the original Gap status table are FIXED/VERIFIED against
All five resolved rows in the Gap status table are now resolved against the decompiled
AbstractCandleBlock.java). No row is left resolved; no generated pumpkin-data field or

## Result

Unit closed. No `.rs` file needed editing: every fix is already present in the current checkout
and was re-verified against the decompiled vanilla source under
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/` (`ButtonBlock.java`,
`AbstractCandleBlock.java`). No row is open; no generated pumpkin-data field or missing
subsystem blocks any row. No shell/cargo/git was run by this worker.

Per-row final results (the five rows carried over from the original audit):

1. `buttons.rs` `on_scheduled_tick` arrow detection box: FIXED/VERIFIED. The state's
   `get_block_outline_shapes_at(pos)` boxes are unioned via `.at_pos(*pos)` instead of the full
   block cube, matching vanilla `checkPressed`
   (.vendor/vanilla-decompiled/net/minecraft/world/level/block/ButtonBlock.java:162-165,
   `state.getShape(level, pos).bounds().move(pos)`). An arrow lodged in the supporting wall no
   longer holds a wooden button pressed.
2. `buttons.rs` `press_with_source` game-event source: FIXED/VERIFIED. Player for a hand press
   and arrow for a wooden-button trigger are passed through `emit_game_event_with_context`
   with `GameEventContext { source_entity }`; release emits `BlockDeactivate` with no source.
   Matches ButtonBlock.java:107,186 (`gameEvent(player/firstArrow, ...)`; `firstArrow` is null
   on release). No cross-crate change needed.
3. `world/explosion.rs` `onExplosionHit` before removal: VERIFIED (file outside this unit's
   original editable set; re-read here). `pumpkin_block.explode(ExplodeArgs { .. })` runs at
   explosion.rs:948 before `set_block_state(pos, AIR)` at explosion.rs:978, so the placed block is
   present, matching vanilla `onExplosionHit` (ButtonBlock.java:90-100). The button `explode` guards
   on `can_trigger_blocks` (buttons.rs:106) and the candle / candle-cake `explode` hooks guard the
   same way (candles.rs:83, candle_cakes.rs:134) before extinguishing, matching
   AbstractCandleBlock.java:104-113 (`explosion.canTriggerBlocks() && state.getValue(LIT)`).
4. `buttons.rs` Paper events: FIXED/VERIFIED. `BlockRedstoneEvent` fires on press (0->15,
   buttons.rs:301-304) and release (15->0, buttons.rs:181-198), and `EntityInteractEvent` fires on
   the arrow rising edge (buttons.rs:115-129); a cancelled event cancels the press/release. Matches
   ButtonBlock.java:80-84, 94-96, 168-182.
5. `buttons.rs` `press_with_source` ordering: FIXED/VERIFIED. `set_block_state`, `update_neighbors`,
   `schedule_block_tick`, sound, game event (buttons.rs:306-339), matching vanilla `press`
   (ButtonBlock.java:102-108).

Also re-verified the earlier in-unit fixes: stone/cherry/bamboo/nether-wood click sounds via
`click_sounds` (vanilla `getSound` -> `type.buttonClickOn()/Off()`, ButtonBlock.java:114-116);
`daylight_detector.rs` uses `Player::may_build` (vanilla `!player.mayBuild()`,
DaylightDetectorBlock.java:70-72); candles/candle-cakes pass the clicking player to `gameEvent`
on manual extinguish (vanilla AbstractCandleBlock.java:97). All present.

Pure tests retained: `button_click_sounds_match_block_set_types` and
`stone_button_classification_matches_vanilla`. No new test added: every fixed behaviour needs a
live `World` (entities, block states, plugins, explosions), so a unit test would be heavily mocked
or formula-mirroring, which Kabocha rules forbid.

Worker compile-pitfall re-check: no `unwrap`/`expect`/`panic`/`todo`, no stdout printing, no em
dashes; imports resolve; no helper in a trait impl, no macro free-fn path, no new struct
initializer, no moved-then-reused value, no unstable std. No `.rs` file was edited, so rustfmt was
not required.

Files changed this pass: this doc only (Result section).
