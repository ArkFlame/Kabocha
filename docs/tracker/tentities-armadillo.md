# entities / Armadillo — re-review

Source: `crates/pumpkin-core/src/entity/passive/armadillo.rs` (last touched by d629b8962).
Reference: vanilla `net.minecraft.world.entity.animal.armadillo.Armadillo` and
`Armadillo$ArmadilloState` (1.21.5+). No decompiled sources in `.vendor/`, so each row names the
vanilla member it must match.

## Fixed this pass
- **Ambient sound never played.** `Mob::get_ambient_sound` defaults to `None` and mobs must override
  it (see bat.rs:200); `EntityType::ARMADILLO.ambient_sound` is never consulted automatically.
  Vanilla `Armadillo.getAmbientSound` -> `ARMADILLO_AMBIENT`. Added
  `fn get_ambient_sound` (armadillo.rs:342).
- **Hurt sound was always the loud variant.** Vanilla `Armadillo.getHurtSound` returns
  `ARMADILLO_HURT_REDUCED` while the body is hidden in the shell and `ARMADILLO_HURT` otherwise.
  Added `impl CustomSound` (armadillo.rs:327) returning the reduced variant only when
  `should_hide_in_shell()` and `None` otherwise so `living.rs:3772` falls back to the
  `pumpkin-data` hurt sound. This also gives `should_hide_in_shell` (armadillo.rs:191) its first
  caller; it was dead code.

## Remaining gaps

| # | Where | Vanilla member | Gap | Fix | Size |
|---|-------|----------------|-----|-----|------|
| 1 | armadillo.rs:273 `is_scared_by`, armadillo.rs:34 `SCARE_CHECK_INTERVAL` | `Armadillo.customServerAiStep` proximity check | `is_scared_by` has **no caller**; nothing scans for nearby threats, so an armadillo never rolls up on approach, only after being hit. `SCARE_CHECK_INTERVAL` is only reused as the danger-duration value at armadillo.rs:373, never as a tick interval | In `custom_server_ai_step`, when state is `Idle` and `danger_detected_recently_ticks == 0` and `tick_count % 80 == 0`, iterate `world.get_nearby_entities(pos, 7.0)` (world/mod.rs:5371), keep living entities and `is_scared_by` hits, then `roll_up()` | M |
| 2 | armadillo.rs:367-379 `on_damage` | `Armadillo.hurtServer` | Both branches store the same value (80), so the `can_stay_rolled_up()` check is dead. `SCARE_CHECK_INTERVAL` is used as a duration, not an interval | Collapse to one store of 80 for non-`ItemEntity` sources; confirm from vanilla whether `!can_stay_rolled_up()` zeroes it instead | S |
| 3 | armadillo.rs:359-365 `modify_incoming_damage` | `Armadillo.getHurtSound`/shell damage handling | Reduction is applied whenever `is_scared()` (Rolling from tick 1, Scared, Unrolling until the last tick) instead of only once `should_hide_in_shell()` is true | Gate the `(amount - 1.0).max(0.0) / 2.0` reduction on `self.should_hide_in_shell()` and verify the formula against the decompiled `Armadillo` | S |
| 4 | armadillo.rs:445-455 `mob_init_data_tracker` | `AgeableMob.defineSynchedData` | The baby flag is set by hand from `entity.age < 0`; `ageable.rs:115` already syncs `DATA_BABY_ID` on every age change, so this is redundant (and duplicates a different constant, `armadillo::BABY_ID`) | Drop the hand-set line and rely on the ageable sync | S |
| 5 | armadillo.rs:236-271 `brush_off_scute` | `Armadillo.brushOffScute` + `AnimalMixin` drop | Drop is spawned with `world.spawn_entity` (saved) while the shed drop at armadillo.rs:405 uses `spawn_entity_non_save`; vanilla uses `spawnAtLocation` for both | Use the same spawn helper as the shed drop for consistent save semantics | S |
| 6 | armadillo.rs:46-52 state ids | `Armadillo$ArmadilloState.fromId` | Id mapping 0..3 matches declaration order; vanilla derives ids from the same enum order, so this is only a risk if the enum is reordered | Add a compile-time/comment anchor or a unit test tying ids to declaration order | S |

## Not gaps (checked)
State thresholds (`should_hide_in_shell` 5/26, durations 10/50/30), roll/unroll sounds + gating,
brush loot table `minecraft:brush/armadillo` + `ENTITY_INTERACT` game event + 16 durability,
shed loot table `minecraft:gameplay/armadillo_shed` + scute drop sound, NBT keys `state`/`scute_time`,
goal priorities (Swim 0, EscapeDanger 1, Breed 2, Tempt 3, FollowParent 4, Wander 5, LookAtPlayer 6,
RandomLookAround 7), food tag, step sound volume 0.15, `roll_up` gated on `!is_scared()`, and
`mob_interact` returning `PASS` while scared.
