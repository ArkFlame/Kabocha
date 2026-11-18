# Parity rows for crates/pumpkin-core/src/entity/mob/elder_guardian.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Elder Guardian** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/tentities-guardian** [partial/OWN-OPEN] own audit: 8 open, 8 fixed, 10 verified, 0 blocked-data (docs/tracker/tentities-guardian.md) docs/tracker/tentities-guardian.md

## Result
See the Result sections below. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field.


## Result (2026-11-18, re-review of changed source against vendored decompiled vanilla)

- **entities/Elder Guardian**: FIXED + VERIFIED. Re-read `crates/pumpkin-core/src/entity/mob/elder_guardian.rs` against `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/ElderGuardian.java` and `.vendor/vanilla-decompiled/net/minecraft/world/effect/MobEffectUtil.java`.
  - FIXED: vanilla's `ElderGuardian` constructor calls `setPersistenceRequired()` (ElderGuardian.java:29), so elders never despawn. Pumpkin's `new()` left `persistence_required` false (default at `mob/mod.rs:322`; elder is not in `requires_custom_persistence`). Added `mob_entity.set_persistence_required()` in `new()`.
  - VERIFIED against ElderGuardian.java: ambient/hurt/death sounds (lines 45-57), inherited 160-tick ambient interval, `getAttackDuration()` = 60 (lines 40-42), and `customServerAiStep` (lines 65-79): `(tick+id)%1200`, radius 50, MINING_FATIGUE duration 6000 / amplifier 2, reapply when amplifier<2 or remaining<1200 (`MobEffectUtil.java:92-105` = `endsWithin(1199)`), survival/adventure-only players, game-event amplitude `isSilent()?0:1`, home restriction 16 (lines 76-78). Water travel, thorns reflection and the beam goal were unchanged and already verified. Pure test `fatigue_application_matches_vanilla_display_limit` remains the gate.
  - Non-blocking notes (no generated-data/WIT/cross-crate change): `getFlopSound()` -> `ELDER_GUARDIAN_FLOP` (ElderGuardian.java:60-62) is a client-only sound and the `Mob` trait has no flop-sound hook; the swim goal keeps Pumpkin's established `RandomSwimmingGoal(1.0, 10)` water-mob convention instead of vanilla's `RandomStrollGoal(1.0, 80)` / `setInterval(400)` (different goal type, prior-verified); goal priorities are renumbered but relative lower-is-first order is unchanged.
- **own/tentities-guardian**: VERIFIED. All six gaps in `docs/tracker/tentities-guardian.md` are already FIXED/VERIFIED in its four Result sections; no STILL OPEN rows remain there and no source change was needed this pass.
