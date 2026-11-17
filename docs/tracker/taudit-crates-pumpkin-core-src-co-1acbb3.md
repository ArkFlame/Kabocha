# command / execute.rs — deep audit

Sources: `crates/pumpkin-core/src/command/commands/execute.rs`.
Compared against vanilla `ExecuteCommand` (1.21.x, Mojang mappings) and Paper (no relevant patches of its own).

## Fixed in this audit
- `execute facing` used the source position, ignoring `execute anchored`: `execute anchored eyes facing entity @e eyes` computed yaw/pitch from the feet. Both `facing <pos>` and `facing entity <targets> <anchor>` now start from `CommandSourceStack#getAnchor().apply` (`anchor_position`, execute.rs:319-326, used at 337 and 359).
- `execute if|unless block` and `execute if|unless biome` resolved coordinates without requiring a loaded chunk; vanilla uses `BlockPosArgument.getLoadedBlockPos`, which raises `argument.pos.unloaded`. Now `get_loaded_block_pos` (execute.rs:394, 490, 507).
- `execute if|unless blocks` only validated the three corner positions; vanilla `ExecuteCommand.checkBlocks` validates every compared source and destination block. Now checked inside the loop (execute.rs:1168-1170).

## Remaining gaps
| # | Location | Vanilla | Gap | Size |
|---|---|---|---|---|
| 1 | execute.rs:1678-1895 (no node) | `ExecuteCommand.register` adds `if function <function>` (1.20.3+); it runs the function through a return-value-capturing source and succeeds when the captured result is non-zero | The whole `if function` condition is missing. Pumpkin has a `function` command but no return-value capture surface to reuse. | M |
| 2 | execute.rs:1678-1895 (no node) | `execute if items block <pos> <slots> <item_predicate>` and `if items entity <target> <slots> <item_predicate>` (1.21.5+) | Condition is missing; it needs slot-range and item-predicate argument types. | L |
| 3 | execute.rs:192-196 | `positionedOver` reads `Heightmap.Types.OCEAN_FLOOR` when asked for `ocean_floor` | The argument falls back to `WorldSurface` because `ChunkHeightmapType` has no `OCEAN_FLOOR` (pumpkin-world/src/chunk/mod.rs:225). Compute the height on demand from section data or start storing the heightmap. | M |
| 4 | execute.rs:252-255 | `if predicate` resolves the predicate id from the datapack at parse time (unknown id is a syntax error) and evaluates it | A `LootPredicate::Reference` always returns false with a debug log; no datapack predicates are loaded. | M |
| 5 | execute.rs:1479-1486 | `on controller` uses `Entity#getControllingPassenger`, which is null unless the entity is a vehicle or ridden mob with a controlling living passenger | Pumpkin takes `passengers.first()`, so any entity carrying a passenger (e.g. a mob with a mounted entity) matches where vanilla yields nothing. | S |
