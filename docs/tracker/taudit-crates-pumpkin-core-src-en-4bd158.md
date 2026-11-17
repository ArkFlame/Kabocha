# Tracker: audit / entity/ai/target_match.rs

Source under review: `crates/pumpkin-core/src/entity/ai/target_match.rs`.
No decompiled reference is vendored (`.vendor/` absent); vanilla
`net.minecraft.world.entity.Mob#getNearestPlayer` / `getNearestEntity`,
`NearestAttackableTargetGoal`, `LookAtEntityGoal` and the `Enemy` / `Monster` class
hierarchy (1.21.x) were compared from knowledge, cross-checked against the generated
entity data in `crates/pumpkin-data/src/generated/entity_type.rs` and the callers in
`crates/pumpkin-core/src/entity/**`.

Markers in the file: no `TODO`/`FIXME`. The four matches are explanatory comments
(`:30`, `:113`, `:164`, `:181`); none is a gap.

## Fixed in this pass
None. The file is parity-clean: class families (`UNDEAD`, `ILLAGER`, `FISH`,
`ZOMBIE_LIKE`, `SKELETON_LIKE`, `ABSTRACT_HORSE`, ...) match the vanilla class
members, `ENEMY_NOT_MONSTER` matches the seven `Enemy` implementors outside
`Monster`, and the `MONSTER` spawn category check plus `NOT_ENEMY` agrees with the
generated `is_enemy` flag for every entry (verified `camel_husk`, `zombie_horse`,
`zombie_nautilus`, `sulfur_cube` = false; `hoglin`, `ender_dragon`, `ghast`,
`phantom`, `shulker`, `slime`, `magma_cube` = true), so no certain behavioural fix
was committed here.

## Remaining gaps

### 1. `Enemy` is derived from a hand-maintained table (S)
`target_match.rs:31-51` computes `Enemy` as `category == MONSTER && !NOT_ENEMY`, but the
generator already emits vanilla's interface flag: `EntityType::is_enemy`
(`crates/pumpkin-data/src/generated/entity_type.rs:38`, already trusted by
`block/entities/conduit.rs:146`). The heuristic is exactly equivalent today, but a new
`MobCategory::MONSTER` mob that does not implement `Enemy`, or a non-monster category
`Enemy`, silently drifts (`NOT_ENEMY` has to be kept in sync by hand). Suggested fix:
`Self::Enemy => entity_type.is_enemy` and delete `NOT_ENEMY`; keep
`ENEMY_NOT_MONSTER` because the generated data carries no `Monster` class marker.
Size: S.

### 2. `players_anywhere` gives broad target filters unbounded player reach (S-M)
`target_match.rs:89-112` (flag defined at `:139`, set by `:164-169`) skips the search box
for players whenever the caller sets the flag. Vanilla only does this in the
`getNearestPlayer` branch, i.e. when the goal's class filter is exactly `Player.class`;
a broad filter (`LivingEntity.class`, `Mob.class`) uses
`getEntitiesOfClass(type, box.inflate(range, 3.0, range))`, so a player more than three
blocks above or below is not a candidate. `goal/active_target.rs:231` and
`goal/look_at_entity.rs:200` set the flag unconditionally, so e.g. the wither
(`entity/boss/wither.rs:51`, `AllExcept(wither_friends)`) and broad `Any` hunts can pick
a vertically distant player that vanilla would ignore. Suggested fix: take the unbounded
player branch only when `self.is_player()` (or have callers set the flag only for
player-only matches) and use `get_players_at_box` otherwise. Size: S-M (three call
sites, no runtime repro available).

### 3. Player-vs-entity tie-break assumes spatial-index order (M)
`target_match.rs:119-128` prefers the player on an exact distance tie (`<=>`). Vanilla
`Mob#getNearestEntity` keeps the first strict minimum while iterating the level's entity
list, which includes players in an order the spatial index does not reproduce. The
comment at `:113` claims index order equals vanilla list order, which is unproven.
Needs a deterministic ordering harness to observe; not fixed blind. Size: M.

### 4. `from_eyes` ignores the pose eye height (S)
`target_match.rs:147` adds `entity_dimension.eye_height` (standing value) to the feet
position. Vanilla `Entity#getEyeY()` is `getY() + getEyeHeight(getPose())`, so crouching
or swimming searchers rank candidates from the wrong origin. Only affects the ranking
origin, not vanilla box membership. Suggested fix: use the pose-adjusted eye height used
elsewhere in `living.rs`. Size: S.
