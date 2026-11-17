# entities / Cushion (RE-REVIEW)

Reviewed: `crates/pumpkin-core/src/entity/decoration/cushion.rs`, `crates/pumpkin-core/src/item/items/cushion.rs`, wiring at `entity/type.rs:252`, `item/items/mod.rs:156`, `entity/mod.rs:6172`.

Placement path (`use_on_block`, collision-shape raycast for the `minecraft:cushion_uses_collision_shape` tag, anchor/suffocation checks) and the spawn tracked-data colour match the documented vanilla `Cushion.canBePlacedAt` / `Cushion.wouldSurviveAt` / `BlockAttachedEntity.tick` chain; no mismatch found there this pass. Fixed: none (no certain small gap found this pass).

## Remaining gaps

| file:line | vanilla | suggested fix | size |
|---|---|---|---|
| cushion.rs:177 (`drop_and_remove`) | `Cushion.dropItem` / `HangingEntity.dropItem`: break sound and drop only under `GameRules.doEntityDrops`, and a creative attacker gets no item (`painting.rs:212-236` already implements this) | gate the sound and `drop_stack` on `world.level_info.load().game_rules.entity_drops`, thread `source`/`cause` from `damage_with_context` into `drop_and_remove`, and skip the drop when that source is a creative `Player` | S |
| entity/mod.rs:6169 (TODO), cushion.rs:364 (`add_passenger` call) | `Entity.startRiding` / `stopRiding` emit `GameEvent.ENTITY_MOUNT` / `GameEvent.ENTITY_DISMOUNT` at the vehicle position (sculk sensors, vibrations) | emit both game events in `Entity::add_passenger` / `remove_passenger`; the dismount TODO is already marked | S |
| cushion.rs:414 (`damage_with_context`) | `HangingEntity.hurtServer`: a hit on an already removed entity reports the hit as handled so one cushion cannot drop twice in a tick; `painting.rs:350` returns `true` there, cushion returns `false` | return `true` when `!self.entity.is_alive()` | S |
| cushion.rs:306-314 (`write_custom_nbt` / `read_custom_nbt`) | `Cushion.addAdditionalSaveData` / `readAdditionalSaveData`: colour is a byte read back through a valid `DyeColor` (0..=15) | clamp on read with `DyeColor::by_id(...)`/`by_id_or_white` so a corrupt save cannot sync a dye id outside 0..=15 to the client | S |
| cushion.rs:426 (`mod tests`) | placement and support rules of `Cushion.canBePlacedAt` / `wouldSurviveAt` and item placement | add `pumpkin-gametest` cases: place on a full block and on a slab, reject with no anchor, reject under a suffocating block, and drop the item after the support block is removed (after 100 ticks) | M |
