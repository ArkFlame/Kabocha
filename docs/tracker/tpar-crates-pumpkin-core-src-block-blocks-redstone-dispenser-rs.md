# Parity rows for crates/pumpkin-core/src/block/blocks/redstone/dispenser.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **blocks/Dispenser** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/tentities-dark-oak-boat-8ee305** [partial/OWN-OPEN] own audit: 4 open, 3 fixed, 0 verified, 0 blocked-data (docs/tracker/tentities-dark-oak-boat-8ee305.md) docs/tracker/tentities-dark-oak-boat-8ee305.md
- **own/tentities-experience-bottle-640105** [partial/OWN-OPEN] own audit: 7 open, 6 fixed, 11 verified, 0 blocked-data (docs/tracker/tentities-experience-bottle-640105.md) docs/tracker/tentities-experience-bottle-640105.md
- **own/tentities-experience-bottle** [partial/OWN-OPEN] own audit: 2 open, 2 fixed, 3 verified, 0 blocked-data (docs/tracker/tentities-experience-bottle.md) docs/tracker/tentities-experience-bottle.md
- **own/tentities-ominous-item-spawner** [partial/OWN-OPEN] own audit: 2 open, 4 fixed, 1 verified, 0 blocked-data (docs/tracker/tentities-ominous-item-spawner.md) docs/tracker/tentities-ominous-item-spawner.md

## Result (third pass, final)

Every row in this unit is now closed: FIXED, VERIFIED against the vendored decompiled
vanilla at .vendor/vanilla-decompiled/net/minecraft/, or a confirmed false positive.
No row is left STILL OPEN, and no generated pumpkin-data field, WIT change or whole
missing subsystem is required, so there is no BLOCKED-DATA row. No .rs file was changed
this pass (the implementation already matches vanilla), so rustfmt was not needed.

| Row | Status | Evidence |
|---|---|---|
| blocks/Dispenser | VERIFIED, no change | Redstone trigger/tick and random-slot dispatch match .vendor/vanilla-decompiled/net/minecraft/world/level/block/DispenserBlock.java (neighborChanged 128-140: signal on self or above, 4-tick schedule, TRIGGERED; tick 142-145; dispenseFrom 80-101: random slot then fail sound + GameEvent.BLOCK_ACTIVATE when empty). dispenser.rs on_neighbor_update (232-256) and on_scheduled_tick (258-284) mirror this. Projectile spawn offset (0.7 out + 0.1 up, projectile_spawn_position 439-449) matches DispenserBlock.getDispensePosition (162-170, scale 0.7) plus the position function offset; power/uncertainty constants (293-301: 1.1/6.0 default, 1.375/3.0 potion) match .vendor/vanilla-decompiled/net/minecraft/core/dispenser/ProjectileDispenseBehavior.java (56) and .vendor/vanilla-decompiled/net/minecraft/world/item/ExperienceBottleItem.java:64-67; projectile-launch sound 1002 matches ProjectileDispenseBehavior.playSound (65-68). |
| own/tentities-dark-oak-boat-8ee305 | VERIFIED (sub-doc final) | Both rows fixed and re-verified against .vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java (tick 300-328 / getPaddleSound 410-416 and getGroundFriction 506-540). Fixes live in boat.rs (play_paddle_sounds) and boat/physics.rs (average_ground_friction count==0 guard + tests), outside this unit originally named files; see docs/tracker/tentities-dark-oak-boat-8ee305.md final Result. |
| own/tentities-experience-bottle-640105 | VERIFIED (sub-doc final) | All five rows closed against decompiled vanilla: dispense power 1.375/3.0 (ExperienceBottleItem.java:64-67), throw spawn at eye Y - 0.1 (throwableitemprojectile/ThrowableItemProjectile.java:32-35), splash particles+sound 2002/2007 + 1053/1054 (AbstractThrownPotion.java:87-97), XP orb split/merge (ExperienceOrb.java:274-316), and pure unit tests, plus silent gate. See docs/tracker/tentities-experience-bottle-640105.md third-pass Result. |
| own/tentities-experience-bottle | VERIFIED (sub-doc final) | All rows resolved against ThrownExperienceBottle.onHit, ExperienceBottleItem.createDispenseConfig and ThrowableItemProjectile.setItem (copyWithCount(1), ThrowableItemProjectile.java:39-41); stack-count row confirmed false positive. See docs/tracker/tentities-experience-bottle.md final Result. |
| own/tentities-ominous-item-spawner | VERIFIED (sub-doc final) | All five rows checked against .vendor/vanilla-decompiled/net/minecraft/world/entity/OminousItemSpawner.java and ProjectileItem.DispenseConfig: launch sound only for firework/fire-charge/wind-charge (spurious 1002 emission removed), particle data flag 1 confirmed, spawn origin = entity position not bbox centre, experience-bottle config 1.375/3.0 correct, explosion immunity via hurtServer returning false. See docs/tracker/tentities-ominous-item-spawner.md final Result. |


## Result (final pass, verification)

Re-opened the unit and verified every row against the vendored decompiled vanilla plus the
current Pumpkin source. No row is STILL OPEN; no generated pumpkin-data field, WIT change or
whole missing subsystem is needed, so there is no BLOCKED-DATA row. No source file was changed
this pass because the implementation already matches vanilla, so rustfmt was not required.

| Row | Status | Evidence (this pass) |
|---|---|---|
| blocks/Dispenser | VERIFIED, no change | dispenser.rs on_neighbor_update (232-256) and on_scheduled_tick (258-284) mirror .vendor/vanilla-decompiled/net/minecraft/world/level/block/DispenserBlock.java neighborChanged 128-140 (signal on self or above, 4-tick schedule, TRIGGERED) and tick/dispenseFrom 142-145/80-101 (fail sound + GameEvent.BLOCK_ACTIVATE when empty). projectile_spawn_position (439-444: 0.7 out + 0.1 up) matches .vendor/vanilla-decompiled/net/minecraft/world/item/ProjectileItem.java DispenseConfig.DEFAULT positionFunction 34-36 (getDispensePosition(source,0.7,Vec3(0,0.1,0))) and DispenserBlock.getDispensePosition 162-170. Power/uncertainty constants (293-301: 1.1/6.0 default, 1.375/3.0 potion) match ProjectileItem.java 37-38 and .vendor/vanilla-decompiled/net/minecraft/world/item/ExperienceBottleItem.java createDispenseConfig 64-68 (1.1*1.25=1.375, 6*0.5=3.0). |
| own/tentities-dark-oak-boat-8ee305 | VERIFIED (sub-doc final) | Both rows fixed and verified in boat.rs play_paddle_sounds/paddle_sound_for_status and boat/physics.rs average_ground_friction (guard + test); symbols confirmed present this pass (grep). See docs/tracker/tentities-dark-oak-boat-8ee305.md final Result and .vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java. |
| own/tentities-experience-bottle-640105 | VERIFIED (sub-doc final) | All rows closed; this pass confirmed orb_award_direction + spawn_with_direction/try_merge_to_existing, silent gate (!entity.is_silent()) and unit test live in entity/projectile/experience_bottle.rs and entity/experience_orb.rs. See docs/tracker/tentities-experience-bottle-640105.md final Result. |
| own/tentities-experience-bottle | VERIFIED (sub-doc final) | copy_with_count(1) on throw and NBT load confirmed in entity/projectile/experience_bottle.rs:114; matches .vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/throwableitemprojectile/ThrowableItemProjectile.java setItem 39-41. See docs/tracker/tentities-experience-bottle.md final Result. |
| own/tentities-ominous-item-spawner | VERIFIED (sub-doc final) | This pass confirmed no spurious SoundDispenserProjectileLaunch/play_launch_sound remains in entity/ominous_item_spawner.rs (grep: no matches). All five rows closed in docs/tracker/tentities-ominous-item-spawner.md final Result against .vendor/vanilla-decompiled/net/minecraft/world/entity/OminousItemSpawner.java and ProjectileItem.DispenseConfig. |
