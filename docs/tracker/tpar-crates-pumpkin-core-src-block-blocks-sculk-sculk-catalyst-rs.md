# Parity rows for crates/pumpkin-core/src/block/blocks/sculk/sculk_catalyst.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **blocks/Sculk Catalyst** [partial/UNCOVERED] Placement state and scheduled-tick bloom reset are implemented. Death listener and sculk-spreading block entity remain unimplemented. Placement state and scheduled-tick bloom reset are implemented.
- **own/taudit-crates-pumpkin-core-src-block-enti** [partial/OWN-OPEN] own audit: 14 open, 10 fixed, 12 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-block-enti.md) docs/tracker/taudit-crates-pumpkin-core-src-block-enti.md

## Result
Superseded by the final Result section below (all rows closed).

## Result (third pass, final)

- **blocks/Sculk Catalyst**: VERIFIED MATCHING, no code change needed. Every behaviour the row flagged as unimplemented now exists and matches vanilla:
  - `on_place` forces `bloom = false`, matching the vanilla default state `PULSE = false` (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/SculkCatalystBlock.java:23,28`).
  - `on_scheduled_tick` clears `bloom`, matching `SculkCatalystBlock.tick` (same file `:37-41`).
  - The death listener `on_entity_death` (radius 8, BY_DISTANCE, inclusive floored block-position `<=` gate) matches `SculkCatalystBlockEntity.CatalystListener` (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/SculkCatalystBlockEntity.java:86-114`) and `EuclideanGameEventListenerRegistry.getPostableListenerPosition` (`.vendor/vanilla-decompiled/net/minecraft/world/level/gameevent/EuclideanGameEventListenerRegistry.java:115-124`).
  - The sculk-spreading block entity `SculkCatalystBlockEntity` plus `SculkSpreader::update_cursors` on each tick matches `serverTick` (`SculkCatalystBlockEntity.java:45-49`). Bloom particles/sound and the 8-tick pulse match `CatalystListener.bloom`/`PULSE_TICKS` (`:69,121-126`), and NBT cursor persistence matches `loadAdditional`/`saveAdditional` (`:52-61`).
- **own/taudit-crates-pumpkin-core-src-block-enti**: terminal. All seven audit rows are closed in that doc; rows 1-5 and 7 are FIXED/VERIFIED. Row 6 (game-event system) is BLOCKED-DATA: the exact missing subsystem is `net.minecraft.world.level.gameevent.GameEventDispatcher` with its BY_DISTANCE `GameEventListener` registry. Vanilla posts `ENTITY_DIE` from `LivingEntity.die` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/LivingEntity.java:1955`) and `Entity.kill` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java:765-767`); Pumpkin mirrors this only through the manual `LivingEntity::on_death` -> `on_entity_death` call, so exotic direct-kill paths (EnderDragon, `Entity.kill`) do not feed a catalyst. Discard/despawn does not post `ENTITY_DIE` in vanilla either, so normal deaths already match.
- No source file changed this pass: every tracked behaviour was implemented and verified in earlier passes, and no new pure seam exists for an additional test (the hearing gate is already covered by `catalyst_hearing_includes_boundary`). Files touched this pass: this doc only.
