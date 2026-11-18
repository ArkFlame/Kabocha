# Parity rows for crates/pumpkin-core/src/block/blocks/redstone/buttons.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-bl-f6470f** [partial/OWN-OPEN] own audit: 10 open, 27 fixed, 1 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-bl-f6470f.md) docs/tracker/taudit-crates-pumpkin-core-src-bl-f6470f.md

## Result

Final pass. All five gap rows in the referenced audit are resolved against the decompiled
vanilla source under `.vendor/vanilla-decompiled/net/minecraft/world/level/block/`
(`ButtonBlock.java`, `AbstractCandleBlock.java`, `state/properties/BlockSetType.java`).
Every row is FIXED or VERIFIED; no row needs a generated `pumpkin-data` field or a missing
subsystem. No source edit was needed this pass: each fix from earlier passes is present in
the current checkout and was re-checked against vanilla now.

1. `on_scheduled_tick` arrow detection box: FIXED/VERIFIED. While a wooden button is powered,
   the tick unions the state's `get_block_outline_shapes_at(pos)` boxes via `.at_pos(*pos)` and
   searches those thin-shape bounds for arrows instead of the full block cube, matching
   `checkPressed` (`ButtonBlock.java:162-165`, `state.getShape(level, pos).bounds().move(pos)`).
   APIs checked: `get_block_outline_shapes_at` (`pumpkin-data/src/block_state.rs:339`) returns
   block-local outline boxes and `BoundingBox::at_pos` (`pumpkin-util/src/math/boundingbox.rs:187`)
   adds the block position. An arrow lodged in the wall no longer holds a button pressed.
2. `press_with_source` game-event source and player: FIXED/VERIFIED. A hand press passes the
   player and an arrow trigger passes the arrow through `emit_game_event_with_context` with
   `GameEventContext { source_entity }`; release emits `BlockDeactivate` with no source,
   matching `press`/`checkPressed` (`ButtonBlock.java:107,186`, `gameEvent(player/firstArrow, ...)`;
   `firstArrow` is null on release).
3. `explode` (`onExplosionHit` before removal): VERIFIED. `explode` guards on the placed block
   and `can_trigger_blocks`, then presses via `press` (which returns early when already powered),
   matching `ButtonBlock.java:90-100` including Paper's `callBinaryRedstoneChange` on line 94.
   The explosion hook runs before `set_block_state(AIR)` in `world/explosion.rs` (outside this
   unit).
4. Paper events: FIXED/VERIFIED. `BlockRedstoneEvent` fires 0 to 15 on press and 15 to 0 on
   release; `EntityInteractEvent` fires on the arrow rising edge in `on_entity_collision`
   (wooden buttons only). A cancelled event cancels the press/release, matching
   `ButtonBlock.java:80-84,94-96,168-182`.
5. `press` ordering: FIXED/VERIFIED. `set_block_state`, `update_neighbors`,
   `schedule_block_tick`, sound, game event, matching vanilla `press`
   (`ButtonBlock.java:102-108`).

Also verified this pass: `normal_use` returns `Consume` only for an already-powered button and
`Success` on a press cancelled by `BlockRedstoneEvent`, matching `useWithoutItem`
(`ButtonBlock.java:76-87`); `click_sounds` maps stone/cherry/bamboo/nether-wood/default-wood to
their vanilla `BlockSetType` sound pairs (`ButtonBlock.java:114-116`);
`stone_button_classification_matches_vanilla` reflects `BlockSetType.STONE`/
`POLISHED_BLACKSTONE` (`canButtonBeActivatedByArrows = false`, 20/30 tick delays).

Pure tests retained: `button_click_sounds_match_block_set_types` and
`stone_button_classification_matches_vanilla`. No new test added: every fixed behaviour needs a
live `World` (entities, block states, plugins, explosions), so a unit test would be heavily
mocked or formula-mirroring, which Kabocha rules forbid.

Worker compile-pitfall re-check: no unwrap/expect/panic/todo, no println, no em dashes; no new
imports, helpers, struct initializers, moved values or unstable std; `Self` used inside the impl.
Files changed this pass: this doc only; no Rust source edit, so rustfmt was not required.
Checks not run (worker rule; orchestrator batch-compiles); verified by source and API review only.
