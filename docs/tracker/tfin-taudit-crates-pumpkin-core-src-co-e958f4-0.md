# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-co-e958f4.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: legacy.rs locate.rs mod.rs

| Bedrock success arguments (`locate.rs`) | VERIFIED: `send_success` uses `translate_cross_with` with `bedrock_success_args` (`locate.rs:183-223`). The cross-cutting split exists in `pumpkin-util/src/text/mod.rs` (`bedrock_with` field threaded at :1055-1065, :1127-1130, :1254-1267, :1459-1466; test :2286) and is read in `legacy.rs:273`. The tracker's "STILL OPEN" note is stale. |

## Result

FINAL PASS: every row is terminal. Re-verified all eight rows from the source
tracker against the decompiled vanilla tree in
`.vendor/vanilla-decompiled/net/minecraft`. No `.rs` source file needed a new
edit this pass: every fixable row is already present and type-consistent in the
checkout, and the one remaining row needs a whole missing subsystem (a generated
registry plus a gameplay reader), so it is terminal BLOCKED-DATA. No rustfmt run
was required.

| Row | Terminal outcome |
|---|---|
| Stronghold blocking search (`locate.rs`) | VERIFIED: the `StructureSearch::Pending` arm sends `Searching for <name>...` and returns 0 (`locate.rs:285-294`); vanilla sends the same literal at `.vendor/vanilla-decompiled/net/minecraft/server/commands/LocateCommand.java:109-112`. |
| Datapack/modded structures via registry (`wanted_structures`) | BLOCKED-DATA: whole missing subsystem. Vanilla resolves `/locate structure` through the dynamic `Registries.STRUCTURE` so datapack/modded ids are locatable (`LocateCommand.java:90-99`). Pumpkin can only name vanilla structures: `wanted_structures` (`locate.rs:123-139`) resolves through the closed generated enum `pumpkin_data::structures::StructureKeys::from_name` (`crates/pumpkin-data/src/generated/structures.rs:9-64`) and gates on `is_vanilla()`, and `structure_sets_containing` (`locate.rs:147-154`) indexes generated `StructureSet::ALL` consts. The datapack loader loads raw `minecraft:worldgen/structure` / `worldgen/structure_set` NBT only for client sync (`crates/pumpkin-core/src/data/datapack/dynamic_registry_loader.rs:101-108`); the dedicated gameplay loaders cover only `template_pool` / `processor_list` (`dynamic_registry_loader.rs:120`, `structure_loader.rs`) and no gameplay reader turns those registry entries into searchable placements (zero matches). Missing subsystem: a dynamic worldgen structure/structure_set gameplay registry that parses the loaded entries into placements, wired into `wanted_structures` / `structure_sets_containing` / the finders, plus regenerated non-closed structure data. |
| POI argument type (`resource_or_tag.rs`) | VERIFIED: `ResourceOrTagArgument::parse` rejects unknown POI ids via generated `pumpkin_data::poi_type::PoiType::from_name` (`crates/pumpkin-command/src/argument_types/resource_or_tag.rs:162-164`), matching vanilla `resourceOrTag(Registries.POINT_OF_INTEREST_TYPE)` (`LocateCommand.java:83-85`); test `poi_ids_are_validated_at_parse_time`. |
| All placements per structure set (`structure_sets_containing`) | FIXED: returns every set index (`locate.rs:147-154`) and the executor scans each; `all_sets_containing_a_structure_are_returned`. Matches vanilla `ChunkGeneratorStructureState.getPlacementsForStructure` returning a list. |
| Random-spread nearest ordering (`structure_finder.rs`) | FIXED: the whole placement ring is evaluated before advancing, nearest hit kept (`crates/pumpkin-world/src/generation/generator/structure_finder.rs:81-109`); vanilla evaluates every placement at radius r and returns at the first radius with any hit (`.vendor/vanilla-decompiled/net/minecraft/world/level/chunk/ChunkGenerator.java:242-271`). |
| Stronghold distance ranking (`find_nearest_concentric`) | FIXED/VERIFIED: ring candidates ranked at chunk center `(bx+8, 32, bz+8)` while the reported position stays `(bx, 0, bz)` (`structure_finder.rs:316-333`); vanilla ranks at `SectionPos.sectionToBlockCoord(chunk,8), 32` (`ChunkGenerator.java:294-305`, `core/SectionPos.java:133-138`). |
| Structure arm placement candidates (`locate.rs:273-279`) | VERIFIED (premise invalid): vanilla `findNearestMapStructure` -> `getStructureGeneratingAt` calls `checkStructurePresence` and only reports real starts (`.vendor/vanilla-decompiled/net/minecraft/world/level/chunk/ChunkGenerator.java:349-362`), so the start-resolving code (`find_nearest_structure_start`) already matches; no change. |
| Bedrock success arguments (`locate.rs`) | VERIFIED: `send_success` uses `translate_cross_with` with `bedrock_success_args` (`locate.rs:183-223`); Java arg list (name, coordinates, distance) matches vanilla `showLocateResult` (`LocateCommand.java:206`), Bedrock lists are edition-specific (structure name,x,z,distance / biome name,x,y,z,distance). `bedrock_with` is threaded through every `TextContent::Translate` path (`crates/pumpkin-util/src/text/mod.rs:1127-1130,1254-1267,1459-1466`; test :2286), Java legacy reads `with` (`crates/pumpkin-util/src/text/legacy.rs:269-280`), test `bedrock_success_args_match_bedrock_arity` (`locate.rs:515-519`). |

Files changed this pass: none (all fixes already present and verified).
Remaining terminal blocker: the one datapack-structure row, BLOCKED-DATA
(dynamic worldgen structure/structure_set gameplay registry plus regenerated
non-closed structure data).
