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

## Fixed in this audit

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
