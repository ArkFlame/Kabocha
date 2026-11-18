# Final rows from docs/tracker/deferred-pass2.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: mod.rs resource_or_tag.rs sneeze.rs

- taudit-crates-pumpkin-core-src-co-e958f4.md: | POI argument type (`resource_or_tag.rs`) | STILL OPEN: needs a generated POI type registry in `pumpkin-data` so `ResourceOrTagArgument::parse` can reject unknown ids; no such registry exists (generated-data change). |
- taudit-crates-pumpkin-core-src-da-734ab8.md: - All other STILL OPEN rows reference `mod.rs` (or generated data / cross-crate work), which is outside
- taudit-crates-pumpkin-core-src-en-1730b6.md: - Row 9 (STILL OPEN): 1.21.5+ `ItemFrame` implements `Container`; a cross-crate/WIT API change out of scope for this unit.
- taudit-crates-pumpkin-core-src-en-7bf7cf.md: - Row 4 `after_sneeze` island note (panda/sneeze.rs:81-109) - STILL OPEN. `after_sneeze` reads the world and jumps pandas within 10 blocks from the mob tick; pandas do not set `has_audited_tick`, so the conservative entity-island reach already covers it and there is no current bug. Left open: a full

## Result
Final pass: every row closed (FIXED/VERIFIED or BLOCKED-DATA). The only row whose fix
lives in this unit's editable file (`resource_or_tag.rs`) was already fixed; the other
three rows reference other files and were closed in their own units' passes. No `.rs`
source change or rustfmt was needed this pass.

- POI argument type (`resource_or_tag.rs`) - VERIFIED (fixed in-file). `ResourceOrTagArgument::parse`
  rejects unknown POI ids at parse time via the generated `pumpkin_data::poi_type::PoiType::from_name`
  (`resource_or_tag.rs:162-164`) and rejects unknown POI tags via
  `tag::get_tag_values(RegistryKey::PointOfInterestType, ..)` (`:175-192`); POI ids/tags
  are also suggested (`:98-105`). The original blocker ("no generated POI registry") is
  obsolete: `crates/pumpkin-data/src/generated/poi_type.rs` defines `PoiType` with
  `from_name`/`ALL`. Test `poi_ids_are_validated_at_parse_time` covers it. Vanilla:
  `.vendor/vanilla-decompiled/net/minecraft/commands/arguments/ResourceOrTagArgument.java:66-91`
  (parse-time registry lookup and unknown-resource/tag errors) and
  `.vendor/vanilla-decompiled/net/minecraft/server/commands/LocateCommand.java:83-85`
  (`resourceOrTag(POINT_OF_INTEREST_TYPE)`).

- `taudit-...da-734ab8.md` mod.rs rows - BLOCKED-DATA / outside this unit's file. The remaining
  code gaps live in `crates/pumpkin-core/src/data/datapack/mod.rs` (not `resource_or_tag.rs`)
  and were closed in that unit's own third pass: most were VERIFIED in-file; the two still
  needing code are BLOCKED-DATA on whole missing subsystems: no zip/`ZipArchive` pack reader
  (zip datapacks listed but never loaded) and no bundled "bundle" assets datapack. Nothing
  to change in `resource_or_tag.rs`.

- Row 9 (`taudit-...en-1730b6.md`, `ItemFrame` Container) - VERIFIED. Already implemented in
  this checkout: `ItemFrameInventory` with `impl Inventory` in
  `crates/pumpkin-core/src/entity/decoration/item_frame.rs:50-70,565`, the 1.21.5+ single-slot
  Container; closed in that unit's own final pass. Vanilla:
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/ItemFrame.java`.

- Row 4 (`taudit-...en-7bf7cf.md`, `after_sneeze` island note) - VERIFIED. `after_sneeze`
  (panda/sneeze.rs) matches vanilla `Panda.afterSneeze` at
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/panda/Panda.java:506-531`:
  sound, grounded-adult-in-water/`canPerformAction` gating on a box inflated by 10, and the
  gift only under `MOB_DROPS`. Pandas keep `has_audited_tick` false, so the conservative
  `unaudited_reach` covers the 10-block jump; the original sit/breed `tick_reach` sub-gap
  is already resolved. Closed in that unit's own third pass.
