# command / locate (DEEP AUDIT)

Target: `crates/pumpkin-core/src/command/commands/locate.rs` (515 lines, no
TODO/FIXME markers; the tracker's "vanilla" hits were explanatory comments).

Reference: decompiled Paper/Canvas 26.3 tree in
`/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft`
(`server/commands/LocateCommand.java`, `world/level/chunk/ChunkGenerator.java`,
`world/level/biome/BiomeSource.java`, `util/Mth.java`,
`world/level/levelgen/structure/placement/StructurePlacement.java`,
`world/entity/ai/village/poi/PoiManager.java`,
`ca/spottedleaf/moonrise/patches/poi_lookup/PoiAccess.java`).

Already matching vanilla (checked, no change needed):

- Both search constants and the argument types: structure uses
  `ResourceOrTagKeyArgument` (vanilla `resourceOrTagKey`), biome/poi use the
  validated `ResourceOrTagArgument`; radius `100` region rings, biome
  `6400/32/64`, poi `256` (`LocateCommand` fields).
- Structure resolution per structure, not per set (`wanted_structures`,
  `getHolders`); known-but-empty tag -> `not_found`, unknown id/tag ->
  `invalid`, like `LocateCommand.getHolders` + `ERROR_STRUCTURE_INVALID`.
- Reported structure position is `StructurePlacement#getLocatePos`
  (chunk min block, y=0): confirmed no structure set in this data version sets
  `locate_offset` (generated `StructureSet` consts: 52 entries, 52 unique
  structures, none in two sets).
- Distances: `Mth.floor(Math.hypot(dx, dz))` for structure/poi and
  `Mth.floor(Mth.sqrt(distSqr))` for biome, plus the `~` y and
  `/tp @s x ~ z` click command (`LocateCommand.showLocateResult`).
- Tag result names: `#ns:path (minecraft:found)` from `asPrintable()` plus
  `getRegisteredName()`.
- Biome sample Y order matches `Mth.outFromOrigin` (`out_from_origin` test in
  `biome_finder.rs`).

Fixed in this pass:

- `crates/pumpkin-world/src/generation/generator/biome_finder.rs:98-150` -
  `/locate biome` scanned each Chebyshev ring row by row; vanilla
  `BiomeSource#findClosestBiome3d` walks `BlockPos.spiralAround(ZERO, radius,
  EAST, SOUTH)`. The first matching sample wins, so equidistant biomes in one
  ring resolved to a different column. Now replayed with the exact vanilla leg
  order (`spiral_offsets`, test `sample_columns_follow_vanilla_spiral_order`).

Remaining gaps:

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| `locate.rs:61-64,256-258` | `LocateCommand#locateStructure` blocks in `ChunkGenerator#findNearestMapStructure` and then reports; Canvas sends `Searching for <name>...` and delivers the result later | Pending stronghold rings return the hard-coded English literal `Stronghold positions are still being calculated, try again shortly` as a command error | Send the "Searching for ..." feedback and return 0 (Canvas behaviour), or add async result delivery on the tick loop | S |
| `locate.rs:127-143` (`wanted_structures`) | `LocateCommand#getHolders` resolves through `Registries.STRUCTURE`, so datapack/modded structures are locatable | Non-`minecraft:` ids always fail because resolution goes through the generated `StructureKeys` enum; same for tag members | Needs a dynamic structure registry instead of `StructureKeys::from_name` | L |
| `locate.rs:374-389` vs `pumpkin-command/src/argument_types/resource_or_tag.rs:177-213` | `ResourceOrTagArgument.resourceOrTag(Registries.POINT_OF_INTEREST_TYPE)` rejects an unknown poi id at parse time ("Unknown poi type") | POI ids are accepted as-is (no generated POI registry), so `/locate poi bogus:x` reports `commands.locate.poi.not_found` instead of a parse error | Generate a POI type registry in `pumpkin-data` and validate in `ResourceOrTagArgument::parse` | M |
| `locate.rs:150-154` (`structure_set_containing`) | `ChunkGeneratorStructureState#getPlacementsForStructure` returns every placement of a structure | Only the first set containing the structure is searched, so a datapack structure placed in two sets would miss one placement | Return all matching set indices and scan each | S |
| `locate.rs:399-401` | Vanilla `PoiManager#getInSquare` filters a horizontal square; Paper (`PoiAccess#findNearestPoiRecord`) additionally rejects `distSqr > radius^2` (3D sphere) | Square-only bound can report a poi inside the square but outside Paper's 3D radius (e.g. dx=250, dy=250 -> 354 blocks) | Pass a `find_closest_where` predicate that also checks the 3D distance | S |
| `locate.rs:338-345` | `Mth.floor(Mth.sqrt((float) sourcePos.distSqr(foundPos)))` (f32 sqrt) | Biome distance uses an f64 sqrt; can differ by 1 when the f32 cast rounds across an integer | Cast through `f32` before the square root | XS |
