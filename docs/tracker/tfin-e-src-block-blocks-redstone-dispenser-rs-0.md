# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-block-blocks-redstone-dispenser-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files:

Rows inherited from the parent tracker: blocks/Dispenser and the four own/ sub-audits
(boat, experience bottle x2, ominous item spawner).

## Result (final pass)

Every row is closed: FIXED and/or VERIFIED against the vendored decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/` (protocol 777 Paper/Canvas). No row is left
STILL OPEN. No generated `pumpkin-data` field, WIT change or cross-crate refactor is
required, so there is no BLOCKED-DATA row. No source file changed this pass: every fix
from earlier passes is present in the working tree and matches vanilla, so rustfmt was
not needed. Worked through all rows in order (smallest first).

| Row | Status | Vanilla evidence + Pumpkin code |
|---|---|---|
| blocks/Dispenser | VERIFIED, no change | `world/level/block/DispenserBlock.java` neighborChanged 128-140 (self-or-above signal, 4-tick schedule, TRIGGERED), tick/dispenseFrom 142-145/80-101 (random slot; empty -> SOUND_DISPENSER_FAIL + GameEvent.BLOCK_ACTIVATE), getDispensePosition 162-170 (0.7 out). `dispenser.rs` on_neighbor_update 232-256, on_scheduled_tick 258-284 and projectile_spawn_position 439-444 mirror these. Power/uncertainty 1.1/6.0 default and 1.375/3.0 potion (dispenser.rs 293-301) match `world/item/ProjectileItem.java` DispenseConfig.DEFAULT 34-38 and `world/item/ExperienceBottleItem.java` createDispenseConfig 64-68 (1.1*1.25=1.375, 6*0.5=3). |
| own/tentities-dark-oak-boat-8ee305 | VERIFIED (sub-doc final) | `world/entity/vehicle/boat/AbstractBoat.java` tick 300-328 / getPaddleSound 410-416 and getGroundFriction 506-540. Fixes live in boat.rs (`paddle_sound_for_status` 60, `play_paddle_sounds` 345, tests 1084-1112) and boat/physics.rs (`average_ground_friction` count==0 guard 267-273 + unit test 279-283). Confirmed present in source this pass. |
| own/tentities-experience-bottle-640105 | VERIFIED (sub-doc final) | All five rows closed against decompiled vanilla: dispense power 1.375/3.0 (`ExperienceBottleItem.java:64-68`), throw spawn at eye Y-0.1 (`ThrowableItemProjectile.java:32-35`), splash particles+sound 2002/2007 + 1053/1054 each gated !isSilent (`AbstractThrownPotion.java:87-97`), XP orb split/merge (`ExperienceOrb.java:270-316`), pure unit tests, plus silent gate. Confirmed in source: `projectile/experience_bottle.rs` `orb_award_direction` 22, `on_hit` 118-154 with `!entity.is_silent()` 143 and `spawn_with_direction` 152; `experience_orb.rs:120` `spawn_with_direction`. |
| own/tentities-experience-bottle | VERIFIED (sub-doc final) | copy_with_count(1) on throw (item `experience_bottle.rs:66`) and NBT load (projectile `experience_bottle.rs:114`) matches `world/entity/projectile/throwableitemprojectile/ThrowableItemProjectile.java` setItem 39-41; stack-count row confirmed false positive. |
| own/tentities-ominous-item-spawner | VERIFIED (sub-doc final) | `world/entity/OminousItemSpawner.java` spawnItem 72-89 levelEvent flag 1, spawnProjectile 91-107 uses `this.position()` (entity origin) and `overrideDispenseEvent().ifPresent` (only firework/fire-charge/wind-charge have an override, so the six silent projectiles emit no launch sound). Confirmed in source: no spurious SoundDispenserProjectileLaunch in `ominous_item_spawner.rs`; spawn at entity pos (`spawn_item` 108-131); damage_with_context -> false + is_immune_to_explosion -> true give vanilla-equivalent immunity via hurtServer -> false. |

## Result (batch x_15, CODER spot-check)
- blocks/Dispenser: VERIFIED. Rust `DispenserBlock` power/uncertainty constants at `crates/pumpkin-core/src/block/blocks/redstone/dispenser.rs:292-301`; vanilla `ProjectileItem.java:37-38` (uncertainty 6.0, power 1.1) and `ExperienceBottleItem.java:64-68`. Spawn offset 0.7 out and 0.1 up at dispenser.rs:439-444.
- own/tentities-dark-oak-boat-8ee305: VERIFIED. `crates/pumpkin-core/src/entity/vehicle/boat/physics.rs:267-273` count==0 guard present.
- own/tentities-ominous-item-spawner: VERIFIED. Vanilla `OminousItemSpawner.java:84` (PARTICLES_TRIAL_SPAWNER_SPAWN_ITEM, flag 1) and `:93` (overrideDispenseEvent gate) match the claim.
- own/tentities-experience-bottle and own/tentities-experience-bottle-640105: not re-spot-checked this pass; parent claims kept as recorded.
- Files edited: none. Status: PASS (spot-checked rows only; the rest stay as recorded).
