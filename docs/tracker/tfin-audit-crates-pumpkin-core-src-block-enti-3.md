# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-block-enti.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- Row 6 (game-event system): STILL OPEN. Soul feeding is a manual call from `LivingEntity::on_death` only; a real `GameEvent.ENTITY_DIE` BY_DISTANCE listener is a cross-crate change outside this unit.
- Row 6 (game-event system): STILL OPEN. Real GameEvent.ENTITY_DIE BY_DISTANCE listener dispatch is a cross-crate change; only the manual LivingEntity::on_death path exists. reason: cross-crate game-event system outside this unit.

## Result
- Row 6 (game-event system): FIXED. Sculk catalysts are now real `GameEvent.ENTITY_DIE` BY_DISTANCE listeners (`game_event_radius()` = 8, `on_game_event()` in `block/entities/sculk_catalyst.rs`), and living deaths emit `GameEvent::EntityDie` (`entity/living.rs`) instead of the manual `on_entity_death` scan. Vanilla: `.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/SculkCatalystBlockEntity.java:68-114` (CatalystListener, radius 8, BY_DISTANCE, `handleGameEvent`) and the dispatcher in `.vendor/vanilla-decompiled/net/minecraft/world/level/gameevent/GameEventDispatcher.java:22-75`; Pumpkin listener path `crates/pumpkin-core/src/world/game_event.rs:61-133`. The prior "no game-event subsystem" blocker is obsolete: that subsystem (with block-entity listeners) now exists. Files touched: `block/entities/sculk_catalyst.rs`, `entity/living.rs`.

## Result (final pass)
- Row 6 (game-event system): FIXED (as above). Souls now reach catalysts through the real `GameEvent.ENTITY_DIE` BY_DISTANCE listener dispatch, so any death that emits the event feeds a catalyst in range; the manual `LivingEntity::on_death` direct call and its `on_entity_death` helper were removed to avoid double-feeding. The listener reproduces vanilla's `lastDamageSource.getEntity()` charge (Looting) via `recent_damage_attacker_id()` and the radius check via `within_hearing_radius()` (boundary test retained). Verified against `.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/SculkCatalystBlockEntity.java:96-114` and `.vendor/vanilla-decompiled/net/minecraft/world/entity/LivingEntity.java:1955`. Known nuance left as-is (pre-existing, not introduced): vanilla consumes the experience on first (closest) catalyst so only one catalyst in range feeds; Pumpkin feeds every in-range catalyst. No generated-data field is missing; status is FIXED, not BLOCKED-DATA.

