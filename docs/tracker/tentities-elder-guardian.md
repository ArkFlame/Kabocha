# Elder Guardian - vanilla parity audit (RE-REVIEW, 2026-11-16)

Source: `crates/pumpkin-core/src/entity/mob/elder_guardian.rs` (plus shared `guardian.rs`,
`living.rs`). No decompiled vanilla tree is vendored in this worktree (`.vendor/` absent),
so references below cite the vanilla classes.

## Already matching
- Attributes (MAX_HEALTH 80, ATTACK_DAMAGE 8, MOVEMENT_SPEED 0.3), experience 10 and
  hitbox 1.9975/0.99875 are data-driven from `pumpkin-data/.../entity_type.rs`
  (`ELDER_GUARDIAN`), not set in Rust.
- `customServerAiStep`: every `(tickCount + id) % 1200 == 0`, mining fatigue
  amplifier 2 / 6000 ticks / 50 block radius plus `GUARDIAN_ELDER_EFFECT` game event.
  Matches `ElderGuardian.customServerAiStep`. (`elder_guardian.rs:85-110`)
- Prey set (player, squid, glow squid, axolotl; outside 3 blocks) matches
  `Guardian.GuardianAttackSelector`. (`guardian.rs:21-40`)

## Fixed in this pass
- `elder_guardian.rs:58-78` - hurt/death sounds had no `CustomSound` impl, so the
  data-driven land variants were used unconditionally. Now picks
  `ELDER_GUARDIAN_HURT/DEATH` vs `*_LAND` from `touching_water`, as in
  `ElderGuardian.getHurtSound/getDeathSound`.
- `living.rs:1535-1540` - guardians could receive mining fatigue; vanilla
  `Guardian.canBeAffected` returns false for `DIG_SLOWDOWN` for both guardian and
  elder guardian.

## Remaining gaps
1. **No guardian beam attack.** `Guardian.GuardianAttackGoal` + the beam state
   (`clientSideAttackTime`, `getAttackAnimationScale`, damage from ATTACK_DAMAGE
   every 80 ticks, `setTarget` requirement) has no Rust counterpart; `Mob for
   ElderGuardianEntity` only implements `on_damage`. Elder guardian never fires a
   beam, so its main damage source is missing.
   `elder_guardian.rs:80-119`, `guardian.rs:97-108`. Vanilla: `Guardian` (inner class
   `GuardianAttackGoal`). Fix: port the goal with the 80-tick charge, range 3..=16 and
   squared-distance > 9 gate, tick `attack_target`. Size M.
2. **No monument restriction movement.** Vanilla `Guardian.registerGoals` adds
   `MoveTowardsRestrictionGoal(1.0)` at priority 5 and `setRestriction` keeps the mob
   near its home; the Rust goal list (swim 0, wander 4, look 5/6) has no restriction
   goal, so guardians/elders drift away from the monument forever.
   `elder_guardian.rs:42-50`, `guardian.rs:59-69`. Fix: port
   `MoveTowardsRestrictionGoal` plus the restriction field, set it on first tick when
   `!has_restriction`. Size M.
3. **No `GuardianMoveControl`.** Vanilla guardians only move in water and float in
   air (`GuardianMoveControl`, `travel` override). Pumpkin uses the generic mob move
   control, so an elder guardian out of water walks. `elder_guardian.rs` (no move
   control), `guardian.rs` (same). Fix: implement water-only move control, reuse for
   both. Size M.
4. **Thorns reflect predicate is too broad.** `on_damage` always reflects 2 thorns
   damage. Vanilla `Guardian.hurt` requires: not creative player, direct entity
   present, direct entity not a guardian, damage type not in
   `minecraft:avoids_guardian_thorns`, and `random.nextFloat() < 0.5`.
   `elder_guardian.rs:112-118`, `guardian.rs:102-108`. Fix: add the tag check
   (`tag::DamageType::MINECRAFT_AVOIDS_GUARDIAN_THORNS` exists in pumpkin-data),
   creative and guardian checks, and the 0.5 roll. Size S.
5. **Appearance effect is sent to every nearby player.** Vanilla applies it through
   `MobEffectUtil.addEffectToPlayersAround`, which returns only the players that
   actually received the effect, and owns the `GUARDIAN_ELDER_EFFECT` packet to them;
   the Rust loop always sends the shake packet, even to players that already carry
   amplifier >= 2. `elder_guardian.rs:94-108`. Fix: gate packet + effect on the
   existing-effect amplitude rule. Size S.
6. **Ambient and flop sounds keep the land variant.** Data uses
   `ELDER_GUARDIAN_AMBIENT_LAND` and there is no flop hook, while vanilla overrides
   `getAmbientSound` (water/land split) and `getFlopSound`
   (`ELDER_GUARDIAN_FLOP`). `CustomSound` only exposes hurt/death
   (`crates/pumpkin-core/src/entity/custom_sound.rs:4-6`). Fix: extend `CustomSound`
   with `ambient_sound`/`flop_sound` and implement for guardian + elder guardian.
   Size S (trait change touches every impl).

## Gap status
- Remaining-gap items verified against `.vendor/vanilla-decompiled` on 2026-11-18; see Result.

## Result (re-verified 2026-11-18)

All six remaining gaps were checked against decompiled vanilla in this checkout:
`Guardian.java`, `ElderGuardian.java`, `world/effect/MobEffectUtil.java`, and the
`pumpkin-data` generated tracked-data/tag files. Gaps 1-5 are already implemented in
`guardian.rs` / `elder_guardian.rs` and match vanilla; no source edit was needed this pass.

1. Guardian beam attack: FIXED (present in checkout). `GuardianAttackGoal` mirrors
   `Guardian.GuardianAttackGoal`: attack_time starts at -10, at tick 0 it writes
   `DATA_ID_ATTACK_TARGET` and broadcasts `GuardianAttackSound`, at attack_duration it deals
   `INDIRECT_MAGIC` (1.0, +2 Hard, +2 elder) plus a melee swing, gated on line of sight and,
   for non-elders, squared distance > 9. The synced INT fields already exist in pumpkin-data
   for both mobs. Pure tests cover damage and charge duration.
2. Monument restriction: FIXED. `MoveTowardsRestrictionGoal::new(1.0)` sits at vanilla
   priority 5 on both mobs; the elder sets home via `set_home_to` in `custom_server_ai_step`
   when it has no position target, matching `ElderGuardian.customServerAiStep`.
3. GuardianMoveControl: VERIFIED. The custom control and `travel_in_water` are already in
   `guardian.rs` and used by both mobs; they match vanilla's wave push, look lerp and moving flag.
4. Thorns reflect: FIXED. `reflect_thorns` / `thorns_reflection_blocked` mirror
   `Guardian.hurtServer`: skip when moving or when the damage type is in
   `minecraft:avoids_guardian_thorns` (which already contains thorns, magic, fireworks,
   explosion, player_explosion and bad_respawn_point), then reflect 2.0 onto the living direct
   cause. Vanilla has no 0.5 roll and no guardian-attacker exclusion. The creative-player guard
   is retained because Pumpkin `damage()` lacks vanilla's creative immunity. Pure test covers
   the tag gate.
5. Appearance effect gating: VERIFIED. `should_apply_fatigue` matches
   `MobEffectUtil.addEffectToPlayersAround`: apply when there is no effect, amplifier < 2, or
   `endsWithin(displayEffectLimit - 1)` (remaining < 1200), survival/adventure players only.
   The `GUARDIAN_ELDER_EFFECT` packet is sent only to players that pass the gate, matching
   vanilla sending it only to the affected-player list it returns. Pure boundary test present.
6. Ambient/flop: the water/land ambient split is already implemented via `get_ambient_sound`
   overrides on both mobs and matches vanilla; no `CustomSound` change needed. Flop sound is
   still OPEN: vanilla plays `getFlopSound()` from client-side `Guardian.aiStep` on landing,
   and Pumpkin has no generic flop hook on the Mob tick (only fish have one), so adding it
   needs a file outside this unit. It is a client-only sound effect and does not block landing.

Files changed this pass: this doc only. No cargo/git run (per unit rules).
