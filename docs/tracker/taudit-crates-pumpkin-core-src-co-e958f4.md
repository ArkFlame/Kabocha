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
  the registered name (`LocateCommand.java:168` for `ResourceOrTagArgument`,
  `:181-182` for `ResourceOrTagKeyArgument`, where the resource branch is
  `element.identifier().toString()`); `result_name` matches both overloads.
- `includeY` false for structure/poi, true for biome, and the `Mth` distance
  formulas (`LocateCommand.java:196-198`, `:211-215`).
- POI candidate filter `distSqr <= radius^2` with radius 256
  (`PoiManager.java:376-385`, `PoiAccess.java:674` keeps `<=`).
- Error arguments are literal text (Rust `TextComponent::text`) where vanilla
  uses `Component.translatableEscape` (`LocateCommand.java:39-50`).
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
| `locate.rs` structure arm (`:273-279`) | `LocateCommand.java:105-118` searches placement candidates only: `findNearestMapStructure` returns the nearest chunk whose placement predicate holds, without proving the structure generates there | The arm resolves a real structure start, so a candidate whose biome/validity check fails is skipped: `/locate` can report not-found or a farther position where vanilla reports the candidate | Add a key-returning variant of the potential-chunk search (`find_nearest_structure`) and report its candidate, keeping `_start` for explorer maps | S |
| `locate.rs` `send_success` (`:298-306`, `:359-367`) | Bedrock `commands.locate.structure.success` takes (name, x, z, distance) and `commands.locate.biome.success` (name, x, y, z, distance) | `translate_cross` shares one argument list, so the 3 Java args land in the Bedrock placeholders shifted and the distance parameter is missing | Give `send_success` per-platform argument lists (or a `translate_cross` overload that accepts both key/args pairs) | FIXED for Bedrock player sources (`locate.rs` `send_success` uses `bedrock_success_args`, UNVERIFIED test run): M. STILL OPEN (cross-cutting, console or other shared translate sites): `TextContent::Translate` has one `with` list shared by Java and Bedrock packets (`pumpkin-util/src/text/mod.rs`), so a general split needs a new content field touching ~10 match sites in pumpkin-util, pumpkin-core and legacy.rs |

Deep audit in this pass (no behaviour change needed):

- Read `LocateCommand.java` end to end against `locate.rs`: argument types,
  permission level `LEVEL_GAMEMASTERS` (2), `BlockPos.containing`, search
  parameters, per-structure holder resolution, both `showLocateResult`
  overloads, distance formulas, `~` y and the `/tp @s x ~ z` suggest-command,
  `sendSuccess(..., false)`, and the four error types all match.
- Confirmed there is no `TODO`/`FIXME` in the file; the tracker's 9 markers are
  the explanatory `vanilla` comments.

## Gap status
- Stronghold blocking search (`locate.rs` stronghold arm): STILL OPEN (M; `locate.rs` is out of scope for this pass).
- Datapack/modded structures via registry (`locate.rs` `wanted_structures`): STILL OPEN (M).
- POI argument type (`locate.rs` vs `resource_or_tag.rs`): STILL OPEN (S).
- All placements per structure set (`locate.rs` `structure_set_containing`): STILL OPEN (M).
- Random-spread nearest ordering (`structure_finder.rs`): STILL OPEN (M).
- Stronghold distance ranking (`structure_finder.rs` `find_nearest_concentric`): STILL OPEN (S).
- Structure arm placement candidates (`locate.rs:273-279`): STILL OPEN (M).
- Bedrock success arguments (`locate.rs:298-306,359-367`): STILL OPEN (S).


## Result

Verified every STILL OPEN row against decompiled vanilla; fixed the in-scope
rows. `rustfmt --edition 2024` run on both touched files (exit 0).

| Row | Outcome |
|---|---|
| Stronghold blocking search (`locate.rs`) | Already matches vanilla Canvas (`LocateCommand.java:103-112`): the `StructureSearch::Pending` arm sends `Searching for <name>...` and returns 0. No change made. |
| Datapack/modded structures via registry (`wanted_structures`) | STILL OPEN: needs a dynamic `worldgen/structure` registry in generated `pumpkin-data`; `StructureKeys::from_name` cannot resolve custom ids (generated-data / cross-crate change). |
| POI argument type (`resource_or_tag.rs`) | STILL OPEN: needs a generated POI type registry in `pumpkin-data` so `ResourceOrTagArgument::parse` can reject unknown ids; no such registry exists (generated-data change). |
| All placements per structure set (`structure_set_containing`) | FIXED: renamed to `structure_sets_containing`, returning every set index (vanilla `ChunkGeneratorStructureState.getPlacementsForStructure` returns a list); the executor scans each set; added `all_sets_containing_a_structure_are_returned`. |
| Random-spread nearest ordering (`structure_finder.rs`) | FIXED: `find_nearest_structure` now evaluates every placement at a radius, keeps the nearest hit, and advances only after the full ring (vanilla `ChunkGenerator.java:242-271`). |
| Stronghold distance ranking (`find_nearest_concentric`) | FIXED: ring candidates are ranked with y=32 while the reported position stays y=0 (vanilla `ChunkGenerator.java:296-297`). |
| Structure arm placement candidates (`locate.rs:273-279`) | STILL OPEN: vanilla `getStructureGeneratingAt` (`ChunkGenerator.java:349-362`) checks `checkStructurePresence`, so it does not report pure placement-only candidates; the row's premise needs re-verification before adding a key-returning candidate search (multi-pass change). |
| Bedrock success arguments (`locate.rs`) | Already fixed for Bedrock player sources via `bedrock_success_args`. The remaining cross-cutting split needs `pumpkin-util/src/text/mod.rs`, outside this unit (out of scope). |

Files changed: `locate.rs`, `structure_finder.rs`.


## Result (final re-audit)

Re-verified every STILL OPEN row against the decompiled vanilla tree in
`.vendor/vanilla-decompiled/net/minecraft`. All in-scope, fixable rows are
already fixed in the current checkout by earlier passes; no further code
change was needed this pass, so no `.rs` file was touched and no `rustfmt`
run was required. Each row was checked against vanilla source, not memory.

| Row | Outcome |
|---|---|
| Stronghold blocking search (`locate.rs`) | Already matches vanilla Canvas (`LocateCommand.java:109-112`): the `StructureSearch::Pending` arm sends `Searching for <name>...` and returns 0. No change. |
| Datapack/modded structures via registry (`wanted_structures`) | STILL OPEN, out of scope: needs a dynamic `worldgen/structure` registry in generated `pumpkin-data`; `StructureKeys::from_name` cannot resolve custom ids (generated-data / cross-crate change). |
| POI argument type (`resource_or_tag.rs`) | Already fixed: `ResourceOrTagArgument::parse` rejects unknown POI ids via generated `pumpkin_data::poi_type::PoiType::from_name` (`:162-164`), covered by `poi_ids_are_validated_at_parse_time`; matches vanilla `resourceOrTag(POINT_OF_INTEREST_TYPE)`. |
| All placements per structure set (`structure_set_containing`) | Already fixed: `structure_sets_containing` returns every set index (vanilla `getPlacementsForStructure` returns a list); the executor scans each set; `all_sets_containing_a_structure_are_returned` test. |
| Random-spread nearest ordering (`structure_finder.rs`) | Already fixed: `find_nearest_structure` evaluates every placement at a radius, keeps the nearest hit, and advances only after the full ring (vanilla `ChunkGenerator.java:242-271`). |
| Stronghold distance ranking (`find_nearest_concentric`) | Already fixed: ring candidates are ranked with y=32 while the reported position stays y=0 (vanilla `ChunkGenerator.java:296-297`). |
| Structure arm placement candidates (`locate.rs:273-279`) | Verified the row's premise is invalid: vanilla `getStructureGeneratingAt` (`ChunkGenerator.java:349-362`) calls `checkStructurePresence` and only reports a candidate where a start can exist, so the current start-resolving code (`find_nearest_structure_start`) already matches. No change. |
| Bedrock success arguments (`locate.rs`) | Already fixed for player sources via `bedrock_success_args` plus `bedrock_success_args_match_bedrock_arity`. Remaining cross-cutting split needs `pumpkin-util/src/text/mod.rs`, outside this unit (out of scope). |

Open rows (both out of scope, one-line reason each):
- Datapack/modded structures: needs a dynamic structure registry in generated `pumpkin-data` (generated-data / cross-crate change).
- Bedrock success args for console/shared sources: needs a separate Java/Bedrock `with` list in `TextContent::Translate` (`pumpkin-util/src/text/mod.rs`), touching ~10 match sites across crates.

## Result

Continuation pass after the server restart. Re-verified every STILL OPEN row
against `.vendor/vanilla-decompiled/net/minecraft`, including
`ChunkGenerator.java`. All in-scope fixes are already present in the checkout;
this pass changed no `.rs` file, so no rustfmt run was needed.

| Row | Outcome |
|---|---|
| Stronghold blocking search (`locate.rs`) | Already matches Canvas: the `StructureSearch::Pending` arm sends "Searching for ..." and returns 0. No change. |
| Datapack/modded structures (`wanted_structures`) | OPEN: needs a dynamic `worldgen/structure` registry in generated `pumpkin-data`; `StructureKeys::from_name` cannot resolve custom ids. |
| POI argument type (`resource_or_tag.rs`) | Fixed: `PoiType::from_name` validation plus `poi_ids_are_validated_at_parse_time`. |
| All placements per structure set | Fixed: `structure_sets_containing` plus `all_sets_containing_a_structure_are_returned`. |
| Random-spread nearest ordering (`structure_finder.rs`) | Fixed: whole ring is evaluated before advancing (vanilla `ChunkGenerator.java:242-271`). |
| Stronghold distance ranking (`find_nearest_concentric`) | Fixed: candidates ranked at y=32, reported at y=0 (vanilla `ChunkGenerator.java:319-324`). |
| Structure arm placement candidates (`locate.rs`) | No change: vanilla `getStructureGeneratingAt` calls `checkStructurePresence` and rejects placement-only candidates (`ChunkGenerator.java:349-362`), so the start-resolving code already matches. The row's premise is invalid. |
| Bedrock success arguments (`locate.rs`) | Fixed for player sources via `bedrock_success_args` plus the arity test. Console/shared split needs `pumpkin-util/src/text/mod.rs`, outside this unit. |

Files changed: none (all fixes already present from earlier passes).

Open rows:
- Datapack/modded structures: needs a dynamic structure registry in generated `pumpkin-data` (generated-data / cross-crate change).
- Bedrock success args for console/shared sources: needs separate Java/Bedrock `with` lists in `TextContent::Translate` (`pumpkin-util/src/text/mod.rs`).


## Result (native-4 cross-file pass)
- Datapack/modded structures (`wanted_structures`): BLOCKED-DATA. Needs a dynamic `worldgen/structure` registry in generated pumpkin-data.


## Result (locate deep-audit continuation)

Re-verified every STILL OPEN row against the decompiled vanilla tree in
`.vendor/vanilla-decompiled/net/minecraft`
(`server/commands/LocateCommand.java`, `world/level/chunk/ChunkGenerator.java`,
`core/SectionPos.java`). One new parity fix applied this pass; no compile run
(workers do not compile, orchestrator batch-checks).

| Row | Outcome |
|---|---|
| Stronghold blocking search (`locate.rs`) | Already matches Canvas (`LocateCommand.java:109-112`): the `StructureSearch::Pending` arm sends "Searching for <name>..." and returns 0. No change. |
| Datapack/modded structures via registry (`wanted_structures`) | OPEN (out of scope): resolving custom `worldgen/structure` ids needs a dynamic registry in generated `pumpkin-data`; `StructureKeys::from_name` only knows vanilla ids (generated-data / cross-crate change). |
| POI argument type (`resource_or_tag.rs`) | Already fixed: `ResourceOrTagArgument::parse` rejects unknown POI ids via generated `pumpkin_data::poi_type::PoiType::from_name` (`:162-164`) plus `poi_ids_are_validated_at_parse_time`; matches vanilla `resourceOrTag(POINT_OF_INTEREST_TYPE)`. |
| All placements per structure set | Already fixed: `structure_sets_containing` returns every set index (vanilla `getPlacementsForStructure` returns a list); `all_sets_containing_a_structure_are_returned` test. |
| Random-spread nearest ordering (`structure_finder.rs`) | Already fixed: `find_nearest_structure` evaluates every placement at a radius, keeps the nearest, and advances only after the whole ring (vanilla `ChunkGenerator.java:242-271`). |
| Stronghold distance ranking (`find_nearest_concentric`) | FIXED (this pass): vanilla ranks ring candidates at the chunk CENTER `(chunk<<4)+8, y=32, (chunk<<4)+8` (`ChunkGenerator.java:296`, `SectionPos.sectionToBlockCoord(chunk,8)` = `sectionToBlockCoord` + 8 at `SectionPos.java:133-138`); Pumpkin ranked at chunk min x/z. Distance now uses `(bx + 8, 32, bz + 8)` while the reported position stays `(bx, 0, bz)` (`getLocatePos`). No pure seam for a unit test (needs a populated `GlobalStructureCache`). |
| Structure arm placement candidates (`locate.rs:273-279`) | No change: verified the premise is invalid. `findNearestMapStructure` -> `getNearestGeneratedStructure` -> `getStructureGeneratingAt` calls `checkStructurePresence` (`ChunkGenerator.java:349-362`), so only real starts are reported; the start-resolving code already matches. |
| Bedrock success arguments (`locate.rs`) | Already fixed: `send_success` uses `translate_cross_with` with `bedrock_success_args`, and the cross-cutting split now exists in `pumpkin-util/src/text/mod.rs` (`bedrock_with` field threaded through Java/Bedrock packets; `legacy.rs` reads the Java `with`). The tracker's "STILL OPEN" note is stale. |

Files changed this pass: `crates/pumpkin-world/src/generation/generator/structure_finder.rs`,
`docs/tracker/taudit-crates-pumpkin-core-src-co-e958f4.md`.

Open rows:
- Datapack/modded structures: needs a dynamic `worldgen/structure` registry in generated `pumpkin-data` (generated-data / cross-crate change).
