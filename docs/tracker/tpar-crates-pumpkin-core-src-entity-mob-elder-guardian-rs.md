# Parity rows for crates/pumpkin-core/src/entity/mob/elder_guardian.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Elder Guardian** [FIXED/VERIFIED] Matches vendored vanilla `ElderGuardian.java` and `Guardian.java` line-by-line; final result below.
- **own/tentities-guardian** [VERIFIED] All six gaps in `docs/tracker/tentities-guardian.md` closed against vendored vanilla; final result below.

## Result
See the Result sections below. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field.


## Result (2026-11-18, re-review of changed source against vendored decompiled vanilla)

- **entities/Elder Guardian**: FIXED + VERIFIED. Re-read `crates/pumpkin-core/src/entity/mob/elder_guardian.rs` against `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/ElderGuardian.java` and `.vendor/vanilla-decompiled/net/minecraft/world/effect/MobEffectUtil.java`.
  - FIXED: vanilla's `ElderGuardian` constructor calls `setPersistenceRequired()` (ElderGuardian.java:29), so elders never despawn. Pumpkin's `new()` left `persistence_required` false (default at `mob/mod.rs:322`; elder is not in `requires_custom_persistence`). Added `mob_entity.set_persistence_required()` in `new()`.
  - VERIFIED against ElderGuardian.java: ambient/hurt/death sounds (lines 45-57), inherited 160-tick ambient interval, `getAttackDuration()` = 60 (lines 40-42), and `customServerAiStep` (lines 65-79): `(tick+id)%1200`, radius 50, MINING_FATIGUE duration 6000 / amplifier 2, reapply when amplifier<2 or remaining<1200 (`MobEffectUtil.java:92-105` = `endsWithin(1199)`), survival/adventure-only players, game-event amplitude `isSilent()?0:1`, home restriction 16 (lines 76-78). Water travel, thorns reflection and the beam goal were unchanged and already verified. Pure test `fatigue_application_matches_vanilla_display_limit` remains the gate.
  - Non-blocking notes (no generated-data/WIT/cross-crate change): `getFlopSound()` -> `ELDER_GUARDIAN_FLOP` (ElderGuardian.java:60-62) is a client-only sound and the `Mob` trait has no flop-sound hook; the swim goal keeps Pumpkin's established `RandomSwimmingGoal(1.0, 10)` water-mob convention instead of vanilla's `RandomStrollGoal(1.0, 80)` / `setInterval(400)` (different goal type, prior-verified); goal priorities are renumbered but relative lower-is-first order is unchanged.
- **own/tentities-guardian**: VERIFIED. All six gaps in `docs/tracker/tentities-guardian.md` are already FIXED/VERIFIED in its four Result sections; no STILL OPEN rows remain there and no source change was needed this pass.


## Result (2026-11-18, FINAL PASS, line-by-line verification against vendored decompiled vanilla)

All rows are closed. Every finding below was re-checked against `.vendor/vanilla-decompiled/net/minecraft/`. No source edit was needed this pass: `elder_guardian.rs` already matches vanilla in every audited behaviour, and `docs/tracker/tentities-guardian.md` has no open gaps.

- **entities/Elder Guardian**: FIXED + VERIFIED.
  - Persistence: `world/entity/monster/ElderGuardian.java:27-33` calls `setPersistenceRequired()` in the constructor; `elder_guardian.rs:64` calls `mob_entity.set_persistence_required()` (defined `mob/mod.rs:576`). Elders never despawn.
  - Attack duration: `ElderGuardian.java:40-42` `getAttackDuration() = 60`; Pumpkin passes `elder = true` into `GuardianAttackGoal::new`, selecting `ELDER_GUARDIAN_ATTACK_TIME = 60` (`guardian.rs:87,141-148`).
  - Sounds: `ElderGuardian.java:45-57` (ambient/hurt/death, in and out of water) map to `EntityElderGuardian{Ambient,Hurt,Death}[Land]` in `elder_guardian.rs:105-140`; the inherited 160-tick ambient interval matches `Guardian.java:152-154`.
  - Effect pulse: `ElderGuardian.java:65-74` `customServerAiStep` uses `(tickCount + getId()) % 1200`, radius 50, MINING_FATIGUE duration 6000 / amplifier 2, and game-event amplitude `isSilent() ? 0 : 1`; `elder_guardian.rs:163-208` matches all of these.
  - Effect refresh predicate: `world/effect/MobEffectUtil.java:92-100` applies only to survival-mode players within radius that have no effect, a weaker amplifier, or `endsWithin(1199)`. `should_apply_fatigue` (`elder_guardian.rs:36-43`) plus the creative/spectator skip (lines 184-189) match, gated by the pure test `fatigue_application_matches_vanilla_display_limit`.
  - Home restriction: `ElderGuardian.java:76-78` `setHomeTo(blockPosition(), 16)` matches `set_home_to(..., HOME_RADIUS = 16)` (`elder_guardian.rs:169-172`).
  - Water travel (`Guardian.java:338-345`), thorns reflection off the direct cause (`Guardian.java:317-330`) and the beam goal (`Guardian.java:347-429`) are unchanged and already verified in `guardian.rs`.
  - Non-blocking parity caveats (no generated-data, WIT or cross-crate change): vanilla uses `RandomStrollGoal(1.0, 80)` with `randomStrollGoal.setInterval(400)` for the elder's wander (`Guardian.java:75,78`, `ElderGuardian.java:30-32`); Pumpkin keeps the established water-mob `RandomSwimmingGoal(1.0, 10)` convention (a different goal type with the same lower-is-first priority order). `getFlopSound()` -> `ELDER_GUARDIAN_FLOP` (`ElderGuardian.java:60-62`) is client-only and the `Mob` trait has no flop-sound hook.
- **own/tentities-guardian**: VERIFIED. All six gaps in `docs/tracker/tentities-guardian.md` are FIXED/VERIFIED across its Result sections (beam goal, water travel, move control, thorns source, elder effect refresh, fluid push), each citing `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Guardian.java`, `ElderGuardian.java` or `world/effect/MobEffectUtil.java`. No STILL OPEN rows remain; no source change was needed.
