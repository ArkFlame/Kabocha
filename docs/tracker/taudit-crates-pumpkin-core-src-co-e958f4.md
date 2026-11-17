# command / locate (DEEP AUDIT)

Target: `crates/pumpkin-core/src/command/commands/locate.rs` (no TODO/FIXME
markers; the tracker's "vanilla" hits were explanatory comments).

Reference: decompiled Paper/Canvas 26.3 tree in
`/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft`
(`server/commands/LocateCommand.java`, `world/level/chunk/ChunkGenerator.java`,
`world/level/biome/BiomeSource.java`, `util/Mth.java`,
`world/level/levelgen/structure/placement/StructurePlacement.java`,
`world/entity/ai/village/poi/PoiManager.java`,
`ca/spottedleaf/moonrise/patches/poi_lookup/PoiAccess.java`).

Already matching vanilla (checked, no change needed):

- Argument types: structure uses `ResourceOrTagKeyArgument` (vanilla
  `resourceOrTagKey`), biome/poi use the validated `ResourceOrTagArgument`.
- Biome `6400/32/64` and poi `256` search parameters (`LocateCommand` fields).
- Structure resolution per structure, not per set (`wanted_structures`,
  `getHolders`); known-but-empty tag -> `not_found`, unknown id/tag ->
  `invalid`, like `LocateCommand.getHolders` + `ERROR_STRUCTURE_INVALID`.
- Reported structure position is `StructurePlacement#getLocatePos`
  (chunk min block, y=0): confirmed no structure set in this data version sets
  `locate_offset` (generated `StructureSet` consts: 52 entries, 52 unique
  structures, none in two sets).
- Horizontal distance for structure/poi and 3D distance for biome, the `~` y
  and `/tp @s x ~ z` click command (`LocateCommand.showLocateResult`).
- Tag result names: `#ns:path (minecraft:found)` from `asPrintable()` plus
  the registered name.
- Biome sample Y order matches `Mth.outFromOrigin` (`out_from_origin` test in
  `biome_finder.rs`).

Fixed in earlier passes:

- `crates/pumpkin-world/src/generation/generator/biome_finder.rs` -
  `/locate biome` now replays `BlockPos.spiralAround` leg order
  (`spiral_offsets`), not per-ring row-by-row scanning.

Fixed in this pass:

- `locate.rs` (`horizontal_distance`, `absolute_distance`) - vanilla computes
  `Mth.floor((float) Math.hypot(dx, dz))` and
  `Mth.floor(Mth.sqrt((float) distSqr))`, i.e. through `f32`; both helpers now
  cast before the root/floor. Test `biome_distance_rounds_through_f32`.
- `locate.rs` (`poi_within_radius`) - Paper's
  `PoiAccess.findNearestPoiRecords` bounds candidates by a 3D sphere of
  squared radius 256^2, not the enclosing x/z square; the poi executor now
  uses `find_closest_where` with that predicate. Test
  `poi_search_is_bounded_by_3d_radius`.
- `locate.rs` (`STRUCTURE_SEARCH_RADIUS_REGIONS`) - the radius given to
  `find_nearest_structure*` is counted in placement regions, and vanilla's
  `ChunkGenerator.findNearestMapStructure` multiplies each ring offset by the
  placement spacing (`sectorX = chunkOriginX + spacing * x`, x in
  `-100..=100`). Passing `100 / spacing + 1` under-searched by up to 30x, so
  `/locate structure` reported not-found for structures vanilla reaches.

Remaining gaps:

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| `locate.rs` stronghold arm | `LocateCommand#locateStructure` blocks in `ChunkGenerator#findNearestMapStructure`; Canvas sends `Searching for <name>...` and delivers the result later | Pending stronghold rings return the hard-coded English literal `Stronghold positions are still being calculated, try again shortly` as a command error | Send the "Searching for ..." feedback and return 0 (Canvas behaviour), or add async result delivery on the tick loop | S |
| `locate.rs` (`wanted_structures`) | `LocateCommand#getHolders` resolves through `Registries.STRUCTURE`, so datapack/modded structures are locatable | Non-`minecraft:` ids always fail because resolution goes through the generated `StructureKeys` enum; same for tag members | Needs a dynamic structure registry instead of `StructureKeys::from_name` | L |
| `locate.rs` poi executor vs `pumpkin-command/src/argument_types/resource_or_tag.rs` | `ResourceOrTagArgument.resourceOrTag(Registries.POINT_OF_INTEREST_TYPE)` rejects an unknown poi id at parse time | POI ids are accepted as-is (no generated POI registry), so `/locate poi bogus:x` reports `commands.locate.poi.not_found` instead of a parse error | Generate a POI type registry in `pumpkin-data` and validate in `ResourceOrTagArgument::parse` | M |
| `locate.rs` (`structure_set_containing`) | `ChunkGeneratorStructureState#getPlacementsForStructure` returns every placement of a structure | Only the first set containing the structure is searched, so a datapack structure placed in two sets would miss one placement (current data has no such duplicate) | Return all matching set indices and scan each | S |
| `pumpkin-world/.../structure_finder.rs` random-spread loop | `ChunkGenerator` evaluates every placement at radius r before choosing the nearest, and returns at the first radius with any hit | The finder breaks at the first placement that hits at radius r, and `locate.rs` then compares per-set results by distance, so cross-set ordering can differ | Collect all placements' hits per ring, pick the nearest, then advance | S |
| `pumpkin-world/.../structure_finder.rs` (`find_nearest_concentric`) | Stronghold candidates rank by `pos.distSqr` with candidate y=32 (reported y stays 0 via `getLocatePos`) | Candidate y is 0, so 3D ranking ignores the 32 block offset (unobservable unless mixed with a horizontally tied candidate) | Rank concentric candidates at y=32 | XS |
