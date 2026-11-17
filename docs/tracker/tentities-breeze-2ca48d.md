# Tracker: entities / Breeze (RE-REVIEW, 2ca48d)

Source under review: `crates/pumpkin-core/src/entity/mob/breeze.rs` (105 lines).
Supersedes `docs/tracker/tentities-breeze.md` (previous pass).
No decompiled reference is vendored (`.vendor/` absent); vanilla compared against
`net.minecraft.world.entity.monster.Breeze` / `BreezeAi` (1.21.x) plus the values the
extractor already pulled from the vanilla jar (`assets/entities.json`).

## What changed since the previous review
Commit `96b82d97e feat(entity): add breeze ambient sound and revenge target goal` added
`Mob::get_ambient_sound` (`breeze.rs:72-85`) and `RevengeGoal::new(true)` at target
priority 2 (`breeze.rs:49-50`). Both re-checked in this pass.

## Re-checked and accepted (no action)
- Ambient sound (`breeze.rs:72-85`): `BREEZE_IDLE_GROUND` on the ground, `BREEZE_IDLE_AIR`
  in the air, matching `Breeze#getAmbientSound`. The generated default in
  `crates/pumpkin-data/src/generated/entity_type.rs:1568` is air-only, so the override is a
  real fix, not dead code (`Mob::get_ambient_sound` is dispatched by the mob ambient tick).
- Fall damage + land sound (`breeze.rs:57-70`): `BREEZE_LAND` above 3.0 blocks then
  `super.causeFallDamage`, identical to `Breeze#causeFallDamage`.
- `getSwimHeight` (`crates/pumpkin-core/src/entity/living.rs:2567-2568`): breeze returns its
  eye height, matching `Breeze#getSwimHeight`. Already present before this row.
- Deflection plumbing (`breeze.rs:91-104` + `crates/pumpkin-core/src/entity/projectile_deflection.rs:80-91`):
  the `minecraft:deflects_projectiles` tag does contain the breeze
  (`crates/pumpkin-data/src/generated/tag.rs:20404`, id 17), so the fallback path is live and
  `ProjectileDeflection::Breeze` (deflect sound + reverse) is reachable.
- Attributes (`crates/pumpkin-data/src/generated/entity_type.rs:1533-1564`): movement speed
  0.63, health 30, follow range 24, attack damage 3, dimension 0.6x1.77, all from the
  extracted vanilla data. Nothing hand-edited here.

## Previous findings withdrawn (false positives)
- XP reward: the earlier note claimed vanilla awards 5 XP and the generated 10 was wrong.
  `assets/entities.json` -> `breeze.experience_reward` is 10, i.e. the extracted vanilla
  value; `experience_reward: 10u32` (`entity_type.rs:1565`) is correct. Not a gap.

## Remaining gaps

### 1. Breeze never attacks: no shoot/slide/jump AI (L)
`breeze.rs:36-51` registers `SwimGoal(0)`, `WanderAroundGoal(5)`, `LookAtEntityGoal(6)`,
`RandomLookAroundGoal(7)` plus `ActiveTargetGoal(1)`/`RevengeGoal(2)`. Vanilla 1.21
`Breeze` drives all motion from a brain (`BreezeAi`) with `BreezeShootTask`,
`BreezeSlideTask`, `BreezeJumpTask` and `BreezeJumpToWaterTask` over the memories
`BREEZE_SHOOT`, `BREEZE_SHOOT_CHARGING/RECOVERING/COOLDOWN`, `BREEZE_JUMP_COOLDOWN/INHALING/TARGET`,
`BREEZE_LEAVING_WATER`. Those memories are already declared and completely unused
(`crates/pumpkin-core/src/entity/ai/brain/memory/types.rs:152-159`; the only other reference
is `name_of` at line 204). Consequence: the breeze never fires a wind charge, so
`WindChargeEntity::new_breeze` (`crates/pumpkin-core/src/entity/projectile/wind_charge.rs:76`)
is only reachable by spawning `BREEZE_WIND_CHARGE` directly
(`crates/pumpkin-core/src/entity/type.rs:312-318`), and the breeze charge explosion
(power 3.0, `wind_charge.rs:103-108`) never happens in gameplay. The unused breeze sounds
(`BREEZE_CHARGE`, `BREEZE_INHALE`, `BREEZE_JUMP`, `BREEZE_SHOOT`, `BREEZE_SLIDE`,
`BREEZE_WHIRL`) confirm the missing tasks.
Fix: port `BreezeAi` (core/idle/fight activity lists, the four tasks, shoot/jump controllers)
onto the existing brain framework, then delete the fabricated goal set.
Size: L (multi-file, needs the brain activity/controller APIs).

### 2. Fabricated goal set (M, folded into 1)
Same lines. `WanderAroundGoal`, `LookAtEntityGoal` and `RandomLookAroundGoal` are not part of
the vanilla breeze; the vanilla target handling is brain sensors (`NearestLivingEntities` +
`HurtBy`) feeding `ATTACK_TARGET`, not `ActiveTargetGoal`/`RevengeGoal` in a `GoalSelector`.
Removing them alone leaves the mob inert, so land with gap 1.
Side note: the comment at `breeze.rs:49` says `HurtByTargetGoal` has "no alerting";
vanilla `HurtByTargetGoal` alerts same-class mobs by default, and the Rust call leaves
`alert_others = false` (`crates/pumpkin-core/src/entity/ai/goal/revenge.rs:45`). Do not
"fix" this under the goal-based approximation: the whole selector is replaced by gap 1.
Size: M, folded.

### 3. `ENTITY_ACTION` game event missing on breeze landing (S)
`breeze.rs:64-66` plays `BREEZE_LAND` and delegates; vanilla `Breeze#causeFallDamage` also
emits `GameEvent.ENTITY_ACTION` at that point (used by sculk sensors). Pumpkin has no entity
game-event API yet.
Fix: after entity game events land, emit `ENTITY_ACTION` from the caller before delegating.
Size: S, blocked on the game-event subsystem.

### 4. Deflection exemption list to confirm (S)
`breeze.rs:91-104` returns `ProjectileDeflection::None` for both `WIND_CHARGE` and
`BREEZE_WIND_CHARGE`. Vanilla only needs to exempt the breeze's own charge (it is the
shooter, so it does not target itself); a player wind charge is expected to be deflected like
any other projectile. Without `.vendor/` this cannot be confirmed, and `WindChargeEntity`
already refuses to hit its owner (`wind_charge.rs:154-160` filters other wind charges and end
crystals independently), so narrowing the list to `BREEZE_WIND_CHARGE` is the likely fix.
Verify against decompiled `Breeze#deflection` before changing.
Size: S (verify first).

## Audit verdict
No small, certain code change was justified in this pass: the two commits under review
(`96b82d97e`) are correct, the old XP finding is withdrawn, and every concrete remaining gap
is either the L brain port, folded into it, or blocked on a missing subsystem.
