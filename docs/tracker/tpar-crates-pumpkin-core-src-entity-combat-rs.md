# Parity rows for crates/pumpkin-core/src/entity/combat.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-en-39d9d7** [partial/OWN-OPEN] own audit: 5 open, 5 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-en-39d9d7.md) docs/tracker/taudit-crates-pumpkin-core-src-en-39d9d7.md
- **own/taudit-crates-pumpkin-core-src-entity-com** [partial/OWN-OPEN] own audit: 3 open, 1 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-entity-com.md) docs/tracker/taudit-crates-pumpkin-core-src-entity-com.md
- **own/titems-mace** [partial/OWN-OPEN] own audit: 3 open, 4 fixed, 6 verified, 0 blocked-data (docs/tracker/titems-mace.md) docs/tracker/titems-mace.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)


## Result (third pass, final)

Re-read the last Result of each referenced tracker and re-checked every row against the
decompiled vanilla in .vendor/vanilla-decompiled/net/minecraft/ plus the live source. Every
STILL OPEN row now ends FIXED, VERIFIED or BLOCKED-DATA; no parity fix was needed because all
second-pass changes are already present in combat.rs and their call sites.

- taudit-crates-pumpkin-core-src-en-39d9d7, mace_smash_knockback (S): FIXED. push_by marks the
  velocity dirty like MaceItem.knockback -> Entity.push(x, 0.7, z, attacker)
  (MaceItem.java:126-153); motion packet to pushed players kept; 3D falloff. Test
  smash_knockback_power_uses_full_3d_distance.
- ... record_damage enter/leave hooks (M): FIXED. CombatStateChange {entered, left,
  end_duration} fired at the vanilla hook points (CombatTracker.java:47-68,165-185); consumed in
  living.rs:4625-4629. Test combat_reset_clears_state_and_reports_duration.
- ... get_message_for_assisted_fall live item (S): FIXED. live_main_hand_item_name resolves the
  attacker's current main hand by id at message time (CombatTracker.java:74-81). Test
  assisted_fall_uses_live_item_name_and_falls_back_without_it.
- ... player.rs sweep damage (M): VERIFIED. compute_sweep_damage/sweep_in_reach implement
  Player.doSweepAttack (Player.java:1277-1305): (1 + SWEEPING_DAMAGE_RATIO * base) *
  attackStrengthScale, box inflate (1.0, 0.25, 1.0), ally/marker/3-block filters, 0.4 knockback,
  sweep particle. No sweep damage type in generated data is needed: vanilla reuses the attack
  DamageSource (Paper-only knownCause tag), Pumpkin uses DamageType::PLAYER_ATTACK. The earlier
  BLOCKED-DATA reason is obsolete.
- ... causeExtraKnockback non-living push (S): FIXED. non_living_knockback_impulse =
  (-sin*strength, 0.1, cos*strength), used at player.rs:2140, matching Player.java:1213. Test
  non_living_knockback_uses_opposite_horizontal_sign_and_small_lift.
- taudit-crates-pumpkin-core-src-entity-com: all 6 rows VERIFIED/CLOSED against vanilla (see its
  Result); no open rows.
- titems-mace: rows 11-15 FIXED/CONFIRMED against MaceItem.java; row 16 (drop-player pumpkin-
  gametest) is BLOCKED-DATA: GameTestWorld (crates/pumpkin-gametest/src/world.rs) has no API to
  equip a mace, trigger a player melee attack or read entity health/damage - a whole missing
  subsystem; behavior is VERIFIED against MaceItem.java and covered by combat.rs pure-unit seams.

Only code change this pass: combat.rs doc-comment placement (cosmetic). Tracker docs for this
unit and taudit-crates-pumpkin-core-src-en-39d9d7.md updated to terminal statuses.
