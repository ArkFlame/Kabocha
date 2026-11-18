# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-entity-living-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Re-checked every STILL OPEN row across the four referenced audits against the decompiled vanilla tree under `.vendor/vanilla-decompiled/net/minecraft/`. All rows are now closed; no row remains STILL OPEN.
Every STILL OPEN row in all four referenced audits is now closed. Each row was re-checked against the decompiled vanilla tree at `.vendor/vanilla-decompiled/net/minecraft/` and against the actual source in the main checkout. All claimed fixes are present in the tree; no new source edit was needed this pass.

## Result
FINAL PASS. Every referenced row is closed. Each was re-checked this pass against the actual source in the main checkout and against the decompiled vanilla tree at `.vendor/vanilla-decompiled/net/minecraft/`. All claimed fixes are present in the tree; no source edit was needed this pass.

### taudit-crates-pumpkin-core-src-en-06afb0 (8 rows)
- 1 `add_effect` instance update: FIXED. `update_effect` (living.rs:5625) merges the incoming instance (stronger amplifier wins, equal amplifier only extends, infinite stored duration never shortened) before the attribute rebuild and only resends on change; tests `effect_update_tests` (:5666). Vanilla `LivingEntity.addEffect` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/LivingEntity.java:1205-1230`) and `MobEffectInstance.update`.
- 2 Negative heal without `die`: VERIFIED, no change. `heal` (living.rs:1263) clamps at 0 without `die()`; vanilla `LivingEntity.heal` returns void and clamps at `setHealth` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/LivingEntity.java:1430`). No config switch exists in vanilla.
- 3 Fall damage prevention raycast: FIXED. `move_entity` (crates/pumpkin-core/src/entity/mod.rs:3773-3795) raycasts the collision-adjusted movement from the pre-move position (capped at 8 blocks) for `#minecraft:fall_damage_resetting` or water and resets fall distance before `checkFallDamage`, matching vanilla `Entity.move` `ClipContext.Block.FALLDAMAGE_RESETTING` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java:1245`). The landed-block decision is `LivingEntity.checkFallDamage` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/LivingEntity.java:424`) / `Block.fallOn`.
- 4 Effective gravity below world: VERIFIED, no change. The `-0.1`/`0.0` fallback is client-only in `Entity.getGravity`; the server always subtracts gravity (`.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java`).
- 5 Climbing sneak clamp: FIXED. `climbing_descent_blocked` (living.rs:5656) plus the `handleOnClimbable` use (:2600) zero a sneaking player's negative Y velocity off anything but scaffolding; unit test (:5714). Vanilla `LivingEntity.handleOnClimbable` / `isSuppressingSlidingDownLadder` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/LivingEntity.java:3413-3420,4589`).
- 6 `MobEffectInstance` NBT fields: VERIFIED, no change. `blend` is transient after NBT load in vanilla (`MobEffectInstance.load` / `BlendState`); Pumpkin omits it from write and derives it on read.
- 7 `bypasses_armor` tag: FIXED, no longer BLOCKED-DATA. Generated `tag::DamageType::MINECRAFT_BYPASSES_ARMOR` exists (crates/pumpkin-data/src/generated/tag.rs:20069); production armor absorb uses it (living.rs:4104) and the `#[cfg(test)]` helper `bypasses_armor_durability` (:5745-5746) delegates to the generated tag. Vanilla tag `minecraft:bypasses_armor`.
- 8 Looting from killing blow: FIXED. `drop_equipment` (living.rs:3082) takes `looting_level`/`killed_by_player` from the killing attacker and only rolls a chance for a player kill, matching vanilla `Mob.dropCustomDeathLoot` / `EnchantmentHelper.processEquipmentDropChance` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/Mob.java`, `.vendor/vanilla-decompiled/net/minecraft/world/item/enchantment/EnchantmentHelper.java`).

### taudit-crates-pumpkin-core-src-entity-liv (5 rows)
- 1 `heal`: VERIFIED, no change (deep-audit row 2 above); vanilla `LivingEntity.heal` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/LivingEntity.java:1430`).
- 2 Fall damage prevention block state: FIXED. `should_prevent_fall_damage_in_area` (living.rs:1974) reads the single block under the feet plus `#minecraft:fall_damage_resetting`, matching vanilla `Block.fallOn` (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/Block.java`).
- 3 `getGravity` void fallback: VERIFIED, no change. Client-only branch in `Entity.getGravity` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java`).
- 4 Scaffolding descent clamp: FIXED (deep-audit row 5 above), unit test present.
- 5 Consumable-effect probability: FIXED. `consume_effect_probability_applies` (living.rs:5661) gates the data-driven `ConsumableImpl` probability at :5269 with strict `random < probability`; unit test (:5725-5732). Vanilla `ApplyStatusEffectsConsumeEffect.apply` (`.vendor/vanilla-decompiled/net/minecraft/world/item/consumable/ApplyStatusEffectsConsumeEffect.java`).

### tentities-axolotl (6 rows)
- 1 Effective-AI travel guard: FIXED. `travel_in_water` falls back on `is_no_ai()` (axolotl.rs:592-594), matching vanilla `Axolotl.travelInWater` gated on `isEffectiveAi() && isInWater()` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/axolotl/Axolotl.java:531-535`).
- 2 Play-dead regen constants: VERIFIED. `PlayDead.start` applies `REGENERATION` 200/0 (`.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/axolotl/PlayDead.java:32`); Pumpkin uses 200/0.
- 3 No-AI air path: VERIFIED. Vanilla `Axolotl.baseTick` / `handleAirSupply` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/axolotl/Axolotl.java:184-189,256-266`); Pumpkin routing matches.
- 4 Hurt play-dead condition: FIXED. `has_causing_entity: source.is_some() || cause.is_some()` (axolotl.rs:374), tests at :1358-1359, matching vanilla `Axolotl.hurtServer` `(source.getEntity() != null || source.getDirectEntity() != null)` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/axolotl/Axolotl.java:396-408`).
- 5 Bucket NBT keys: VERIFIED. Vanilla `saveToBucketTag` / `loadFromBucketTag` write and restore Age/AgeLocked/HuntingCooldown (`.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/axolotl/Axolotl.java:427-449`); Pumpkin (axolotl.rs:412-452) matches.
- 6 Duplicated doc comment: VERIFIED, single doc line (axolotl.rs:184).

### tentities-strider (6 rows)
- 1 Shivering sync / `set_suffocating`: FIXED. `mob_tick` ports the vanilla temperature update and fires `StriderTemperatureChangeEvent` only on a flip (strider.rs:224-239), matching `Strider.tick` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Strider.java`).
- 2 Navigation: VERIFIED, no change. Vanilla `Strider.createNavigation` returns a ground navigator with a `WalkNodeEvaluator` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Strider.java:388-389,523-537`), equal to Pumpkin's default; `is_stable_destination` (:255) and `get_walk_target_value` (:267) cover the lava behaviour. An `AmphibiousPathNavigation` hook would be a regression.
- 3 Jockey spawn: FIXED. `finalize_spawn` (strider.rs:309) ports the 1/30 zombified piglin and 1/10 baby strider branches with the baby-chance override, matching `Strider.finalizeSpawn` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Strider.java`).
- 4 Sounds (happy + retreat): FIXED. Happy from `use_player_item`; retreat in `mob_tick` on `is_panicking()` with a 1/60 chance (strider.rs:208), matching `Strider.tick` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Strider.java`).
- 5 Lava-only fluid standing: FIXED in living.rs. Lava skips fluid travel, water uses water travel, matching `Strider.canStandOnFluid` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Strider.java`).
- 6 Passenger attachment: VERIFIED, no server change. Vanilla applies the rider offset only when `level().isClientSide()` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Strider.java:190-199`); the server returns `super`.

No BLOCKED-DATA rows remain: the sole former blocker (`MINECRAFT_BYPASSES_ARMOR`) is generated at crates/pumpkin-data/src/generated/tag.rs:20069. No pure unit test was added: every row with a pure seam already has a test (`effect_update_tests`, `climbing_descent_blocked`, `consume_effect_probability_applies`, `bypasses_armor_durability`, axolotl hurt/bucket); the remaining rows are world/RNG/plugin driven. No `.rs` file was changed this pass, so rustfmt was not applicable. No cargo or git command was run per task rules; compile and tests are UNVERIFIED (orchestrator batch-compiles).

Status: PASS fixed=0 open=0 files=docs/tracker/tfin-crates-pumpkin-core-src-entity-living-rs-0.md
