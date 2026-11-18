# Parity rows for crates/pumpkin-core/src/entity/mob/guardian.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Guardian** [partial/RE-REVIEW] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.

## Result
## Result (THIRD PASS, independent line-by-line review against vendored vanilla)

Source of truth: `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Guardian.java`
(and `crates/pumpkin-data/src/generated/tag.rs` for the damage-type tag).

- **entities/Guardian**: VERIFIED. No source change needed; `guardian.rs` already matches vanilla.
  - Beam attack: `GuardianAttackGoal` (guardian.rs:125-245) matches `Guardian.java` GuardianAttackGoal
    (lines 347-429): `attack_time` starts -10, navigation stop + look at target (90, 90), LOS cancel,
    sets `DATA_ID_ATTACK_TARGET` at tick 0 and broadcasts `GuardianAttackSound` when not silent,
    charges to 80, deals `INDIRECT_MAGIC` with the guardian as both source and cause, calls
    `do_hurt_target`, and clears the target. `beam_magic_damage` = 1.0 +2 Hard +2 elder
    (Guardian.java:410-417). Target match (`GUARDIAN_PREY`/`outside_melee_range`, guardian.rs:28-47)
    matches `GuardianAttackSelector` (Guardian.java:431-442: Player/Squid/Axolotl, >9 sq; GlowSquid is
    a Squid subclass). The goal is registered first (Pumpkin 3, one lower-is-first selector),
    preserving vanilla's relative ordering (Guardian.java:76-81).
  - Water travel: `travel_in_water` (guardian.rs:449-459) matches `travelInWater`
    (Guardian.java:338-345): moveRelative(0.1), move SELF, uniform 0.9 drag, -0.005 y sink only while
    not moving and with no target.
  - GuardianMoveControl (guardian.rs:251-330) matches `GuardianMoveControl`
    (Guardian.java:444-490): MOVE_TO && !navigator.isDone, yaw rotlerp 90 (MAX_TURN), speed lerp 0.125,
    wave push with 0.05 scale and (tick+id) phase, getEyeY()+yd/length look-target quirk, 0.125 look
    lerp + setLookAt(...,10,40), setMoving(true)/else reset. The `DATA_ID_MOVING` synced flag is driven
    by the same AtomicBool; `guardian_look_target` snaps when not looking (unit-tested).
  - Thorns reflection: `reflect_thorns` (guardian.rs:49-79) matches `hurtServer`
    (Guardian.java:317-330): blocked while moving, and only from a living direct entity. Generated
    `MINECRAFT_AVOIDS_GUARDIAN_THORNS` (tag.rs:20050-20061) contains magic, thorns, fireworks,
    explosion, player_explosion, bad_respawn_point, so the tag check subsumes vanilla's explicit
    `DamageTypes.THORNS` test (unit-tested).
  - Synced data defaults, ambient/hurt/death sounds and ambient interval (guardian.rs:337-346,
    389-424, 443-445) match `defineSynchedData` (lines 97-101), `getAmbientSoundInterval` (152-154)
    and the land/water sound selectors (156-169).
  - No generated-data field, WIT or cross-crate change was required. The existing pure unit tests in
    guardian.rs remain the orchestrator's compile/test gate. Static verification only: worker shell
    and compiler are denied.
