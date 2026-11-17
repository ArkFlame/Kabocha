# entities / Armadillo — audit (2026-11-17)

Source: `crates/pumpkin-core/src/entity/passive/armadillo.rs`.
Reference: vanilla `net.minecraft.world.entity.animal.armadillo.Armadillo` and
`Armadillo$ArmadilloState`. No decompiled sources in `.vendor/`; each row names the vanilla
member it must match and is therefore marked "verify" where the exact Java shape is unsure.

## Why this row was re-opened

c079e26b7 ("docs(tracker): re-review birch boat parity gaps") was a docs-labelled commit that
also mass-reverted code across many entities. For the armadillo it deleted the nearby-threat
scan and the roll-up on damage, while the previous tracker still claimed the scan was present.
e9e63bd6e later restored the damage path; the scare scan was still missing.

## Fixed this pass

- **Nearby threats never triggered `rollUp`.** `is_scared_by` (armadillo.rs:296) had no caller
  after the revert. Restored the idle scan in `custom_server_ai_step` (armadillo.rs:453):
  every `SCARE_CHECK_INTERVAL` ticks while `Idle`, query `World::get_nearby_entities` with the
  bounding-sphere radius of vanilla's `inflate(7.0, 2.0, 7.0)` box and roll up on the first
  living entity that `is_scared_by` accepts.
- Ambient sound (`Mob::get_ambient_sound`, armadillo.rs:366) and the shell-muted hurt sound
  (`CustomSound::hurt_sound`, armadillo.rs:349) are present and match `Armadillo.getAmbientSound`
  / `Armadillo.getHurtSound`. No action.

## Remaining gaps

| # | Where | Vanilla member | Gap | Fix | Size |
|---|-------|----------------|-----|-----|------|
| 1 | armadillo.rs:402 `modify_incoming_damage` | `Armadillo.hurtServer` | Reduction is applied for the whole threatened state (`is_scared()`), including Rolling tick 1 and the last Unrolling ticks, instead of only while `should_hide_in_shell()`; formula `(amount - 1).max(0) / 2` is unverified | Gate on `self.should_hide_in_shell()` and confirm the arithmetic against the decompiled `hurtServer` (verify) | S |
| 2 | armadillo.rs:453-468 scare scan | `Armadillo.customServerAiStep` | Scan runs on `age % 80`; vanilla may instead scan every tick while `dangerDetectedRecentlyTicks == 0`. AABB `inflate(7, 2, 7)` is approximated by a bounding sphere plus the per-entity check in `is_scared_by` (equivalent within rounding) | Confirm the vanilla scan cadence; drop the modulo if it is per-tick (verify) | S |
| 3 | armadillo.rs:438 `on_damage` | `Armadillo.actuallyHurt` | Any living attacker stores 80 ticks and rolls up, even when already threatened; vanilla may zero the counter for non-living sources and only roll up from `Idle`. Panic branch rolls out but does not touch the counter | Confirm `actuallyHurt` branch order and counter writes against decompiled source | M |
| 4 | armadillo.rs:296-312 `is_scared_by` | `Armadillo.isScaredBy` | Player test uses `is_sprinting() || has_vehicle()`; vanilla checks sprinting or *riding* (`isPassenger()`) and excludes spectators. `has_vehicle()` semantics in Pumpkin may cover it, needs a check | Verify `Entity::has_vehicle` vs `is_passenger` and add a spectator exclusion if vanilla has one | S |
| 5 | armadillo.rs:497 `mob_init_data_tracker` | `AgeableMob.defineSynchedData` | Baby flag is set by hand from `entity.age < 0` using `armadillo::BABY_ID`; `ageable.rs` already syncs `DATA_BABY_ID`. Harmless if the indices match, wrong if they diverged | Assert the two constants are the same index and drop the hand-set, or keep it with a comment | S |
| 6 | armadillo.rs:44-68 state ids | `Armadillo$ArmadilloState` (`fromId`/`id`) | Ids come from declaration order; a reorder silently corrupts network/NBT state | Add a unit test binding ids 0..3 to the names written to NBT | S |

## Not gaps (checked)

State thresholds (`should_hide_in_shell` 5/26, durations 10/50/30), roll/unroll sounds and their
gating, brush loot table `minecraft:brush/armadillo`, `ENTITY_INTERACT` game event and 16
durability, shed loot table `minecraft:gameplay/armadillo_shed` plus scute-drop sound, NBT keys
`state`/`scute_time`, goal priorities (Swim 0, EscapeDanger 1, Breed 2, Tempt 3, FollowParent 4,
Wander 5, LookAtPlayer 6, RandomLookAround 7), food tag `minecraft:armadillo_food`, step sound
volume 0.15, `roll_up` gated on `!is_scared()`, `mob_interact` returning `PASS` while scared,
`can_stay_rolled_up` (panic, water, leash, passenger, vehicle).

Brush drop uses `spawn_entity` while the shed drop uses `spawn_entity_non_save`; both create
tracked, persisted `ItemEntity`s, so only the chunk-save trigger differs from vanilla
`spawnAtLocation`. Left as-is.
