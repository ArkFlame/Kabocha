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

## Status (h2 pass)

- 5 brush/shed scute spawn helper mismatch: **FIXED** (d712d13fa), shed scute now `spawn_entity` (saved), matching `spawnAtLocation`. Note: the tracker suggested the opposite helper, but vanilla saves both drops.
- 2 dead `can_stay_rolled_up` branch: **STILL OPEN** (not reached in this pass).
- 3 shell reduction gate: **FIXED (UNVERIFIED)**, reduction gated on `should_hide_in_shell()` in `modify_incoming_damage`; formula unverified against decompiled `hurtServer`.
- 4 redundant baby flag: **STILL OPEN** (not reached in this pass).


## Result (CODER pass, 2026-11-17)

Vanilla reference: decompiled `net.minecraft.world.entity.animal.armadillo.Armadillo` and
`ArmadilloAi` from the Paper upstream cache (no `.vendor/vanilla-decompiled` copy exists).

- **2 dead `can_stay_rolled_up` branch: VERIFIED, no code change.** Current `on_damage` matches
  decompiled `actuallyHurt`: the 80-tick `DANGER_DETECTED_RECENTLY` lifetime is stored only for
  living-attacker sources (vanilla `source.getEntity() instanceof LivingEntity`), then
  `roll_up()` is gated on `can_stay_rolled_up()`. Vanilla does not zero the memory when
  `!canStayRolledUp()`. `SCARE_CHECK_INTERVAL` (80) is the documented memory lifetime; the 5-tick
  probe is the separate `SCARE_SCAN_INTERVAL`.
- **4 redundant baby flag: FIXED.** Removed the hand-set `DATA_BABY_ID` write from
  `mob_init_data_tracker`; `AgeableMob::set_age` already syncs the flag on every age boundary and
  is the only writer of armadillo `age`. Kept the compile-time assert tying `armadillo::BABY_ID`
  to `ageable_mob::DATA_BABY_ID`.
- **3 shell damage formula: VERIFIED (bonus), no code change.** Decompiled `hurtServer` is exactly
  `if (isScared()) damage = (damage - 1.0F) / 2.0F;`, matching current `modify_incoming_damage`.
  The tracker's suggested `should_hide_in_shell()` gate and `.max(0.0)` do not match vanilla.
- **1 proximity scan: already implemented in source** (5-tick `get_all_at_box` probe using
  `is_scared_by`, matching the vanilla sensor). **6 state-id mapping: already guarded** by
  `state_ids_and_names_match_vanilla_order`. Added pure test `shell_thresholds_match_vanilla`.
- No generated-data, WIT, cross-crate changes or files outside this unit.


## Result (CODER re-review, 2026-11-18)

Decompiled vanilla is now present at .vendor/vanilla-decompiled/net/minecraft/world/entity/animal/armadillo/Armadillo.java; all rows re-verified directly against it.

- 1 proximity scan: VERIFIED, no change. The 5-tick get_all_at_box probe in custom_server_ai_step matches the vanilla armadillo_scare_detected sensor; is_scared_by covers all three vanilla conditions (UNDEAD tag, getLastHurtByMob() == candidate already in source, sprinting/passenger player). Vanilla bounding-box inflate 7/2/7 is approximated by centre distances.
- 2 dead can_stay_rolled_up branch: VERIFIED, no change. on_damage matches vanilla actuallyHurt: the 80-tick danger memory is stored only for living-attacker sources and roll_up() is gated on can_stay_rolled_up(); vanilla never zeroes the memory.
- 3 shell damage: VERIFIED, no change. modify_incoming_damage is exactly vanilla hurtServer: if (isScared()) damage = (damage - 1.0F) / 2.0F; - no should_hide_in_shell gate, no .max(0.0).
- 4 redundant baby flag: VERIFIED, no change. mob_init_data_tracker sets only ARMADILLO_STATE; AgeableMob::set_age syncs DATA_BABY_ID on each age-boundary crossing, and the compile-time assert ties armadillo::BABY_ID to ageable_mob::DATA_BABY_ID (both id 16).
- 5 scute spawn helper: VERIFIED, no change. Brush and shed drops both use world.spawn_entity, matching vanilla spawnAtLocation (saved).
- 6 state ids: VERIFIED, no change. state_ids_and_names_match_vanilla_order and shell_thresholds_match_vanilla pin ids 0..3 and thresholds 5/26 to the vanilla enum order.
- Bonus fix: get_ambient_sound now returns None while scared, matching vanilla getAmbientSound() returning null; previously it always played the ambient sound, leaving the tracker's ambient-sound fix incomplete.

No generated-data, WIT or cross-crate changes; only armadillo.rs touched (rustfmt clean). No pure test seam exists for sound/AI behaviour; existing state tests cover the pure logic.
## Result (CODER BRIEF2, 2026-11-18)
Checked each row against `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/armadillo/Armadillo.java` and `.../ai/sensing/SensorType.java`. No Rust edits were needed.
- 1 proximity scan: VERIFIED. Vanilla `SensorType.ARMADILLO_SCARE_DETECTED` is `MobSensor<>(5, isScaredBy, canStayRolledUp, DANGER_DETECTED_RECENTLY, 80)` (SensorType.java:26-28). Rust scans on `age % SCARE_SCAN_INTERVAL` with interval 5 (armadillo.rs:35, 489-502); `is_scared_by` covers undead, last-hurt-by, and non-spectator sprinting or riding players (armadillo.rs:287-326, vanilla Armadillo.java:238-245). Residual, not a row: the scan is Idle-only and does not refresh the 80-tick memory while a threat persists, as vanilla's sensor does. Left open.
- 2 dead `can_stay_rolled_up` branch: VERIFIED. Vanilla `actuallyHurt` (Armadillo.java:290-305) stores 80 ticks for living attackers and gates `rollUp` on `canStayRolledUp`; Rust `on_damage` (armadillo.rs:431-446) matches, including the `!isNoAi && !isDeadOrDying` guard.
- 3 shell damage formula: VERIFIED. Vanilla `hurtServer` (Armadillo.java:280-286) is `if isScared() damage = (damage - 1.0F) / 2.0F`; Rust `modify_incoming_damage` (armadillo.rs:414-429) matches, with no `should_hide_in_shell` gate and no clamp.
- 4 redundant baby flag: VERIFIED. Rust `mob_init_data_tracker` (armadillo.rs:534-543) sets only `ARMADILLO_STATE`; baby sync is `ageable.rs:115`. Compile-time assert ties `armadillo::BABY_ID` to `ageable_mob::DATA_BABY_ID` (armadillo.rs:562-566).
- 5 scute spawn helper: VERIFIED. Vanilla uses `spawnAtLocation` for brush (Armadillo.java:324) and shed (line 144). Rust uses `world.spawn_entity` for both (armadillo.rs:276, 472), so the saved-drop semantics match. Note: `tentities-armadillo-d4b8f2.md` says the shed drop uses `spawn_entity_non_save`; that is stale against current code.
- 6 state ids: VERIFIED. Vanilla enum order IDLE 0, ROLLING 1, SCARED 2, UNROLLING 3 (Armadillo.java:384-402). Pure test `state_ids_and_names_match_vanilla_order` covers ids and names (armadillo.rs:571-586).
- Bonus check: `get_ambient_sound` returns `None` while scared (armadillo.rs:377-380), matching vanilla `getAmbientSound` (Armadillo.java:342-344).
Files edited: none. rustfmt not run (no Rust touched). No cargo or git run per brief.
