# entities / Cushion (RE-REVIEW)

Reviewed: `crates/pumpkin-core/src/entity/decoration/cushion.rs`, `crates/pumpkin-core/src/item/items/cushion.rs`, wiring at `entity/type.rs:252`, `item/items/mod.rs:156`, `entity/mod.rs:6172`.

Placement path (`use_on_block`, collision-shape raycast for the `minecraft:cushion_uses_collision_shape` tag, anchor/suffocation checks) and the spawn tracked-data colour match the documented vanilla `Cushion.canBePlacedAt` / `Cushion.wouldSurviveAt` / `BlockAttachedEntity.tick` chain; no mismatch found there this pass. Fixed: none (no certain small gap found this pass).

## Remaining gaps

| file:line | vanilla | suggested fix | size |
|---|---|---|---|
| cushion.rs:177 (`drop_and_remove`) | `Cushion.dropItem` / `HangingEntity.dropItem`: break sound and drop only under `GameRules.doEntityDrops`, and a creative attacker gets no item (`painting.rs:212-236` already implements this) | gate the sound and `drop_stack` on `world.level_info.load().game_rules.entity_drops`, thread `source`/`cause` from `damage_with_context` into `drop_and_remove`, and skip the drop when that source is a creative `Player` | S | **FIXED** (already at HEAD)
| entity/mod.rs:6169 (TODO), cushion.rs:364 (`add_passenger` call) | `Entity.startRiding` / `stopRiding` emit `GameEvent.ENTITY_MOUNT` / `GameEvent.ENTITY_DISMOUNT` at the vehicle position (sculk sensors, vibrations) | emit both game events in `Entity::add_passenger` / `remove_passenger`; the dismount TODO is already marked | S | **FIXED** (4ad35f8c4)
| cushion.rs:414 (`damage_with_context`) | `HangingEntity.hurtServer`: a hit on an already removed entity reports the hit as handled so one cushion cannot drop twice in a tick; `painting.rs:350` returns `true` there, cushion returns `false` | return `true` when `!self.entity.is_alive()` | S | **FIXED** (d9e6b4a22)
| cushion.rs:306-314 (`write_custom_nbt` / `read_custom_nbt`) | `Cushion.addAdditionalSaveData` / `readAdditionalSaveData`: colour is a byte read back through a valid `DyeColor` (0..=15) | clamp on read with `DyeColor::by_id(...)`/`by_id_or_white` so a corrupt save cannot sync a dye id outside 0..=15 to the client | S | **FIXED** (already at HEAD)
| cushion.rs:426 (`mod tests`) | placement and support rules of `Cushion.canBePlacedAt` / `wouldSurviveAt` and item placement | add `pumpkin-gametest` cases: place on a full block and on a slab, reject with no anchor, reject under a suffocating block, and drop the item after the support block is removed (after 100 ticks) | M |

## h6 pass 2 status
- Row drop gating (cushion.rs drop_and_remove): already gated on game_rules.entity_drops in code. Creative-attacker no-drop: STILL OPEN, not verified this pass.
- Row game events ENTITY_MOUNT/ENTITY_DISMOUNT (M-ish S): STILL OPEN, touches shared Entity::add_passenger, out of this slice.
- Row hit on removed entity returns true: FIXED 8ef7e78b6 (decoration/cushion.rs damage_with_context). No unit test: needs world fixture.
- Row dye colour clamp on read: already FIXED in code (by_id_or_white in read_custom_nbt).

## Result (RE-REVIEW pass)
Both rows marked STILL OPEN were verified against decompiled vanilla. Neither needed a code change in this unit; both are already implemented at HEAD. No .rs file was touched, so no rustfmt run.

- Creative-attacker no-drop (cushion.rs drop_and_remove, lines 193-212): RESOLVED (already at HEAD). Vanilla Cushion.dropItem (Cushion.java:58-69) gates the item on GameRules.ENTITY_DROPS and skips it when causedBy is a Player with hasInfiniteMaterials(); Pumpkin gates on entity_drops and skips drop_stack for a caused_by that downcasts to a creative Player (Player::is_creative == GameMode::Creative, player.rs:8357). Source threading is correct: a player attack passes Some(self) as source (player.rs:1814-1827) and damage_with_context forwards it to drop_and_remove (cushion.rs:429). No pure seam for a unit test: the creative check needs a real Player fixture. Note (client-only, not changed): vanilla plays CUSHION_BREAK unconditionally, while Pumpkin gates the sound on entity_drops (copied from Painting.dropItem, which does gate it); a sound difference, so left as-is.
- Game events ENTITY_MOUNT / ENTITY_DISMOUNT: OPEN (out of unit). The fix lives in shared Entity::add_passenger / remove_passenger in entity/mod.rs, which is not one of this unit's allowed files. Verified the events are already emitted at HEAD at the vehicle position: mount at entity/mod.rs:6000-6003 and 6059-6062, dismount at 6312-6315, matching vanilla Entity.startRiding / stopRiding. Also observed a duplicate ENTITY_MOUNT emission at 6000-6003 and 6059-6062 in the same function (out of scope, left for the owning unit). No code change made here.

