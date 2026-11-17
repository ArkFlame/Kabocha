# Tracker: entities / Breeze (RE-REVIEW)

Source under review: `crates/pumpkin-core/src/entity/mob/breeze.rs`.
No decompiled reference is vendored (`.vendor/` absent); vanilla class compared from
`net.minecraft.world.entity.monster.Breeze` (1.21.x) and the generated data in
`crates/pumpkin-data/src/generated/entity_type.rs:1533`.

## Fixed in this pass
- Ambient sound: vanilla `Breeze.getAmbientSound()` returns `BREEZE_IDLE_GROUND` on the
  ground and `BREEZE_IDLE_AIR` in the air. Rust returned `None` for every mob by default,
  so the breeze was silent. Added `Mob::get_ambient_sound` (`breeze.rs:73`).
- Target goal: vanilla `Breeze.registerGoals` ends with `new HurtByTargetGoal(this)` (no
  alerting). Added `RevengeGoal::new(true)` at target priority 2 (`breeze.rs:50`).

## Remaining gaps

### 1. No wind charge attack (L)
`breeze.rs:36-48` — vanilla `Breeze.registerGoals` registers
`BreezeSlideGoal` (4), `BreezeShootGoal` (5), `BreezeJumpGoal` (6). Rust registers only
`SwimGoal(0)`, `WanderAroundGoal(5)`, `LookAtEntityGoal(6)`, `RandomLookAroundGoal(7)`,
none of which exist on the vanilla breeze. The breeze therefore never attacks.
The projectile side already exists and is unused: `WindChargeEntity::new_breeze`
(`crates/pumpkin-core/src/entity/projectile/wind_charge.rs:76`), `WindChargeKind::Breeze`
held bonuses at `wind_charge.rs:103-106`.
Suggested fix: port the three goals under `crates/pumpkin-core/src/entity/ai/goal/`, fire
`BREEZE_WIND_CHARGE` with the breeze as owner, and drive them from the vanilla brain
memories already declared in
`crates/pumpkin-core/src/entity/ai/brain/memory/types.rs:152-159`
(`BREEZE_SHOOT`, `BREEZE_SHOOT_CHARGING/RECOVERING/COOLDOWN`, `BREEZE_JUMP_*`).
Size: L.

### 2. Fabricated goal set (M)
Same lines as above. Vanilla breeze has no `WanderAroundGoal`, `LookAtEntityGoal` or
`RandomLookAroundGoal`; it only slides, jumps and shoots, plus `FloatGoal` behaviour
(`swim` in Pumpkin). Removing them without gap 1 leaves the mob inert, so land this
together with gap 1. Size: M (folded into gap 1).

### 3. Experience reward disagrees with vanilla (S)
`crates/pumpkin-data/src/generated/entity_type.rs:1565` has `experience_reward: 10u32`
for `BREEZE`, while vanilla `Breeze` awards 5 XP. Generated data must not be hand-edited;
this needs the extractor/codegen source value checked first.
Suggested fix: confirm against the decompiled `Breeze#getExperienceReward` and, if 5 is
correct, fix the asset/JSON fed to `tools/pumpkin-codegen` and regenerate.
Size: S (data-side, blocked on extractor value).

### 4. Deflection exception list to verify (S)
`breeze.rs:92-104` returns `ProjectileDeflection::None` for both `WIND_CHARGE` and
`BREEZE_WIND_CHARGE`, then falls back to `default_deflection` (tag
`minecraft:deflects_projectiles`). Vanilla only exempts the breeze's own wind charge
type; a player wind charge should still knock the breeze back rather than be exempted
only if that matches the decompiled override. Confirm `Breeze#deflection` before
narrowing the list to `BREEZE_WIND_CHARGE`. Size: S.

### 5. Fall damage / land sound parity detail (S)
`breeze.rs:54-66` plays `EntityBreezeLand` for `fall_distance > 3.0` then delegates to
`LivingEntity::cause_fall_damage`. Vanilla also emits
`GameEvent.ENTITY_ACTION` at that point (`Breeze#causeFallDamage`); Pumpkin has no game
event for entity actions yet. Size: S, game-event dependent.
