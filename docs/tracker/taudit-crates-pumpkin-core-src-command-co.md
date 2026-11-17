# Audit: crates/pumpkin-core/src/command/commands/locate.rs

Tracker row: audit / crates/pumpkin-core/src/command/commands/locate.rs (coverage AUDIT).
The "10 TODO/FIXME/vanilla markers" are the word vanilla in doc comments; the file has no
TODO, FIXME, XXX or HACK markers.

Checked against vanilla LocateCommand (1.21): /locate structure|biome|poi, permission level
2, error types and translation keys, chat.coordinates + chat.square_brackets composition,
tilde vs absolute y, click/hover events, printable name and "(found)" suffix for tags,
horizontal distance for structures/POIs vs 3D distance for biomes, 6400/32/64 biome probe,
256 POI radius, one structure-set placement per requested structure.

## Fixed

- locate.rs (old line 38-40, 249, 270): vanilla passes 100 chunks to
  ChunkGenerator.findNearestMapStructure, but find_nearest_structure* counts
  chunk-region rings (structure_finder.rs:22, 81-100). Passing 100 searched up to
  ~3400 chunks (~51k blocks) instead of 1600, and scanned up to 100 region rings for a
  structure that never generates. Now converted by structure_search_rings(set) in the same
  file: chunks / spacing + 1 rings per set.

## Remaining gaps

1. Other callers share the radius-unit bug: crates/pumpkin-core/src/entity/passive/villager/mod.rs:566
   (vanilla cartographer maps search 100 chunks) and
   crates/pumpkin-core/src/entity/passive/dolphin/goals.rs:165 (vanilla dolphin treasure
   search 50 chunks) still pass chunk counts as region rings, over-searching ~32x.
   Fix: change find_nearest_structure* in
   crates/pumpkin-world/src/generation/generator/structure_finder.rs to take a chunk radius
   and do the chunks / placement.spacing + 1 conversion internally, then drop the
   per-caller conversion in locate.rs. Size M.
2. Datapack/modded structure ids are rejected: locate.rs:127-143 gates on id.is_vanilla()
   and resolves through the generated StructureKeys, so vanilla resolves any entry of the
   dynamic worldgen/structure registry (LocateCommand.getHolders) while Pumpkin returns
   commands.locate.structure.invalid. Fix: resolve via a runtime structure registry
   instead of the generated enum. Size L.
3. Argument node type: locate.rs:438 uses ResourceOrTagKeyArgument, vanilla uses
   ResourceOrTagArgument.resourceOrTag(Registries.STRUCTURE). An unknown structure id
   therefore parses and reaches commands.locate.structure.invalid instead of failing
   argument parsing; the wanted_structures -> None branch only fires for this case and for
   unknown tags. Fix: switch to ResourceOrTagArgument and drop the dead branch. Size S
   (cosmetic: different error text for a bad id).
4. Datapack biomes and POI types are dropped silently: locate.rs:312 Biome::from_name and
   locate.rs:320/377 tag members filtered against the generated registry, so /locate biome
   on a datapack biome reports commands.locate.biome.not_found. Same root cause as (2).
   Size L.
5. POI nearest selection filters a square, not a circle:
   crates/pumpkin-world/src/poi/mod.rs:949-954 clamps abs(dx) <= radius && abs(dz) <= radius,
   vanilla (PoiManager.findAll / findAllClosestFirstWithType) keeps POIs with
   distSqr < radius * radius. A corner POI at ~360 blocks can be returned by /locate poi
   inside a 256 radius. Fix: compare distance_sq against radius * radius. Size S.
6. Stronghold Pending aborts a tag search: locate.rs:256-258 returns
   STRUCTURE_PENDING_ERROR_TYPE from inside the per-set loop, discarding a hit already found
   for another requested structure. Vanilla rings are precomputed, so it never aborts.
   Fix: remember the pending state and only surface it when no hit was found. Size S.
