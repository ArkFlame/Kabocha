# blocks/Creaking Heart - re-review

Source: `crates/pumpkin-core/src/block/blocks/creaking_heart.rs`,
`crates/pumpkin-core/src/block/entities/creaking_heart.rs`,
`crates/pumpkin-core/src/entity/mob/creaking.rs`.

No decompiled vanilla sources in this worktree (`.vendor/` absent, no vanilla repo next to it), so rows are checked
against the vanilla `CreakingHeartBlock`/`CreakingHeartBlockEntity` classes as known from 1.21.4/1.21.5 plus
`docs/VANILLA_REF.md` (line 7). Re-check numeric rows with the Extractor before changing them.

## Verified already correct (previous review was stale)

- Heart state tick: `entities/creaking_heart.rs:62-69` calls `update_state` every 20 ticks, so the day/night
  `gameplay/creaking_active` flip drives `AWAKE`/`DORMANT` without a neighbour change.
- Comparator: `blocks/creaking_heart.rs:116-140` (`15 - floor(15d/32)`, 0 when unbound/unloaded), matching
  `docs/VANILLA_REF.md:7`. The exact vanilla rounding (cast vs floor) is unverified without decompiled source.
- Placement `natural`: generated default state (`pumpkin-data/.../block.rs:553594`, state offset 7) is
  `natural = false`; the pale oak feature sets `natural = true` (`pumpkin-world/.../decorator/creaking_heart.rs:50`),
  matching vanilla `getPlacementState` + feature override.
- Drops: generated loot table `BLOCKS_CREAKING_HEART` (`pumpkin-data/.../loot_table.rs:21998`) already encodes
  `creaking_heart` with Silk Touch and 1-3 `resin_clump` without it.
- Break sound (`blocks/creaking_heart.rs:143`) and neighbour/place state recomputation (`:106-114`).

## Fixed in this pass

- `blocks/creaking_heart.rs:148` - `broken` played the break sound but never granted vanilla's 20-24 experience for
  world-gen hearts (`CreakingHeartBlock#onStacksDropped` -> `dropExperienceWhenMined`). Added the `natural` guard,
  the Silk Touch / creative early-outs and `ExperienceOrbEntity::spawn(20 + rand(0..5))`. Not compiled here;
  batch check required.

## Remaining gaps

| file:line | vanilla | suggested fix | size |
|---|---|---|---|
| `entities/creaking_heart.rs:93` (`set_creaking_uuid`, no caller) | `CreakingHeartBlockEntity#tick` spawns its creaking when `AWAKE` with a player nearby, stores its UUID in `creaking`, and despawns/re-binds when the entity is gone or farther than 34 blocks | add the spawn/despawn loop in `tick`: spawn `minecraft:creaking` inside the two logs, `set_creaking_uuid(Some(uuid))`, clear on death or >34 blocks | L |
| `entities/creaking_heart.rs:87-91` (`is_protector`) | vanilla `CreakingHeartBlockEntity#isProtector` requires the stored UUID to equal the creaking's UUID (an unbound heart protects nothing) | once binding exists, return `self.creaking_uuid.load() == Some(creaking_uuid)`; the current `is_none_or` fallback is only a stand-in for the missing spawn path | S |
| `entity/mob/creaking.rs:486-503` (heart check) | vanilla creaking despawns/tears down when farther than 34 blocks from its heart | add the 34-block distance check to the existing home-heart block and start the tear-down | M |
| `entity/mob/creaking.rs:119,120-124` | creaking-specific wander/melee goals (`enable_creaking_melee_attack`) | see `docs/tracker/tentities-creaking.md`, same root cause from the mob side | M |
| `entities/creaking_heart.rs:97` (`creaking_hurt`) | vanilla heart also emits its hurt particle and `setChanged` when the creaking is hit | left as-is: sound + sway already reproduce the observable effect | S |
| piston behaviour | vanilla creaking heart cannot be pushed | not actionable in this crate: `pumpkin-data`'s `Block` has no push-reaction field and no piston push code consults one, so there is nothing per-block to fix here | M |

## h6 gap status

- Row 2 (isProtector UUID match): STILL OPEN (S, blocked on the missing creaking spawn/bind path in tick, L).
- Row 1, 3, 4, 6: STILL OPEN (L/M, need spawn loop and creaking AI teardown; piston row needs push-reaction data).
- Row 5 hurt particle: STILL OPEN (S, left as-is by design).


## Result

Verified against decompiled vanilla (`CreakingHeartBlock.java`, `CreakingHeartBlockEntity.java`,
`Creaking.java` in `.vendor/vanilla-decompiled/net/minecraft/`), now present in this worktree.

| row | outcome | evidence |
|---|---|---|
| 1 spawn/despawn loop in heart tick | FIXED | `CreakingHeartBlockEntity.java:101-136`: added `try_despawn_creaking` in `tick` (day break, >34 blocks, player-stuck, persistence, missing-entity clear) and added the hostile/mob-spawning gates to `try_spawn_creaking`. |
| 2 `is_protector` UUID match | VERIFIED (already correct) | current `is_some_and(|uuid| uuid == creaking_uuid)` matches `CreakingHeartBlockEntity.java:333-335` (`map(c -> c == creaking).orElse(false)`); existing `unbound_heart_protects_no_creaking` test covers it. |
| 3 mob 34-block distance | RESOLVED via row 1 | `Creaking.java:241-258` mob side only checks `isProtector` then sets health 0 (already present in `creaking.rs:504-521`); the 34-block rule lives heart-side (`CreakingHeartBlockEntity.java:130`) and is now implemented. No mob edit needed. |
| 4 creaking wander/melee goals | STILL OPEN | needs new creaking-specific goal types under `entity/ai/goal/` (files outside this unit); see `tentities-creaking.md`. |
| 5 hurt particle / setChanged | STILL OPEN | vanilla emits client-only particles and resin clumps; sound + sway already reproduce the observable effect, and client-only effects do not block. |
| 6 piston push-reaction | STILL OPEN | needs a push-reaction field on generated `pumpkin-data` `Block` plus piston push code (generated data / cross-crate), nothing per-block to fix here. |

Also: comparator rounding is now verified as vanilla `Redstone.SIGNAL_MAX - (int)floor(clamp(distance,0,32)/32*15)`
(`CreakingHeartBlockEntity.java:341-348`), matching `blocks/creaking_heart.rs:141-142`. Not compiled here (orchestrator batch check).
Added pure unit test `creaking_too_far_boundary` for the vanilla 34-block threshold. Formatted with rustfmt.


## Result (re-review pass, vanilla decompiled present)

Re-checked the three STILL OPEN rows (4, 5, 6) against `.vendor/vanilla-decompiled`.

| row | outcome | evidence |
|---|---|---|
| 4 creaking wander/melee goals | STILL OPEN (files outside unit) | Vanilla `CreakingAi.java:30-85` drives CORE/IDLE/FIGHT through a Brain/Behaviour system Pumpkin does not have. `creaking.rs:162-189` already registers `SwimGoal`, `MeleeAttackGoal(1.0,false,interval 40)` gated by `canMove`, `WanderAroundGoal(0.3)` gated by `canMove`, `LookAtEntityGoal(8.0)`, `RandomLookAroundGoal`, and `ActiveTargetGoal`, reproducing the observable gating/speed/interval. Full Brain/Activity parity needs new goal types under `entity/ai/goal/` (outside this unit's files) and is tracked on the mob side in `tentities-creaking.md`. |
| 5 hurt particle / emitter trail | STILL OPEN (client-only) | Vanilla `CreakingHeartBlockEntity.java:228-250` `creakingHurt` emits `TrailParticleOption` particles and seeds `emitter` so `CREAKING_HEART_HURT` replays 10x over 100 ticks (`:82-99`); all of that is client-side sound/particles. `creaking.rs:619-627` calls `creaking_hurt` only for a bound protector, and `creaking_heart.rs:221-245` plays the hurt sound and grows resin while awake. No server-observable divergence; client-only effects do not block. |
| 6 piston push | RESOLVED (already correct, no change) | Vanilla blocks pushing via `PistonBaseBlock.isPushable` returning `!state.hasBlockEntity()` (`PistonBaseBlock.java:284`), not a push-reaction field; `CREAKING_HEART` has no explicit `pushReaction` (`Blocks.java:1392-1396`). Pumpkin's `piston.rs:92-99` uses the equivalent `!has_block_block_entity(block)`, and `CREAKING_HEART` carries a block-entity type (`entities/mod.rs:558-565`, `generated/block.rs:6479`), so it is already unpushable. No generated-data or cross-crate change needed. |

No source files changed: row 6 was already correct, rows 4 and 5 are out of unit / client-only. No pure unit-test seam exists for these rows (goal behavior needs live world state; piston push needs world + block-entity data). No rustfmt run (no `.rs` touched). Not compiled in this role; orchestrator batch check.

## Result (verification pass)

Re-checked every STILL OPEN row against `.vendor/vanilla-decompiled`. No source files needed changing.

| row | outcome | evidence |
|---|---|---|
| 1 spawn/despawn loop | VERIFIED (already fixed) | `entities/creaking_heart.rs:155-244` has `try_spawn_creaking` + `try_despawn_creaking` matching `CreakingHeartBlockEntity.java:73-136`. |
| 2 `is_protector` UUID match | VERIFIED (already fixed) | `entities/creaking_heart.rs:129-133` `is_some_and(|uuid| uuid == creaking_uuid)` matches `CreakingHeartBlockEntity.java:333-335` (`map(c -> c == creaking).orElse(false)`). |
| 3 mob 34-block distance | VERIFIED (already fixed) | Heart-side `is_creaking_too_far` (`entities/creaking_heart.rs:246-251`, 34.0) tears the mob down; mob-side `tick` (`creaking.rs:586-599`) only checks `isProtector` then sets health 0, matching `Creaking.java:241-258`. No mob edit needed. |
| 4 creaking wander/melee goals | STILL OPEN (out of unit) | Vanilla `CreakingAi.java:30-85` uses a Brain/Activity/Behaviour system plus a 32-block-to-home path constraint (`Creaking.java:601-611`, `MAX_DISTANCE_TO_HOME_SQ=1024`). `creaking.rs:162-189` already registers Swim, canMove-gated Melee (interval 40) and Wander (0.3), LookAt(8), RandomLook and ActiveTarget, reproducing the observable gating/speed/interval. Full parity needs new goal/navigation types under `entity/ai/` (files outside this unit). |
| 5 hurt particle / `setChanged` | RESOLVED (no-op, vanilla claim stale) | Vanilla `creakingHurt` (`CreakingHeartBlockEntity.java:228-250`) does NOT call `setChanged`; it only emits client particles and seeds `emitter`/`emitterTarget` for a client-side sound replay. The server-observable parts (2-3 resin clumps + `RESIN_PLACE` sound + `BLOCK_PLACE` game event while awake) are already in `entities/creaking_heart.rs:253-277`. No server divergence, nothing to change. |
| 6 piston push | VERIFIED (already correct) | Vanilla `PistonBaseBlock.java:284` `isPushable` returns `!state.hasBlockEntity()`; Pumpkin `piston.rs:92-99` uses the equivalent `!has_block_block_entity(block)`, and `CREAKING_HEART` carries a block entity, so it is already unpushable. |

Also re-verified comparator rounding: vanilla `getAnalogOutputSignal` (`CreakingHeartBlockEntity.java:337-348`) is `SIGNAL_MAX - (int)floor(clamp(distance,0,32)/32*15)`, matching `blocks/creaking_heart.rs:141-148`. No test seam exists for rows 4/5 (live-world goal behaviour / client-only effects). No `.rs` touched, so no rustfmt. Not compiled in this role; orchestrator batch check.


## Result (isActive target gate pass)

Re-checked the sole STILL OPEN row (4) against decompiled vanilla (`CreakingAi.java:39-55,79-85`,
`Creaking.java:499-519`). Found and fixed an in-unit parity gap in `creaking.rs`.

| row | outcome | evidence |
|---|---|---|
| 4 creaking wander/melee goals | PARTLY FIXED (in unit) | Vanilla gates IDLE `StartAttacking` on `mob.isActive()` (`CreakingAi.java:44-46`) and gives a dormant creaking no attack behaviour (`updateActivity`, `CreakingAi.java:79-85`); `activate()` assigns the attack target directly (`Creaking.java:499-504`). Pumpkin's `ActiveTargetGoal` had no isActive gate, so a dormant creaking auto-acquired a nearby player and melee-attacked before ever being looked at. Added the `creaking_is_active` gate to the target goal in `creaking.rs` plus pure-in-file test `dormant_creaking_disables_target_gate`. Melee/wander gating on canMove, 40-tick interval, 0.3 idle speed and the 32-block home path rule were already correct. Full Brain/Activity behaviour-tree parity still needs new goal types under `entity/ai/goal/` (outside this unit), but the observable activation/targeting gating is now reproduced. |
| 5 hurt particle / `setChanged` | STILL OPEN (client-only, out of unit) | Vanilla `creakingHurt` only emits client particles and seeds an emitter; no server divergence. |
| 6 piston push | VERIFIED (already correct, no change) | `PistonBaseBlock.isPushable` = `!state.hasBlockEntity()`; Pumpkin `piston.rs:92-99` uses the equivalent `!has_block_block_entity` and the heart carries a block entity. |

Rows 1-3 re-verified as already fixed (heart spawn/despawn, `is_protector` UUID match, mob teardown). No generated data, WIT or cross-crate change. Not compiled here (orchestrator batch check). Edits written via the context-mode sandbox Python runtime because no dedicated edit/write tool is exposed to this worker.


## Result (second pass, cross-file edits enabled)

Re-checked the three STILL OPEN rows against `.vendor/vanilla-decompiled`. One small in-unit parity
gap was found and fixed; the rest are a whole missing subsystem or deliberate no-ops.

| row | outcome | evidence |
|---|---|---|
| 4 creaking wander/melee goals | PARTLY FIXED (game-event parity); remainder BLOCKED | `Creaking.java:499-511` emits `GameEvent.ENTITY_ACTION` in `activate`/`deactivate`, `:224` on every `canMove` flip and `:166` on the invulnerability shake. Pumpkin emitted none. Added `GameEvent::EntityAction` via `world.emit_game_event(...)` in `creaking.rs` `activate`, `deactivate`, `mob_tick` canMove flip and `pre_damage_with_context` shake. This is server-observable (sculk sensors react). Full Brain/Activity/CORE/IDLE/FIGHT behaviour-tree parity (`CreakingAi.java:30-85`) still needs the whole Brain/Activity subsystem wired to this mob; Pumpkin's `entity/ai/brain` tree exists but is not wired to `CreakingEntity` and would require a cross-crate AI migration, so it is BLOCKED as a whole missing subsystem. The observable gating already present (canMove gating, 40-tick melee interval, 0.3 idle speed, isActive target gate, 32-block home path rule) was re-verified. |
| 5 hurt particle / `setChanged` | NO-OP (verified) | `CreakingHeartBlockEntity.java:228-250` `creakingHurt` only emits client particles and seeds an emitter; it does not call `setChanged`. Server-observable parts (2-3 resin clumps, `RESIN_PLACE` sound, `BLOCK_PLACE` game event while awake) are already in `entities/creaking_heart.rs:253-277`. No server divergence. |
| 6 piston push | VERIFIED (no change) | `PistonBaseBlock.java:284` `isPushable` = `!state.hasBlockEntity()`; Pumpkin `piston.rs:92-99` uses the equivalent `!has_block_block_entity` and the heart carries a block entity, so it is already unpushable. |

Rows 1-3 re-verified as already fixed (heart spawn/despawn, `is_protector` UUID match, mob teardown).
No generated-data, WIT or cross-crate change. No pure-test seam exists for the game-event emission
(it needs live world state), so no unit test was added. Not compiled in this role; orchestrator batch check.


## Result (third pass - all rows closed)

Re-checked every previously STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/`.
One small in-unit parity gap was found and fixed (knockback gating); the rest were verified or are
deliberate no-ops. No generated pumpkin-data, WIT or cross-crate change was needed.

| row | terminal outcome | evidence |
|---|---|---|
| 1 heart spawn/despawn loop | VERIFIED (already fixed) | `CreakingHeartBlockEntity.java:73-136` -> `entities/creaking_heart.rs:155-244` try_spawn_creaking + try_despawn_creaking (day break, >34 blocks, player-stuck, persistence, missing-entity clear). |
| 2 is_protector UUID match | VERIFIED (already fixed) | `CreakingHeartBlockEntity.java:333-335` (map(c -> c == creaking).orElse(false)) -> `entities/creaking_heart.rs:129-133` is_some_and(|uuid| uuid == creaking_uuid); test unbound_heart_protects_no_creaking. |
| 3 mob 34-block teardown | VERIFIED (already fixed) | Heart-side is_creaking_too_far (`entities/creaking_heart.rs:246-251`, 34.0) tears the mob down; mob-side tick (`creaking.rs:588-605`) checks isProtector then sets health 0, matching `Creaking.java:241-258`. |
| 4 creaking wander/melee goals | FIXED (observable parity complete) | See detail below. Added mob_knockback this pass for `Creaking.java:454-458`. All observable `CreakingAi.java` / `Creaking.java` behaviours reproduced. |
| 5 hurt particle / setChanged | VERIFIED (no-op, client-only) | `CreakingHeartBlockEntity.java:228-250` creakingHurt only emits client particles and seeds an emitter; no setChanged, no server divergence. Server parts already in `entities/creaking_heart.rs:253-277`. |
| 6 piston push | VERIFIED (already correct) | `PistonBaseBlock.java:284` isPushable = !state.hasBlockEntity(); `piston.rs:92-99` uses equivalent !has_block_block_entity, and CREAKING_HEART carries a block entity. |

Row 4 detail: observable behaviour now matches vanilla at every cited site in
`.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/creaking/`:
- `CreakingAi.java:44-46` IDLE StartAttacking gated on isActive -> creaking_is_active gate on ActiveTargetGoal (`creaking.rs:79-83,194-199`).
- `CreakingAi.java:57-68` FIGHT MeleeAttack(canMove, 40) speed 1.0 -> MeleeAttackGoal(1.0,false).with_attack_interval(40).gated_by(creaking_can_move) (`creaking.rs:173-180`).
- `CreakingAi.java:50` idle stroll 0.3 -> WanderAroundGoal(0.3).with_start_condition(creaking_can_move) (`creaking.rs:181-187`).
- `Creaking.java:188-198` isPushable/push gated on canMove -> mob_is_pushable (`creaking.rs:493-495`).
- `Creaking.java:454-458` knockback gated on canMove -> NEW mob_knockback returns !can_move() (`creaking.rs:497-509`), reporting the push as handled so default knockback is skipped while frozen.
- `Creaking.java:499-511` activate/deactivate set attack target + ENTITY_ACTION game event -> `creaking.rs:277-298`; canMove-flip and shake game events at `creaking.rs:563-586,648-649`.
- `Creaking.java:600-612` HomeNodeEvaluator 32-block home rule (1024 sq) -> HomeRestriction + position_target_range 32 (`creaking.rs:108-109,253-269`).
- Tests: dormant_creaking_disables_target_gate, frozen_creaking_disables_goal_gate, NEW frozen_creaking_ignores_knockback (`creaking.rs:704-752`).

BLOCKED-DATA note (structural only, does not block parity): reproducing the internal AI
architecture exactly would require wiring the vanilla Brain<Creaking> behaviour tree - the
Activity/Behaviour/Sensor/MemoryModule system built in `CreakingAi.java:29-85` and provided by
`Creaking.java:59-61,113-115`. Pumpkin has a brain tree under entity/ai/brain but it is not wired
to CreakingEntity, and adding that infrastructure is a whole-subsystem change. Per AGENTS.md (same
observable behaviour is enough; the algorithm may differ) this is not required, so row 4 is FIXED.

Files changed this pass: crates/pumpkin-core/src/entity/mob/creaking.rs (knockback override + test).
No generated data / WIT / cross-crate edit. Not compiled in this role; orchestrator batch check.


## Result (final verification pass, cross-file edits enabled)

Re-checked all six rows against `.vendor/vanilla-decompiled/net/minecraft/`. Every fix from earlier
passes is present in the source files; no further code change was needed. All rows are terminal.

| row | terminal outcome | evidence (.vendor path + Pumpkin code) |
|---|---|---|
| 1 heart spawn/despawn loop | VERIFIED (fixed in an earlier pass) | `world/level/block/entity/CreakingHeartBlockEntity.java:73-136` -> `block/entities/creaking_heart.rs:93-105,155-244` (`try_spawn_creaking` + `try_despawn_creaking`: hostile/spawn-mobs gates, player<=32, 34-block tear-down, persistence, missing-entity clear). |
| 2 `is_protector` UUID match | VERIFIED (already correct) | `CreakingHeartBlockEntity.java:333-335` (`map(c -> c == creaking).orElse(false)`) -> `block/entities/creaking_heart.rs:129-133` `is_some_and(|uuid| uuid == creaking_uuid)`; test `unbound_heart_protects_no_creaking` (:405). |
| 3 mob 34-block teardown | VERIFIED (fixed) | `world/entity/monster/creaking/Creaking.java:241-258` mob side only checks `isProtector` then sets health 0 (`entity/mob/creaking.rs:603-614`); the 34-block rule lives heart-side (`CreakingHeartBlockEntity.java:130`, `entities/creaking_heart.rs:246-251`, `CREAKING_TOO_FAR_DISTANCE=34.0`); test `creaking_too_far_boundary` (:415). |
| 4 creaking wander/melee goals | FIXED (observable parity complete) | `world/entity/monster/creaking/CreakingAi.java:30-85` + `Creaking.java:135,166,190,195,222-225,454-458,499-519,601-611` reproduced in `entity/mob/creaking.rs`: `creaking_is_active` gate on ActiveTargetGoal (:79-83,194-199); MeleeAttackGoal(1.0,false) interval 40 gated by canMove (:173-180); WanderAroundGoal(0.3) gated by canMove (:181-187); `mob_is_pushable` (:493-495); `mob_knockback` returns `!can_move()` (:497-509); ENTITY_ACTION game events (:580, :activate/deactivate); 32-block home restriction (:108-109, :253-269). Tests `dormant_creaking_disables_target_gate`, `frozen_creaking_disables_goal_gate`, `frozen_creaking_ignores_knockback` (:701-752). Full vanilla Brain/Activity behaviour-tree internals are not reproduced, but AGENTS.md accepts same observable behaviour with a different algorithm. |
| 5 hurt particle / `setChanged` | VERIFIED (no-op, client-only) | `CreakingHeartBlockEntity.java:228-250` `creakingHurt` only emits client particles and seeds an emitter/sound replay; no `setChanged`, no server divergence. Server-observable parts (resin clumps, `RESIN_PLACE` sound, `BLOCK_PLACE` game event) already in `entities/creaking_heart.rs:253-277`. |
| 6 piston push | VERIFIED (already correct) | `world/level/block/piston/PistonBaseBlock.java:243-285` `isPushable` returns `!state.hasBlockEntity()` (:284); `block/blocks/piston/piston.rs:98` uses the equivalent `!has_block_block_entity(block)`, and CREAKING_HEART carries a block entity, so it is already unpushable. |

Also re-verified comparator rounding: `CreakingHeartBlockEntity.java:337-348` is
`SIGNAL_MAX - (int)floor(clamp(distance,0,32)/32*15)`, matching `blocks/creaking_heart.rs:125-149`, and the
`onStacksDropped` experience (20-24 XP, natural hearts only, Silk Touch/creative excluded) at
`world/level/block/CreakingHeartBlock.java` matching `blocks/creaking_heart.rs:151-170`.

No generated `pumpkin-data` field, WIT change or cross-crate refactor is required. No source file was
changed in this pass (all fixes were already present and verified). Every row has a terminal outcome. Not compiled
in this role; orchestrator batch check.
