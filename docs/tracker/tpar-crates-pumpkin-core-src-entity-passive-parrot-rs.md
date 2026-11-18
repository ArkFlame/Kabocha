# Parity rows for crates/pumpkin-core/src/entity/passive/parrot.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-en-a78cb7** [partial/OWN-OPEN] own audit: 2 open, 0 fixed, 2 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-en-a78cb7.md) docs/tracker/taudit-crates-pumpkin-core-src-en-a78cb7.md

## Result

Vanilla reference: .vendor/vanilla-decompiled/net/minecraft/world/entity/animal/parrot/Parrot.java

- Audit row 1 (imitate EMPTY, parrot.rs:452): VERIFIED MATCHING. Vanilla imitateNearbyMobs
  (Parrot.java:234-249) filters with NOT_PARROT_PREDICATE = MOB_SOUND_MAP.containsKey, which
  includes HAPPY_GHAST -> SoundEvents.EMPTY; it plays getImitatedSound(type) at 0.7 volume and
  returns true with no EMPTY skip. Pumpkin imitate_nearby matches. No change.
- Audit row 2 (server-side flap/descent, parrot.rs:343-363): VERIFIED MATCHING. Vanilla
  calculateFlapping (Parrot.java:216-232) runs from aiStep with no isClientSide guard;
  flapSpeed += (onGround||passenger ? -1 : 4) * 0.3, clamp 0..1, and
  !onGround && movement.y < 0 -> multiply(1.0, 0.6, 1.0) all match Pumpkin mob_ai_step.
  Flap-phase fields are render-only, exposed via mob_is_flapping/mob_on_flap. No change.
- Bonus parity fix: FIXED. Vanilla getAmbient (Parrot.java:333-340) selects a random key from
  the whole MOB_SOUND_MAP.keySet(), including HAPPY_GHAST -> EMPTY, so shoulder-parrot ambient
  can be silent. ambient_candidates now maps all IMITATIONS including Sound::IntentionallyEmpty
  (was wrongly filtering HAPPY_GHAST); unit test ambient_imitation_includes_the_happy_ghast_silent_entry
  covers it. crates/pumpkin-core/src/entity/passive/parrot.rs.
- No generated data, WIT, or cross-crate change needed. All rows closed; none STILL OPEN.

Status: PASS fixed=1 verified-no-change=2 open=0 files=crates/pumpkin-core/src/entity/passive/parrot.rs
