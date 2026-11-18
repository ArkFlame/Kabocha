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



## Result (pass 3)

Verified both STILL OPEN rows against decompiled vanilla (`Cushion.java`, `Entity.java`) and the current HEAD; no source change was needed in this unit.

- Creative-attacker no-drop (cushion.rs `drop_and_remove`, lines 176-214): RESOLVED at HEAD, no change. Vanilla `Cushion.dropItem` (Cushion.java:58-69) drops only under `GameRules.ENTITY_DROPS` and skips the drop for a `Player` with `hasInfiniteMaterials()`; Pumpkin gates `drop_stack` on `entity_drops` and skips it for a `caused_by` that downcasts to `Player` with `is_creative` (gamemode == Creative, player.rs:8487), matching vanilla. `damage_with_context` forwards the attack source correctly. No pure unit-test seam: the creative check needs a real `Player`. Note (client-only, not changed): vanilla plays the break sound/particles unconditionally while Pumpkin gates the sound on `entity_drops` (copied from `Painting.dropItem`); a sound/particle difference only, so left as-is.
- Game events ENTITY_MOUNT / ENTITY_DISMOUNT (entity/mod.rs): OUT OF UNIT, left open with reason. The fix belongs in shared `Entity::add_passenger` / `remove_passenger` (entity/mod.rs:6201-6204, 6454-6457), not an allowed file for this unit. Verified both events are already emitted at HEAD at the vehicle position, matching vanilla `Entity.startRiding` / `stopRiding`.

No .rs file was touched, so no rustfmt run.

## Result (pass 4)

Re-verified both STILL OPEN rows against decompiled vanilla `Cushion.java` and current HEAD; no source change was needed in this unit.

- Creative-attacker no-drop (cushion.rs `drop_and_remove`, lines 176-214): RESOLVED at HEAD, no change. Vanilla `Cushion.dropItem` (`Cushion.java:58-69`) drops only under `GameRules.ENTITY_DROPS` and skips the item for a `causedBy` that is a `Player` with `hasInfiniteMaterials()`. Pumpkin gates `drop_stack`/sound on `game_rules.entity_drops` and skips the drop for a `caused_by` that downcasts to a creative `Player` (`Player::is_creative`), matching vanilla; `damage_with_context` forwards the attack `source`. No pure unit-test seam (needs a real `Player`). Client-only difference left as-is: vanilla plays `CUSHION_BREAK` and breaking particles unconditionally, Pumpkin gates the sound on `entity_drops` (pattern copied from `Painting.dropItem`, which also gates). `painting.rs` uses the identical drop/creative/damage pattern (lines 278-423), consistent with vanilla `HangingEntity`.
- Game events ENTITY_MOUNT / ENTITY_DISMOUNT: OUT OF UNIT, left open. Fix belongs in shared `Entity::add_passenger` / `remove_passenger` in `entity/mod.rs`, not an allowed file for this unit (cushion.rs / painting.rs only). Verified both are already emitted at HEAD at the vehicle position (`entity/mod.rs:6214` `GameEvent::EntityMount`, `entity/mod.rs:6467` `GameEvent::EntityDismount`), matching vanilla `Entity.startRiding` / `stopRiding`.

No .rs file was touched, so no rustfmt run.

## Result (pass 5)

Re-verified all rows in `cushion.rs`/`painting.rs` against decompiled vanilla (`Cushion.java`, hanging-entity drop semantics) and current HEAD; no source change was made in allowed files.

- Creative-attacker no-drop (`cushion.rs:176-214`, `drop_and_remove`): RESOLVED at HEAD. Sound plays unconditionally before the `doEntityDrops` gate (matching `Cushion.dropItem` order), item drop is gated on `game_rules.entity_drops` and skipped for a creative `Player` `caused_by`, and `damage_with_context` forwards `source`; removed-entity hits return handled. `painting.rs` uses the matching drop/creative/removed-hit pattern. No pure seam for a unit test (needs a real `Player` fixture). Client-only gap not changed: vanilla also emits break particles and special lightning-drop item invulnerability in `Cushion.dropItem`.
- Game events ENTITY_MOUNT / ENTITY_DISMOUNT: OUT OF UNIT, not fixable here (shared `Entity::add_passenger` / `remove_passenger` in `entity/mod.rs`). Already emitted at HEAD at vehicle position (`entity/mod.rs:6214`, `entity/mod.rs:6467`), matching vanilla.
- Placement/support `pumpkin-gametest` cases (table row, M): OPEN, out of unit scope (would require test files outside allowed basenames `cushion.rs`, `painting.rs`).

No .rs file was touched, so no rustfmt run.
## Result (r2y_020 pass)
- Creative-attacker no-drop and entity_drops gate (`cushion.rs` drop_and_remove, 177-218): VERIFIED. Vanilla `Cushion.dropItem` at `.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/Cushion.java:58-69`; Rust sound before gate at `cushion.rs:195-199`, `entity_drops` gate at `:200`, creative `Player` skip at `:201-207`.
- Game events ENTITY_MOUNT / ENTITY_DISMOUNT: VERIFIED. Vanilla `Entity.java:3625` (mount at `vehicle.position`) and `:3672` (dismount at `oldVehicle.position`); Rust `entity/mod.rs:6258-6261` and `:6513-6516`, each emitted once at HEAD (the duplicate noted above is no longer present).
- Hit on removed entity returns handled: FIXED. The removed-entity early return ran before the invulnerability, griefing and build/spawn refusals. Vanilla returns `true` for a removed entity only after those refusals (`BlockAttachedEntity.java:129-160`, `Cushion.java:79-82`). Moved the `is_alive` check after them in `cushion.rs` `damage_with_context` (about 445-472).
- Colour save/load key: FIXED. Vanilla stores `"color"` as the `DyeColor.CODEC` name and reads it back with white as fallback (`Cushion.java:232-241`, `DyeColor.java:52`). Pumpkin wrote a legacy `"Color"` byte. Now writes/reads `"color"` by name (`cushion.rs:334-348`) with white fallback; the by-id clamp row is superseded by this.
- Placement and support `pumpkin-gametest` cases (M): STILL OPEN. Not reached in this pass; no test files were written and no cargo run was allowed.
- Lightning drop invulnerability (`Cushion.thunderHit` / `LIGHTNING_DROP_INVULNERABLE_TICKS`, `Cushion.java:42,64-66,143-149`): BLOCKED-DATA. Missing subsystem: no entity `thunder_hit` dispatch exists in `crates/pumpkin-core/src/entity` (grep `thunder_hit` finds nothing), and `ItemEntity` has no invulnerable-time field (`entity/item.rs`).
- Break sound/particles on drop (`Cushion.showBreakingParticles`, `Cushion.java:157-171`): STILL OPEN, client-visual. Needs a block-state particle payload (`BlockParticleOption` with wool state) that the `spawn_particle` helper does not take here.
Files edited: `crates/pumpkin-core/src/entity/decoration/cushion.rs` (rustfmt --edition 2024 --check passes). Not compiled (cargo forbidden in this brief).
