# Tracker: audit / crates/pumpkin-core/src/block/entities/copper_golem_statue.rs

Sources reviewed: `crates/pumpkin-core/src/block/entities/copper_golem_statue.rs`,
`crates/pumpkin-core/src/block/blocks/weathering_copper.rs:848-1027`,
`crates/pumpkin-core/src/entity/passive/copper_golem.rs:154-202, 297-339`,
`crates/pumpkin-core/src/net/java/play/pick_item.rs:30-54`,
`crates/pumpkin-core/src/block/registry.rs:919-944`,
`crates/pumpkin-core/src/block/mod.rs:633-675`,
`crates/pumpkin-core/src/block/entities/mod.rs:120-209`,
generated data `crates/pumpkin-data/src/generated/loot_table.rs:21728-21752`.
There are no TODO/FIXME markers in the file; the five `vanilla` markers are test
identifiers (lines 178, 185, 186, 188, 191).

## Fixed in this pass

None. Every behaviour this entity owns already matches vanilla; no change was
justified (WORKER.md rule 12).

## Verified matching vanilla (no change)

- NBT layout: the custom name is stored in the block-entity `components` compound
  under `minecraft:custom_name` (`write_nbt`, lines 54-68) and read from the same
  path, with `custom_name`/`CustomName` read fallbacks (lines 41-46). This is the
  1.21.5+ block-entity component layout used by the 26.3 data (`CURRENT_MC_VERSION`).
- `create_statue` (lines 121-127) copies the golem's custom name and marks the
  entity dirty, matching `CopperGolemStatueBlockEntity.createStatue`.
- `remove_statue` (lines 129-158) spawns the golem at block centre (x+0.5, y, z+0.5),
  copies the custom name, and uses `Direction.toYRot()` values for the facing
  (SOUTH 0, WEST 90, NORTH 180, EAST 270). `Entity::set_rotation` also sets
  `head_yaw` and the body yaw is stored, matching `setYRot`/`setYHeadRot`/`setYBodyRot`.
- Spawned golems are already persistent: `CopperGolemEntity::new` stores
  `persistence_required = true` (`entity/passive/copper_golem.rs:191-194`), matching
  vanilla `setPersistenceRequired()` in `removeStatue`.
- Drop path: the generated loot table for the statue copies only
  `minecraft:custom_name` from the block entity
  (`loot_table.rs:21735-21744`); `drop_loot` applies `collect_item_components` to the
  matching dropped item (`block/mod.rs:665-671`), so the name survives breaking and
  the pose does not, exactly like vanilla's loot table.
- Pick block: the pose is written into the item's `minecraft:block_state`
  component (`get_item`, lines 160-168) and applied again on placement
  (`blocks/weathering_copper.rs:900-924`). Creative include-data pick also carries
  the block-entity NBT (`pick_item.rs:34-54`), matching vanilla `getCloneItemStack` +
  `BLOCK_STATE`.
- `apply_item_components` reads only `CustomNameImpl` and marks the entity dirty;
  the placement path applies it after `BlockEntityDataImpl`
  (`block/registry.rs:919-944`), so placing a renamed statue restores the name.

## Remaining gaps (larger)

| Gap | Rust | Vanilla | Suggested fix | Size |
|---|---|---|---|---|
| Nameless statues broadcast an empty `CBlockEntityData` on placement and on every dirty tick, because `chunk_data_nbt` always returns `Some` (`update_packet_nbt` defaults to it, `entities/mod.rs:150-155`) | `block/entities/copper_golem_statue.rs:70-74`; `world/mod.rs:8191-8201` | `CopperGolemStatueBlockEntity` does not override `getUpdatePacket`, so vanilla sends no `ClientboundBlockEntityDataPacket` for statues; nothing in the name is rendered client-side | return `None` from `chunk_data_nbt` when `custom_name` is `None` (keep persistence through `write_nbt`, which already gates on the name). Check `add_block_entity` ordering before changing, since `chunk_data_nbt` there also seeds the chunk-side NBT | S |
| `BlockEntity::get_custom_name` is not implemented, so a named statue reports no name to generic callers | `block/entities/copper_golem_statue.rs:22-26` | the vanilla block entity stores the component and `BlockEntity#getCustomName` returns it | return the locked `custom_name` clone; today only chest/container name callers use it (`block/blocks/chests.rs:475`, `entity/player.rs:7633`), so no gameplay change is observable yet | S |
| `apply_item_components` marks dirty unconditionally, even when the placed item has no custom-name component | `block/entities/copper_golem_statue.rs:87-95` | `BlockEntity.applyComponentsFromItemStack` is a no-op when no components are present | compare with the previous value before storing the dirty flag | S |

## Notes

- No decompiled vanilla source is vendored in this worktree (`.vendor/` is absent),
  so vanilla cross-checks used the generated 26.3 loot table, block-property data and
  `docs/VANILLA_REF.md` rather than the Java class.
