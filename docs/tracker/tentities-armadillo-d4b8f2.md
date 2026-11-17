# entities / Armadillo — re-review (post 17b4edd4d)

Source: `crates/pumpkin-core/src/entity/passive/armadillo.rs`.
Reference: vanilla `net.minecraft.world.entity.animal.armadillo.Armadillo` and
`Armadillo$ArmadilloState` (1.21.5+). No decompiled sources in `.vendor/`; each row names the
vanilla member it must match.

## Fixed this pass
- **Scare scan was dead code.** `is_scared_by` (armadillo.rs:274) had no caller, so an armadillo
  never rolled up on approach, only after being damaged. Added the vanilla idle check in
  `custom_server_ai_step` (armadillo.rs:439): every `SCARE_CHECK_INTERVAL` (80) ticks while
  `Idle`, scan `world.get_nearby_entities(pos, SCARE_DISTANCE_HORIZONTAL)` and `roll_up()` on the
  first living entity that `is_scared_by`.

## Remaining gaps

| # | Where | Vanilla member | Gap | Fix | Size |
|---|-------|----------------|-----|-----|------|
| 1 | armadillo.rs:391-403 `on_damage` | `Armadillo.hurtServer` | Both branches store 80, so the `can_stay_rolled_up()` check is dead; `SCARE_CHECK_INTERVAL` is reused as a duration, not an interval | Collapse to a single store and confirm against decompiled `hurtServer` whether a non-rolled-up armadillo should zero/keep the counter | S |
| 2 | armadillo.rs:383-389 `modify_incoming_damage` | shell damage handling | Reduction applies whenever `is_scared()` (Rolling from tick 1, Scared, Unrolling to the last tick) instead of only once `should_hide_in_shell()` is true | Gate on `should_hide_in_shell()`; verify clamp formula `(amount - 1.0).max(0.0) / 2.0` against vanilla | S |
| 3 | armadillo.rs:200-205 `should_switch_to_scared_state` | `Armadillo.shouldSwitchToScaredState` | Defined but unused, and the state machine inlines `in_state_ticks > animation_duration()` (10) for Rolling->Scared while `should_hide_in_shell` uses >5; if vanilla threshold is 5 the transition is 5 ticks late | Reuse the helper in the Rolling arm and align its threshold with `should_hide_in_shell` | S |
| 4 | armadillo.rs:469-479 `mob_init_data_tracker` | `AgeableMob.defineSynchedData` | Baby flag hand-set from `entity.age < 0` duplicates the ageable sync (ageable.rs:115) and uses a second constant | Drop the hand-set line and rely on the ageable sync | S |
| 5 | armadillo.rs:237-272 `brush_off_scute` | `Armadillo.brushOffScute` | Brush drop uses `world.spawn_entity` (saved) while the shed drop (armadillo.rs:412) uses `spawn_entity_non_save`; vanilla uses `spawnAtLocation` for both | Pick one helper matching `spawnAtLocation` save semantics | S |

## Not gaps (checked)
State ids/name round trip, `should_hide_in_shell` 5/26, durations 10/50/30, roll/unroll sounds and
gating, shoot/brush loot tables `minecraft:brush/armadillo` and `minecraft:gameplay/armadillo_shed`,
scute cooldown `nextInt(6000)+6000` with continued counting while loot is blocked,
`ENTITY_INTERACT` game event and 16 brush durability, NBT keys `state`/`scute_time`,
`ARMADILLO_SPAWNABLE_ON` + bright-enough spawn rule (type.rs:640), biome spawn lists in
pumpkin-data, ambient/death/step sounds and shell-muted hurt sound falling back to
`EntityType::ARMADILLO.hurt_sound`, goal priorities (Swim 0 ... RandomLookAround 7), food tag,
and `mob_interact` refusing interaction while scared.
