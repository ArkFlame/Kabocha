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
| 1 (FIXED UNVERIFIED: gated on `should_hide_in_shell()`) | armadillo.rs:402 `modify_incoming_damage` | `Armadillo.hurtServer` | Reduction is applied for the whole threatened state (`is_scared()`), including Rolling tick 1 and the last Unrolling ticks, instead of only while `should_hide_in_shell()`; formula `(amount - 1).max(0) / 2` is unverified | Gate on `self.should_hide_in_shell()` and confirm the arithmetic against the decompiled `hurtServer` (verify) | S |
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

## h6 gap status

- Rows 1-5: STILL OPEN (UNVERIFIED, need vanilla Armadillo source).
- Row 6 state-id test: STILL OPEN (S, test deferred; id order is a unit-testable constant mapping).

## Result

All six STILL OPEN rows verified against decompiled vanilla (`.vendor/vanilla-decompiled/net/minecraft/`). No generated data, WIT or cross-crate changes were needed. Only `armadillo.rs` was edited; `ageable.rs` was read only.

- **Row 1 FIXED.** `Armadillo.hurtServer` (lines 280-286) gates on `isScared()`, not `shouldHideInShell()`, and uses `(damage - 1.0F) / 2.0F` with no `.max(0)` clamp. `modify_incoming_damage` now uses `self.is_scared()` and `(amount - 1.0) / 2.0`.
- **Row 2 FIXED (cadence).** The threat scan is `SensorType.armadillo_scare_detected = MobSensor<>(5, Armadillo::isScaredBy, Armadillo::canStayRolledUp, DANGER_DETECTED_RECENTLY, 80)` (SensorType.java:27). Vanilla rate is 5 ticks, not per-tick and not 80. Added `SCARE_SCAN_INTERVAL = 5` and switched the modulo to it; the 80-tick `SCARE_CHECK_INTERVAL` is kept as the danger-memory lifetime. Residual (MVP, not this row): the scan is still `Idle`-only and does not refresh the 80-tick counter the way vanilla's sensor refreshes `DANGER_DETECTED_RECENTLY` every 5 ticks while a threat is present, so a threat that persists after rolling up does not hold the armadillo scared indefinitely.
- **Row 3 FIXED.** `Armadillo.actuallyHurt` (lines 290-305) branch order and counter writes already matched: living attacker stores 80 ticks then rolls up only via `canStayRolledUp`/`rollUp`'s own `!isScared` gate; panic-environmental rolls out and leaves the counter alone; non-living non-panic sources do nothing. Added the missing `!this.isNoAi()` guard (via `self.mob_entity.is_no_ai()`).
- **Row 4 NO CHANGE (verified already correct).** `EntityBase::is_passenger()` in `entity/mod.rs:1079` is implemented as `self.get_entity().has_vehicle()`, so Pumpkin `has_vehicle()` is exactly vanilla `isPassenger()`. The `!living_entity.is_spectator()` exclusion is already present at armadillo.rs:299, matching vanilla's `!player.isSpectator()`.
- **Row 5 FIXED (assert).** `ageable_mob::DATA_BABY_ID` and `armadillo::BABY_ID` are both `TrackedId(16u8)` / `BOOLEAN` in the generated `tracked_data.rs`. Added a compile-time `const` assert on their ids; kept the hand-set with a comment.
- **Row 6 FIXED.** Added a pure `#[cfg(test)]` test binding ids 0..3 to Idle/Rolling/Scared/Unrolling and to the NBT names idle/rolling/scared/unrolling, plus `from_id`/`from_name` round-trips.

Additional finding during verification: the doc's "Not gaps" claim that `CustomSound::hurt_sound` matches `getHurtSound` was wrong. Vanilla `getHurtSound` (line 358) gates on `isScared()`, not `shouldHideInShell()`. Changed `hurt_sound()` to `self.is_scared()`; `None` still falls back to the normal hurt sound.

`rustfmt --edition 2024` ran successfully on `armadillo.rs`. No cargo/git run per unit rules; compilation left to the orchestrator.
