# audit: crates/pumpkin-core/src/item/items/spawn_egg.rs

Tracker row: `audit / crates/pumpkin-core/src/item/items/spawn_egg.rs` (coverage AUDIT).
Vanilla reference: `SpawnEggItem`, `EntityType.appendDefaultStackConfig`/`updateCustomEntityTag`,
`Mob.spawnOffspringFromSpawnEgg`, `SpawnerBlockEntity.setEntityId`, `TrialSpawnerBlockEntity`.
No TODO/FIXME markers in the file; the 10 "vanilla" hits are provenance comments.

## Verified matching

- `use` in water/lava: source-only fluid clip, centre at block x/z with block y, random yaw via
  `wrap_degrees`, `GameEvent.ENTITY_PLACE`, consume, `Stats.ITEM_USED` (one award; see gap 5).
- `useOn`: spawner and trial-spawner branch before the peaceful guard, `BLOCK_CHANGE`, consume;
  spawn branch places above on non-empty collision shape, `ENTITY_PLACE`, no consume when the
  spawn event cancels.
- `EntityType.updateCustomEntityTag` order: component type gate, then op-only NBT gate, then
  saveWithoutId/merge/load. `only_op_can_set_nbt` set matches vanilla's op-only types.
- `appendDefaultStackConfig` order: item components (custom name, variants, collars, tropical
  fish, sound variants) first, `entity_data` last.
- Spawn-egg-on-entity: same-type gate, `get_breed_offspring` with the clicked animal as both
  parents, frog returns null, no offspring without a baby form, `on_offspring_spawned_from_egg`.

## Fixed in this audit (2026-11-17 pass)

- spawn_egg.rs:430-431 - `use_on_entity` gave the offspring a random yaw. Vanilla
  `spawnOffspringFromSpawnEgg` does `breedOffspring.moveTo(pos, 0.0F, 0.0F)` (Paper 1.21.4
  SpawnEggItem patch, hunk 2), i.e. the baby faces north, while the block/fluid spawn paths keep
  `EntityType.spawn`'s random yaw. Now `set_rotation(0.0, 0.0)`.

## Fixed in this audit (earlier pass)

1. spawn_egg.rs:278-282 - `SpawnEggItem.use` checks `Level.mayInteract`/`Player.mayUseItemAt`
   before spawning into a fluid; Pumpkin had no spawn-protection gate on that path, so a
   non-op could place mobs inside the spawn protection radius. Now returns before spawning.
   (World border half of `mayUseItemAt` still unported, see gap 6.)

## Remaining gaps

1. spawn_egg.rs:408-422 - spawn-egg-on-entity has no adult gate on the clicked mob and does not
   restrict itself to `Animal`/`AgeableMob` before `from_type`. Vanilla
   `Mob.spawnOffspringFromSpawnEgg` must be re-read (26.3) to confirm whether a baby clicked mob
   yields offspring at all; current code breeds from babies too. Size S.
2. spawn_egg.rs:124-129 - `apply_entity_data` strips `id`, `UUID` and `Pos`; vanilla
   `TypedEntityData.copyTagWithoutId` strips only `id`, so an op-authored `Pos`/`UUID` may apply
   in vanilla. Needs the vanilla class to decide whether stripping is deliberate. Size S.
3. spawn_egg.rs:347-356 - trial spawner branch calls `TrialSpawnerBlockEntity::set_entity_type`
   without vanilla's supported-type check and loot-table/config regeneration; compare with
   `TrialSpawnerBlockEntity.setEntityId`. Size M.
4. spawn_egg.rs:336-341 - spawner branch updates through `update_block_entity` plus
   `BLOCK_CHANGE`; vanilla also does `blockState.sendBlockUpdated(..., 3)` and uses
   `shrink(1)` while Pumpkin uses `decrement_unless_creative`. Verify creative consumption and
   the block-state update (comparator/has_record). Size S.
5. spawn_egg.rs:319 + net/java/play/use_item.rs:40 - the air-use handler already awards
   `Stats.ITEM_USED` unconditionally, and `normal_use_with_rotation` awards it again, so a spawn
   egg used in water counts 2. Same double award exists for other air-use items (e.g.
   wind_charge.rs:76). Root cause is the unconditional award in use_item.rs (vanilla only awards
   when `InteractionResult.consumesAction()`); fixing only this file would regress the Bedrock
   path, which does not award it. Size S, cross-file.
6. spawn_egg.rs:281 - `is_in_spawn_protection` covers the block-protection half of vanilla's
   `Level.mayInteract`; the world-border half of `Player.mayUseItemAt` is still missing from the
   air-use path. Size S.
7. No automated test covers spawn eggs (none in the file, none in `pumpkin-gametest`). Smallest
   useful test: spawn egg with `entity_data` for a variant mob applies the component and the
   peaceful guard blocks monsters, driven through the gametest world. Size M.

## Additional gaps found in the 2026-11-17 pass

8. spawn_egg.rs:413-429 - the `None` (non-animal) arm of `use_on_entity` synthesises a mob with
   `from_type` and then makes it a baby, so a zombie egg used on a zombie consumes the egg and
   spawns a baby zombie. Vanilla `interactLivingEntity`/ `spawnOffspringFromSpawnEgg` work from
   `getBreedOffspring` (AgeableMob) and return PASS otherwise; the exact 26.3 guard (baby parent,
   spawned type) must be read before changing. Size M.
9. spawn_egg.rs:447-458 - `ItemBehaviour::use_on_entity` returns `()`, so a successful spawn never
   reports InteractionResult.CONSUME; interact.rs:135-176 then does not swing and the client keeps
   its own prediction. Fixing needs the trait return type changed for every item. Size M.
10. FIXED fix(core): return consume for peaceful spawn egg use_on. spawn_egg.rs:359-361 - the peaceful rejection returns `BlockActionResult::Fail`; vanilla
   `useOn` always returns CONSUME and lets `EntityType.spawn` fail internally. Client-only
   (swing/prediction). Size S.
11. FIXED fix(block): emit dispenser spawn egg place at entity pos. dispenser.rs:639-682 - `dispense_spawn_egg` emits ENTITY_PLACE at the dispenser position;
   vanilla `EntityType.spawn` emits it at the spawned entity position. Event-position only. Size S.
12. block/entities/mob_spawner.rs:294 - `set_entity_type` only swaps the type; vanilla
   `BaseSpawner.setEntityId` also resets pending spawn data/potentials. Pumpkin's spawner has a
   flat `entity_type` and no potentials, so there is nothing to reset today; revisit if per-spawn
   NBT is added. Size S (informational).

## Open after the 2026-11-17 f5 pass

- Gap 1: STILL OPEN: needs the 26.3 `spawnOffspringFromSpawnEgg` age guard; no decompiled vanilla in .vendor to confirm.
- Gap 2: STILL OPEN: needs vanilla `Entity.load` handling of `Pos`/`UUID` to decide if stripping is deliberate.
- Gap 3: STILL OPEN: needs vanilla `TrialSpawner.setEntityId` supported-type check and config regeneration source.
- Gap 4: STILL OPEN: needs vanilla SpawnEggItem spawner branch (shrink vs consume, sendBlockUpdated flags) to confirm.
- Gap 5: STILL OPEN: root fix is in net/java/play/use_item.rs and touches every air-use item and the Bedrock path; cross-file, not a single S fix.
- Gap 6: STILL OPEN: world-border half of Player.mayUseItemAt needs the vanilla method body to port exactly.
- Gap 7: STILL OPEN: needs a pumpkin-gametest scenario; no cargo run permitted in this pass.
- Gap 8: STILL OPEN: the non-animal arm needs the vanilla AgeableMob/getBreedOffspring guard for 26.3 before changing.
- Gap 9: STILL OPEN: trait return type change across every ItemBehaviour impl; cross-cutting.
- Gap 12: STILL OPEN: informational; Pumpkin spawner has no pending spawn data or potentials to reset.

## Result

Verified each STILL OPEN row against the decompiled Paper 1.21.4 source in
`.vendor/vanilla-decompiled/net/minecraft/`. Two fixes applied to `spawn_egg.rs`; the rest are
false positives or need files/refactors outside this unit.

- Gap 1: CLOSED (false positive). `Mob.checkAndHandleImportantInteractions` ->
  `SpawnEggItem.spawnOffspringFromSpawnEgg` has no adult gate on the clicked mob: it calls
  `ageableMob.getBreedOffspring(level, ageableMob)` regardless of the parent's age. Pumpkin's
  lack of an adult gate matches vanilla.
- Gap 2: FIXED. `TypedEntityData.loadInto` merges the authored tag over `saveWithoutId` then
  calls `Entity.load`, which reads `Pos`, so an op-authored `Pos` moves the mob; only `id`
  (ignored by `load`) and `UUID` (restored by `loadInto` after `load`) are stripped now.
- Gap 3: STILL OPEN. Vanilla `TrialSpawnerBlockEntity.setEntityId` calls
  `TrialSpawner.overrideEntityToSpawn`: `data.reset()` + `config.overrideEntity(type)` +
  `setState(INACTIVE)`. Porting that reset/regeneration lives in `trial_spawner.rs`, outside
  this unit's files.
- Gap 4: FIXED (consumption half). Vanilla's spawner branch uses `itemStack.shrink(1)` while
  the `spawnMob` paths use `consume(1, user)` (creative-aware). Both spawner/trial-spawner
  branches now use `decrement(1)`. Block-update half needs no change: the block state is
  unchanged and spawners have no comparator output, so `update_block_entity` already sends
  the client the data vanilla's `sendBlockUpdated(..., UPDATE_ALL)` sends.
- Gap 5: STILL OPEN. Root fix needs `ItemBehaviour` to return an `InteractionResult` so
  `use_item.rs` can award `ITEM_USED` only on consumption; cross-cutting trait refactor
  touching every item and the Bedrock path, outside this unit's scope.
- Gap 6: CLOSED (false premise). Decompiled `Player.mayUseItemAt` checks `abilities.mayBuild`
  and adventure-mode `canPlaceOnBlockInAdventureMode`, not the world border; there is no
  world-border gate to port. (Adventure-mode placement restrictions remain a separate feature.)
- Gap 7: STILL OPEN. Needs a `pumpkin-gametest` scenario (file outside this unit) plus a
  compiled run, neither permitted in this pass.
- Gap 8: CLOSED (false positive). `spawnOffspringFromSpawnEgg` keeps non-`AgeableMob` parents:
  the `else` branch does `type.create(level, SPAWN_ITEM_USE)` then `setBaby(true)`, so a zombie
  egg on a zombie spawns a baby zombie in vanilla. Pumpkin's `None => from_type(...)` plus
  `spawn_as_baby` already matches.
- Gap 9: STILL OPEN. Requires changing `ItemBehaviour::use_on_entity` return type across
  every implementation; cross-cutting refactor outside this unit's files.
- Gap 12: STILL OPEN (informational). Pumpkin's spawner stores a flat `entity_type` with no
  pending spawn data or potentials, so vanilla `BaseSpawner.setEntityId`'s reset is a no-op.

No pure unit test added: `apply_entity_data` and the consumption paths need a live
`EntityBase`/world, so the only meaningful coverage is the gametest in gap 7.
