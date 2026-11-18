# Tracker: entities/Guardian (RE-REVIEW)

Sources reviewed (2026-11-16):
- crates/pumpkin-core/src/entity/mob/guardian.rs
- crates/pumpkin-core/src/entity/mob/elder_guardian.rs
- crates/pumpkin-data/src/generated/entity_type.rs (GUARDIAN attributes, ambient sounds)
No decompiled vanilla tree is vendored in this worktree; comparison is against
`net.minecraft.world.entity.monster.Guardian` / `ElderGuardian` (1.21.x).

## Fixed in this pass
- guardian.rs:59 / elder_guardian.rs:41 - goals used `FloatGoal` (SwimGoal) at 0 and
  `RandomStrollGoal` (WanderAroundGoal) at 4. Vanilla has no float goal and uses
  `RandomSwimmingGoal(this, 1.0, 10)` at 4. Replaced with `RandomSwimmingGoal::new(1.0, 10)`,
  removed the float goal.
- guardian.rs:101 - `getAmbientSound()` was absent, so guardians played no ambient sound
  (entity_type ambient sound is not consulted by `Mob::play_ambient_sound`). Vanilla returns
  `GUARDIAN_AMBIENT` in water and `GUARDIAN_AMBIENT_LAND` otherwise. Added the override.
- elder_guardian.rs:67 - vanilla `ElderGuardian#customServerAiStep` calls
  `restrictTo(blockPosition(), 16)` when `!hasRestriction()`. Added via
  `MobEntity::set_home_to(..., 16)`, which `random_pos`/`melee_attack` honour.

## Remaining gaps
1. **Beam attack not implemented** (L). guardian.rs:46-73 registers no attack goal; vanilla
   `Guardian$GuardianAttackGoal` at priority 0: stops navigation, looks at target, charges for
   20 ticks, sets `DATA_ID_ATTACK_TARGET` metadata (the client beams), then deals 2.0
   `indirectMagic` damage and clears the target when line of sight is lost. Needs: new goal
   file, synced entity metadata index (guardian attack target), damage
   `DamageType::INDIRECT_MAGIC` with source = guardian, and cancel-on-hurt clearing.
2. **Vanilla water travel missing** (M). guardian.rs has no `Mob::travel_in_water` override;
   vanilla `Guardian#travel` does `moveRelative(speed)` + `move(SELF)`, scales delta by 0.9 and
   adds -0.005 y sink when `getTarget() == null`. The generic
   `LivingEntity::travel_in_water` (living.rs:2289) uses drag 0.8 and no target-based sink, so
   idle guardians drift differently.
3. **GuardianMoveControl missing** (M). Vanilla `Guardian#GuardianMoveControl`: water-only
   movement, speed scaled 0.1, retargets to a random point near `blockPosition().above(10)`
   every 80 ticks (160 when no target). Pumpkin uses the default `MoveControl`, so path shapes
   and the hover-above-player behaviour differ.
4. **Thorns reflection uses the wrong entity kind** (S, needs verification).
   guardian.rs:113 and elder_guardian.rs:96 reflect 2.0 THORNS onto `source`. Vanilla requires
   `source.getDirectEntity() instanceof LivingEntity`, i.e. an arrow/direct projectile hit does
   not reflect. Confirm whether `on_damage_with_context(source, cause)` maps `source` to the
   causing entity; if so use `cause` (direct entity) for the reflection.
5. **Elder guardian effect application is broader than vanilla** (S). elder_guardian.rs:70-87
   applies Mining Fatigue to every player within 50 and sends the game event to all. Vanilla
   `MobEffectUtil.addEffectToPlayersAround(..., 1200)` only affects (and only notifies) players
   that do not already hold DIG_SLOWDOWN with sufficient remaining duration. Net gameplay is
   close because the effect is refreshed every 1200 ticks; fix only if the packet spam matters.
6. **FIXED (UNVERIFIED)**: `Guardian` now overrides `mob_is_pushed_by_fluids` to false (guardian.rs). **Fluid push** (S, unverified). Check whether vanilla `Guardian` overrides
   `isPushedByFluid()` (only `WaterAnimal` certainly does); if it does, add
   `Mob::mob_is_pushed_by_fluids() -> false` in guardian.rs. Do not change without a vanilla
   source to confirm.

## Gap status (h6 sweep)
- Gap 4 (thorns reflection from direct entity): FIXED 99fa5cfb8. Guardian and elder guardian now pass `source` (direct entity) to `reflect_thorns`.
- Gap 1 (beam attack goal): FIXED (see Result sections below). `GuardianAttackGoal` implemented.
- Gap 2 (water travel with target sink): FIXED (see Result sections below). `travel_in_water` overrides added.
- Gap 3 (GuardianMoveControl): FIXED (see Result sections below). Custom move control installed.
- Gap 5 (elder effect broadness): FIXED (see Result sections below). Effect refresh predicate matched.
- Gap 6 (fluid push): VERIFIED (see Result sections below). No override needed; vanilla has none.

## Result (2026-11-17, verified against vendored decompiled vanilla)
- Gap 6 (fluid push): FIXED by removing the incorrect override. Vanilla `Guardian` does not
  override `isPushedByFluid()`; `Entity` returns `true` and the `Mob` trait default in Pumpkin is
  also `true`. Removed `mob_is_pushed_by_fluids -> false` from guardian.rs and elder_guardian.rs.
- Gap 2 (water travel): FIXED. Both entities now override `travel_in_water` with vanilla
  `Guardian#travelInWater`: `moveRelative(0.1)`, move SELF, uniform 0.9 drag, and a -0.005 y sink
  only while not moving and with no target. Constants are shared from guardian.rs.
- Gap 5 (elder effect broadness): FIXED. The effect loop now skips creative/spectator players and
  reapplies only when the held mining fatigue is weaker or ends within 1200 ticks, matching vanilla
  `MobEffectUtil.addEffectToPlayersAround`. The game-event amplitude now comes from `is_silent()`
  instead of `rand::random`. Added a pure unit test for the reapply predicate.
- Gap 3 (GuardianMoveControl): PARTIAL. The wave push, speed lerp and moving flag were already
  present for guardians; added a `new` constructor, installed the same move control on elder
  guardians, and wired its moving flag into elder thorns reflection and the travel sink. The
  remaining look-at update is still OPEN: it needs `LookControl::isLookingAtTarget` plus a wanted
  position getter in `entity/ai/control/look_control.rs`, outside this unit.
- Gap 1 (beam attack): STILL OPEN. Needs a new goal file, a synced attack-target metadata index,
  INDIRECT_MAGIC damage wiring and cancel-on-hurt clearing; all outside this unit.

## Result (2026-11-18, re-verified against vendored decompiled vanilla Guardian.java)
- Gap 1 (beam attack): VERIFIED FIXED in this checkout. guardian.rs already defines
  GuardianAttackGoal (priority 3 guardian / 3 elder) with navigation stop, look at target
  (90, 90), line-of-sight cancel, charge to 80/60 ticks, set DATA_ID_ATTACK_TARGET at tick 0,
  broadcast GuardianAttackSound, INDIRECT_MAGIC damage with guardian as both source and cause,
  do_hurt_target, and metadata cleared in stop(). beam_magic_damage matches vanilla
  1.0 + 2 hard + 2 elder. No code change needed.
- Gap 3 (GuardianMoveControl look-at): VERIFIED FIXED in this checkout. look_control.rs now
  implements LookControlTrait::wanted_position and is_looking_at_target; GuardianMoveControl::tick
  applies the vanilla 0.125 lerp (snap when not looking) and look_at_with_range(..., 10.0, 40.0).
  GuardianMoveControl::new is installed on both guardian and elder guardian. No code change needed.
- Gaps 2, 4, 5, 6 were already recorded FIXED in the 2026-11-17 Result section above.
- No STILL OPEN rows remain. No generated-data, WIT or cross-crate changes were required.

## Result (2026-11-18, independent re-verification against vendored decompiled vanilla)
Re-read guardian.rs, elder_guardian.rs and look_control.rs and compared each row with
`net/minecraft/world/entity/monster/Guardian.java`, `ElderGuardian.java` and
`MobEffectUtil.java`. No STILL OPEN rows remain and no code change was needed.

- Gap 1 (beam attack): VERIFIED. `GuardianAttackGoal` matches vanilla: starts at -10, stops
  navigation, looks at target (90/90), cancels on lost sight, sets DATA_ID_ATTACK_TARGET at
  tick 0, broadcasts GuardianAttackSound when not silent, charges to 80 (guardian) / 60 (elder),
  deals INDIRECT_MAGIC with the guardian as both source and cause, calls do_hurt_target and clears
  the target. `beam_magic_damage` is 1.0 +2 Hard +2 elder, matching vanilla lines 410-417.
- Gap 2 (water travel): VERIFIED. Both `travel_in_water` overrides use speed 0.1, drag 0.9, and
  a -0.005 y sink only while idle with no target (vanilla lines 338-345).
- Gap 3 (GuardianMoveControl): VERIFIED. `tick()` matches vanilla lines 450-488, including the
  `yd / length` look-target quirk on line 470, the 0.125 speed lerp, wave push and 90-degree turn
  (`MoveControl::MAX_TURN` is 90.0). `look_control.rs` supplies `wanted_position` and
  `is_looking_at_target`, and both entities install the move control.
- Gap 4 (thorns source): VERIFIED. On arrow hit Pumpkin passes the arrow as `source` and the
  shooter as `cause` (arrow.rs:774-776), so reflect_thorns gets the direct entity as vanilla
  requires (`source.getDirectEntity() instanceof LivingEntity`). Generated
  `AVOIDS_GUARDIAN_THORNS` contains magic and thorns, so the explicit vanilla thorns check is
  covered by the tag.
- Gap 5 (elder effect): VERIFIED. Vanilla `ElderGuardian` lines 67-78 and `MobEffectUtil`
  lines 89-110 use `(tick+id)%1200`, amplifier 2, duration 6000, survival-only players, and
  `endsWithin(displayLimit - 1)` which equals Pumpkin's `remaining < 1200`. The appearance-event
  amplitude uses `is_silent()`.
- Gap 6 (fluid push): VERIFIED. Neither entity overrides `mob_is_pushed_by_fluids`; vanilla
  `Guardian` has no `isPushedByFluid` override.

Static verification only. Worker shell is denied, so no compile or test run was performed;
the existing pure unit tests in guardian.rs and elder_guardian.rs remain the gate for the
orchestrator.

## Result (2026-11-18, THIRD PASS, independent verification against vendored decompiled vanilla)
Re-read guardian.rs, elder_guardian.rs, look_control.rs and arrow.rs and compared every gap
directly with the vendored decompiled sources now available under `.vendor/vanilla-decompiled/`.
No STILL OPEN rows remain and no source-file edit was needed.

- Gap 1 (beam attack): VERIFIED. `net/minecraft/world/entity/monster/Guardian.java`
  GuardianAttackGoal (lines 347-429) and `ElderGuardian.java:40-42`. Pumpkin's GuardianAttackGoal
  matches: attack_time starts -10, navigation stop + look at target 90/90, LOS-cancel, sets
  DATA_ID_ATTACK_TARGET at tick 0 and broadcasts GuardianAttackSound when not silent, charges to
  80/60, then indirectMagic with the guardian as both source and cause plus do_hurt_target and
  target clear. beam_magic_damage = 1.0 +2 Hard +2 elder (Guardian.java:410-417). Target selector
  (Guardian.java:431-442; Player/Squid/Axolotl >9 sq) matches GUARDIAN_PREY/outside_melee_range.
  Vanilla registers at priority 4, Pumpkin at 3; goal_selector uses lower-is-first
  (goal_selector.rs:305), so relative ordering (beam, swim, restriction, look, lookaround) is identical.
- Gap 2 (water travel): VERIFIED. `Guardian.java:338-345` = moveRelative(0.1), move SELF, scale 0.9,
  -0.005 y sink only when !isMoving && target==null. Both Pumpkin overrides (guardian.rs:449-459,
  elder_guardian.rs:145-155) match exactly.
- Gap 3 (GuardianMoveControl): VERIFIED. `Guardian.java:444-490`: yaw rotlerp 90 (line 458),
  speed lerp 0.125 (461), wave push (463-467), look target getEyeY()+yd/length quirk (470),
  lerp 0.125 + setLookAt(...,10,40) (481-484), setMoving(true)/else reset. guardian.rs:268-329
  matches; look_control.rs supplies wanted_position and is_looking_at_target (== lookAtCooldown>0,
  `LookControl.java:73-75`) and is installed on both entities; MoveControl::MAX_TURN=90.0
  (move_control.rs:177).
- Gap 4 (thorns source): VERIFIED. `Guardian.java:317-330` reflects 2.0 thorns only when !isMoving,
  damage outside AVOIDS_GUARDIAN_THORNS/THORNS and source.getDirectEntity() is a LivingEntity. An
  arrow's direct entity is the arrow (not living), so no reflection; Pumpkin's projectile/arrow.rs:777-785
  passes the arrow as `source` and shooter as `cause`, and guardian on_damage_with_context forwards
  `source` to reflect_thorns which checks get_living_entity. Generated tag.rs:20050-20061 confirms
  magic and thorns are in the avoids tag. Matches.
- Gap 5 (elder effect): VERIFIED. `ElderGuardian.java:67-78` and `MobEffectUtil.java:89-110`:
  (tick+id)%1200, radius 50, duration 6000, amplifier 2, displayEffectLimit 1200, survival-only
  players, reapply when amplifier<2 or endsWithin(1199) (== remaining<1200), event amplitude
  isSilent()?0:1. elder_guardian.rs:161-207 and should_apply_fatigue match, as does home restriction
  (ElderGuardian.java:76-78).
- Gap 6 (fluid push): VERIFIED. `Guardian.java` has no isPushedByFluid override (inherits Entity's
  true); neither Pumpkin entity overrides mob_is_pushed_by_fluids.

Static verification only; worker shell/compiler denied. Existing pure unit tests in guardian.rs and
elder_guardian.rs remain the orchestrator's gate. No generated-data, WIT or cross-crate change needed.


## Result (2026-11-18, FOURTH PASS, independent line-by-line verification against vendored vanilla)
Re-read guardian.rs, elder_guardian.rs, look_control.rs, move_control.rs, goal_selector.rs and
projectile/arrow.rs and compared every gap directly with
`.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Guardian.java`,
`ElderGuardian.java`, `.../ai/control/LookControl.java`, `.../ai/control/MoveControl.java`,
`.../ai/goal/GoalSelector.java` and `.../world/effect/MobEffectUtil.java`.
All six gaps are FIXED/VERIFIED; no source-file edit was needed this pass.

- Gap 1 (beam attack): VERIFIED. `GuardianAttackGoal` matches Guardian.java:347-429: attack_time
  starts -10, navigation stop + look at target 90/90, LOS cancel, DATA_ID_ATTACK_TARGET set at
  tick 0 with broadcast GuardianAttackSound when not silent, charge 80/60, INDIRECT_MAGIC with
  guardian as source and cause, do_hurt_target, target cleared. `beam_magic_damage` = 1.0 +2 Hard
  +2 elder (Guardian.java:410-417). Registered at Pumpkin priority 3; goal_selector.rs:305 uses
  lower-priority-wins, so relative ordering (beam < swim < restriction < look < lookaround) matches
  vanilla's (4/5/7/8/8/9). stop() clears the synced attack target when should_continue fails.
- Gap 2 (water travel): VERIFIED. Guardian.java:338-345 = moveRelative(0.1), move SELF, scale 0.9,
  -0.005 y sink only when !isMoving && target==null. Both overrides (guardian.rs:449-459,
  elder_guardian.rs:145-155) match; constants shared from guardian.rs.
- Gap 3 (GuardianMoveControl): VERIFIED. Guardian.java:449-489: MOVE_TO && !navigator.isDone,
  yaw rotlerp 90 (MAX_TURN=90.0 at move_control.rs:177), speed lerp 0.125, wave push with 0.05
  scale and (tick+id) phase, yd/length look-target quirk (line 470), look lerp 0.125 +
  setLookAt(...,10,40), setMoving(true)/else reset. look_control.rs:107-113 supplies
  wanted_position and is_looking_at_target (== look_at_timer>0, matching LookControl.java:73-75).
  Installed on both entities; synced DATA_ID_MOVING is driven by the same moving flag.
- Gap 4 (thorns source): VERIFIED. Guardian.java:317-330 reflects 2.0 thorns only when !isMoving,
  damage outside AVOIDS_GUARDIAN_THORNS/THORNS and source.getDirectEntity() is a LivingEntity.
  projectile/arrow.rs:777-785 passes the arrow as `source` and the shooter as `cause`;
  on_damage_with_context forwards `source` to reflect_thorns, whose get_living_entity() check
  rejects the arrow. The generated avoids tag covers magic and thorns. Matches.
- Gap 5 (elder effect): VERIFIED. ElderGuardian.java:67-78 and MobEffectUtil.java:89-110:
  (tick+id)%1200, radius 50, duration 6000, amplifier 2, displayEffectLimit 1200, survival-only
  players, reapply when amplifier<2 or remaining<1200 (== endsWithin(1199)), event amplitude
  isSilent()?0:1, home restriction 16. elder_guardian.rs:161-207 and should_apply_fatigue match.
- Gap 6 (fluid push): VERIFIED. Guardian.java has no isPushedByFluid override; neither Pumpkin
  entity overrides mob_is_pushed_by_fluids.

No STILL OPEN rows remain. No generated-data, WIT or cross-crate change needed. Source files were
left untouched this pass; existing pure unit tests remain the orchestrator's gate.
