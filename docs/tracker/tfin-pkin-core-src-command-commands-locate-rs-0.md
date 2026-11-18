# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-command-commands-locate-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

All rows below are terminal. The decompiled vanilla tree named in the task is not present under `.vendor/` in this checkout; the equivalent tree at `/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft` was used for every citation below (`server/commands/LocateCommand.java`, `world/level/chunk/ChunkGenerator.java`, `core/SectionPos.java`).

## Result
FINAL PASS. Re-verified every row against decompiled vanilla. Every in-scope fix is already present in the checkout (`locate.rs`, `structure_finder.rs`, `resource_or_tag.rs`, `pumpkin-util/src/text/mod.rs`) and type-consistent, so this pass needed no source edit and no rustfmt run.

| Row | Terminal outcome |
|---|---|
| Stronghold blocking search (`locate.rs`) | VERIFIED: the `StructureSearch::Pending` arm sends "Searching for <name>..." and returns 0 (`locate.rs:285`), matching `server/commands/LocateCommand.java:110`. |
| Datapack/modded structures (`wanted_structures`) | BLOCKED-DATA: whole missing subsystem. Vanilla resolves ids through the dynamic `Registries.STRUCTURE` (`LocateCommand.java:90-99`, `:97`), so datapack/modded ids are locatable; Pumpkin only has the closed generated enum `pumpkin_data::structures::StructureKeys::from_name` plus an `is_vanilla()` gate (`locate.rs:123-126`), and the finders index generated `StructureSet::ALL` constants. Missing: a generated `worldgen/structure` / `worldgen/structure_set` gameplay registry in `crates/pumpkin-data/src/generated/` (not worker-editable) plus wiring into `find_nearest_structure_start`/`structure_sets_containing`. The datapack loader only reads those entries for client sync (`data/datapack/dynamic_registry_loader.rs:101-108`); no gameplay reader consumes them. |
| POI argument type (`resource_or_tag.rs`) | VERIFIED: `ResourceOrTagArgument::parse` rejects unknown POI ids via generated `pumpkin_data::poi_type::PoiType::from_name` (`resource_or_tag.rs:164`), matching `resourceOrTag(Registries.POINT_OF_INTEREST_TYPE)` (`LocateCommand.java:83-84`); test `poi_ids_are_validated_at_parse_time`. |
| All placements per structure set (`structure_sets_containing`) | FIXED/VERIFIED: returns every set index (`locate.rs:147-154`) and the executor scans each; test `all_sets_containing_a_structure_are_returned`. Matches `getPlacementsForStructure` returning a list (`ChunkGenerator.java:205`). |
| Random-spread nearest ordering (`structure_finder.rs`) | FIXED/VERIFIED: the whole placement ring is evaluated before advancing and the nearest hit kept (`structure_finder.rs:66`); matches `ChunkGenerator.java:242-271`. |
| Stronghold distance ranking (`find_nearest_concentric`) | FIXED/VERIFIED: ring candidates ranked at chunk center `(bx + 8, 32, bz + 8)` while the reported position stays `(bx, 0, bz)` (`structure_finder.rs:320-326`); matches `ChunkGenerator.java:296` and `core/SectionPos.java:133-138`. |
| Structure arm placement candidates (`locate.rs:273-279`) | VERIFIED (premise invalid): `findNearestMapStructure` -> `getStructureGeneratingAt` -> `checkStructurePresence` (`ChunkGenerator.java:349-362`) only reports real starts, so the start-resolving code already matches; no change. |
| Bedrock success arguments (`locate.rs`) | VERIFIED: `send_success` (`locate.rs:183`) uses `translate_cross_with` with `bedrock_success_args` (`locate.rs:209`); Bedrock arity (name,x,z,distance / name,x,y,z,distance) matches the edition-specific messages. The cross-cutting Java/Bedrock `with` split exists in `pumpkin-util/src/text/mod.rs` (`bedrock_with` field); tests `bedrock_success_args_match_bedrock_arity` and the text split test. |

Files changed this pass: none (all fixes already present and verified). Remaining terminal blocker: the single datapack-structure row, BLOCKED-DATA (generated `worldgen/structure` registry, not editable by workers).
