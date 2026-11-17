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
10. spawn_egg.rs:359-361 - the peaceful rejection returns `BlockActionResult::Fail`; vanilla
   `useOn` always returns CONSUME and lets `EntityType.spawn` fail internally. Client-only
   (swing/prediction). Size S.
11. dispenser.rs:639-682 - `dispense_spawn_egg` emits ENTITY_PLACE at the dispenser position;
   vanilla `EntityType.spawn` emits it at the spawned entity position. Event-position only. Size S.
12. block/entities/mob_spawner.rs:294 - `set_entity_type` only swaps the type; vanilla
   `BaseSpawner.setEntityId` also resets pending spawn data/potentials. Pumpkin's spawner has a
   flat `entity_type` and no potentials, so there is nothing to reset today; revisit if per-spawn
   NBT is added. Size S (informational).
