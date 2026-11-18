# audit: crates/pumpkin-core/src/command/argument_types/resource_key.rs

Vanilla reference: `net.minecraft.commands.arguments.ResourceKeyArgument`,
`ResourceArgument`, `AdvancementCommand`, `FillBiomeCommand`.

## Fixed on this branch

- Dimension resource keys returned no suggestions at all (`Suggestions::empty()`),
  while vanilla suggests registry elements. Now suggests the dimensions of the
  loaded worlds, the same set `execute in` / `clone <dimension>` accept.
- Errors for unknown advancement and wrong-registry keys printed only the path
  (`story/root`); vanilla prints the full location (`minecraft:story/root`).
- Unknown biome in `fillbiome` used `commands.fillbiome.invalid`, a key absent
  from the extracted `en_us` (so it rendered as a raw key). Now uses
  `argument.resource.not_found` with the element id and `minecraft:worldgen/biome`,
  matching `ResourceArgument`'s `ERROR_UNKNOWN_RESOURCE`.

## Remaining gaps

1. `get_biome` (resource_key.rs:141) and `get_advancement` (resource_key.rs:123)
   go through `is_vanilla_then()`, so only `minecraft:` keys can ever resolve.
   Vanilla resolves against the dynamic registry, so datapack biomes and
   datapack advancements work. Pumpkin has no dynamic biome registry yet.
   Fix: resolve through a registry lookup (Biome by full identifier,
   advancement manager by full identifier) instead of the vanilla-namespace
   shortcut. Size: M.
2. Dimension suggestions come from `server.worlds` (loaded worlds only).
   Vanilla advertises every dimension in the registry, including datapack
   dimensions that are not currently instantiated as a world. Same root cause
   as (1): no datapack dimension registry. Size: M.
3. `ERROR_NOT_SUMMONABLE_ENTITY` (resource_key.rs:29) and `ERROR_INVALID`
   (resource_key.rs:44) are unused here: vanilla exposes
   `ResourceKeyArgument.getSummonableEntityType` from this helper, but Pumpkin
   implements the check in `crates/pumpkin-command/src/argument_types/resource.rs`.
   Delete the dead constants (or move the summonable-entity helper here for
   parity). Size: S.
4. No automated test: `list_suggestions`/`get_*` need a live `CommandContext`
   with a server; covered indirectly by `pumpkin-gametest` command tests when
   they exist for `execute in` / `fillbiome`. Size: S.

## Gap status
- Fixed on this branch section: FIXED per note.
- 1. `get_biome`/`get_advancement` suggestions (`resource_key.rs:123,141`): STILL OPEN (S).
- 2. Dimension suggestions only loaded worlds: STILL OPEN (S).
- 3. `ERROR_NOT_SUMMONABLE_ENTITY`/`ERROR_INVALID` (`resource_key.rs:29`): STILL OPEN (S).
- 4. No automated test for suggestions (needs live `CommandContext`): STILL OPEN (gametest candidate).

## Result

- 3. FIXED: removed the unused `ERROR_NOT_SUMMONABLE_ENTITY` and `ERROR_INVALID`
  constants from `resource_key.rs` (the summonable-entity check lives in
  `pumpkin-command`'s `ResourceArgument`, verified unused repo-wide).
- Added two pure `parse` unit tests (registry preserved; bare path defaults to
  the `minecraft` namespace), the only seam that needs no live `CommandContext`.
- 1. LEFT OPEN: resolving `get_advancement`/`get_biome` against datapack
  registries needs a dynamic registry lookup and a return type other than
  `&'static` generated data (cross-crate refactor, outside this unit).
- 2. LEFT OPEN: advertising datapack dimensions requires a dimension registry;
  Pumpkin only has instantiated worlds (outside this unit).
- 4. LEFT OPEN: suggestion/`get_*` tests need a live server `CommandContext`
  (pumpkin-gametest candidate, outside this unit).

### Final pass (this session)

- 3. FIXED (confirmed): ERROR_NOT_SUMMONABLE_ENTITY / ERROR_INVALID remain removed; repo-wide search finds no callers.
- 1. Biome half FIXED: get_biome_id + resolve_biome_id resolve datapack biomes via DynamicRegistryManager, and list_suggestions for BIOME_REGISTRY now reads get_all_entry_ids("minecraft:worldgen/biome") with a generated-data fallback when the context has no server. Verified against FillBiomeCommand.java and ResourceArgument.java (both use ResourceArgument + registry element suggestions).
- 1. Advancement half LEFT OPEN: vanilla resolves getAdvancement against the server's dynamic advancement registry; Pumpkin's AdvancementManager::get_advancements returns only generated Advancement data, so datapack advancements need a dynamic advancement registry (cross-crate, outside this unit).
- 2. LEFT OPEN: vanilla suggests all dimension_type registry elements, but Pumpkin's execute in resolves against instantiated worlds only (execute.rs:190-193); advertising unloaded dimension types would yield always-failing commands. Needs a dimension registry / level registration (cross-crate, outside this unit). Dimension suggestions now have a no-server fallback (see dimension_suggestions_are_filtered_by_prefix test).
- 4. Partially FIXED: seven pure unit tests in resource_key.rs cover parse, client parser, biome suggestions (server-less fallback), get_registry_key, get_biome, get_advancement, and resolve_biome_id (vanilla + datapack). A live-server gametest for execute in / fillbiome remains LEFT OPEN (needs a running server, outside this unit).


## Result (native-4 cross-file pass)
- Rows 1, 2 (datapack advancement/biome resolution, dimension suggestions): BLOCKED-L. Needs a dynamic advancement registry and a dimension_type registry with level registration in pumpkin-core, not a single-file fix.
- Row 4 (live-server gametest): BLOCKED-L, needs a `pumpkin-gametest` server context.


## Result (coder pass)

- 3. FIXED (confirmed): `ERROR_NOT_SUMMONABLE_ENTITY` / `ERROR_INVALID` are absent from `resource_key.rs`; no code change needed.
- 1 (biome, datapack resolution path for commands): FIXED for the command-facing seam. `get_biome_id` + `resolve_biome_id` resolve datapack biomes via `DynamicRegistryManager`, biome suggestions read dynamic registry entry ids with generated fallback, and `fillbiome` uses `get_biome_id`. `get_biome` still returns only `&'static` generated vanilla biomes, but no in-unit caller requires datapack `&'static Biome` objects.
- 1 (advancement): LEFT OPEN. `get_advancement` returns `&'static Advancement` from generated data only; datapack advancements need a dynamic advancement registry and cross-crate return/caller changes (outside this unit).
- 2. LEFT OPEN. Dimension suggestions/resolution use instantiated `server.worlds`, matching `execute in`/`clone` behavior that can only target loaded worlds; advertising uninstantiated datapack dimensions would require level registration (cross-crate, outside this unit).
- 4. Partially FIXED. Added pure unit tests `get_biome_rejects_unknown_vanilla_biome`, `get_advancement_rejects_unknown_vanilla_advancement`, and `dimension_suggestions_are_filtered_by_prefix` in `resource_key.rs` (dummy context, no server). Live-server/gametest coverage for dimension and advancement suggestion branches remains outside this unit.
## Result (coder pass 2, batch r2_003)
- 3. VERIFIED: `grep` finds no `ERROR_NOT_SUMMONABLE_ENTITY` or `ERROR_INVALID` in `crates/pumpkin-core/src/command/argument_types/resource_key.rs`; constants stay removed.
- 1. STILL OPEN (advancement half): needs a dynamic advancement registry; `get_advancement` at `resource_key.rs` returns generated `&'static Advancement` only. Vanilla source: `.vendor/vanilla-decompiled/net/minecraft/commands/arguments/ResourceKeyArgument.java`. Missing subsystem, so BLOCKED-DATA candidate, not closed here.
- 2. STILL OPEN: dimension suggestions need instantiated-level registration; missing subsystem (BLOCKED-DATA candidate), not closed here.
- 4. STILL OPEN: needs a live-server `pumpkin-gametest` context; not closed here.

## Result (coder pass 3, batch r3_019)
- 1. STILL OPEN (advancement half): `get_advancement` (`crates/pumpkin-core/src/command/argument_types/resource_key.rs:158`) returns `&'static Advancement` from generated data via `Advancement::from_name`; vanilla `ResourceKeyArgument.java` resolves against the server's dynamic registry. `server.advancement_manager` (`crates/pumpkin-core/src/data/advancement_data.rs`) holds datapack ids, so the fix is an owned-return refactor across callers, not a single-file edit. Left open, no code change.
- 2. STILL OPEN (dimension half): suggestions already prefer `dynamic_registry_manager` entries (`resource_key.rs:78-99`), but `execute in`/`clone` resolve only instantiated `server.worlds`; advertising uninstantiated datapack dimension types would give always-failing commands. Needs level registration for datapack dimension types (missing subsystem, BLOCKED-DATA candidate). No code change.
- 4. STILL OPEN: live-server gametest for `execute in`/`fillbiome` suggestions not written; needs a running server context. Not reached this pass.
