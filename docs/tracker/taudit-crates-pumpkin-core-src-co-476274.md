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
