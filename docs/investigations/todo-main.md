# Audit of docs/TODO.md deferred rows

Summary: DONE=29 STILL-OPEN=6 STALE=4 BLOCKED=3 (42 rows: 16 BLOCKED-DATA + 26 L).

Audit scope: the two table sections in `docs/TODO.md` that carry Rust `file:line` citations (BLOCKED-DATA rows 1-16, L rows 1-26). The `## Issues` section is an issue-title mirror of `docs/LEDGER.md` with no Rust citations and was out of scope for this item-by-item code verification.

Verdict keys: DONE = cited gap is closed in current code/data; STILL-OPEN = gap remains; STALE = path/line moved or premise superseded, gap no longer maps cleanly; BLOCKED = missing data confirmed absent from `assets/` and `crates/pumpkin-data`.

## Row-by-row results

| # | Area | Claim in TODO.md | Verdict | Evidence file:line |
|---|---|---|---|---|
| BD-1 | `blocks/redstone/daylight_detector.rs` | Comparator/analog output flag; `assets/blocks.json` lacks `HAS_ANALOG_OUTPUT_SIGNAL` | DONE | `crates/pumpkin-core/src/block/blocks/redstone/daylight_detector.rs:64-68` implements `get_comparator_output` by reading the `power` state directly, so no block flag is needed. |
| BD-2 | `physics.rs:10-12` | `bubble_column` missing from `washed_away_by_fluids` | DONE | `crates/pumpkin-core/src/block/fluid/physics.rs:19-21` uses the generated tag. Vanilla `.vendor/vanilla-decompiled/net/minecraft/data/tags/VanillaBlockTagsProvider.java:1537+` does **not** add `BUBBLE_COLUMN` to `WASHED_AWAY_BY_FLUIDS`; `assets/datapack/data/minecraft/tags/block/washed_away_by_fluids.json` matches. No gap existed. |
| BD-3 | `shelf.rs` (absent) | Shelf shape and light occlusion; needs generated shape data for `#minecraft:wooden_shelves` | STALE | Files now exist: `crates/pumpkin-core/src/block/blocks/shelf.rs:1-30` (602 lines, `#[pumpkin_block_from_tag("minecraft:wooden_shelves")]`, uses generated `AcaciaShelfLikeProperties`) and `crates/pumpkin-core/src/block/entities/shelf.rs` (254 lines); tags at `assets/datapack/data/minecraft/tags/{block,item}/wooden_shelves.json`. |
| BD-4 | `pumpkin-world/.../sculk/spreader.rs:81-83` | Level spreader constants; needs decompiled constants | DONE | `crates/pumpkin-world/src/generation/feature/features/sculk/spreader.rs:81-83` has `new_level_spreader()` with explicit named constants (10, 4, 10, 5) plus `new_world_gen_spreader()`. |
| BD-5 | `living.rs:5504` | `bypasses_armor` tag as bitmask; needs tag generation | DONE | Tag generated at `crates/pumpkin-data/src/generated/tag.rs:20069`; used at `crates/pumpkin-core/src/entity/living.rs:4089`, helper `bypasses_armor_durability` at `:5729` with tests at `:5831`. Cited line 5504 now sits in `visibility_percent`. |
| BD-6 | `spawn.rs:20-22` | Halloween spawn constants; no decompiled reference | DONE | `crates/pumpkin-core/src/entity/mob/spawn.rs:18-27` defines `HALLOWEEN_HEAD_CHANCE = 0.25`, `JACK_O_LANTERN_CHANCE = 0.1` and `is_halloween()`, attributed to vanilla `SpecialDates.isHalloween`. |
| BD-7 | `zombie/mod.rs:564-578` | Fire-on-hit main-hand guard; needs decompiled `Zombie.doHurtTarget` | DONE | `crates/pumpkin-core/src/entity/mob/zombie/mod.rs:564-586` implements `do_hurt_target` with the `main_hand_empty` guard before setting the target on fire. |
| BD-8 | `generated/tracked_data.rs` | Armor stand flags not synced; `DATA_ARMOR_STAND_FLAGS` absent (index 14) | DONE | `crates/pumpkin-data/src/generated/tracked_data.rs:2791` exposes `armor_stand::DATA_CLIENT_FLAGS` (BYTE, id 15); `crates/pumpkin-core/src/entity/decoration/armor_stand.rs:248-252` syncs the bitmask through it. The cited field name/index no longer applies. |
| BD-9 | `armor_stand.rs:314-448` | `get_clicked_slot` hitboxes; needs decompiled `getClickedSlot` | DONE | `crates/pumpkin-core/src/entity/decoration/armor_stand.rs:379-381` implements `get_clicked_slot` with the small-scale division and per-slot hitbox helper, called at `:455`. |
| BD-10 | `minecart.rs:225` | Minecart portal skipped; needs a dimension-change path | DONE | `crates/pumpkin-core/src/entity/vehicle/minecart.rs:322-324` calls `self.vehicle.entity.tick_portal(caller)` in `tick` (defined at `crates/pumpkin-core/src/entity/mod.rs:4059`). |
| BD-11 | charged creeper `root` loot table | Drop table; no loot table JSON in `assets/` (count 0) | DONE | `assets/datapack/data/minecraft/loot_table/charged_creeper/root.json` exists; `crates/pumpkin-core/src/entity/mob/creeper.rs:382` loads `minecraft:charged_creeper/root` and generates head drops at `:382-403`. |
| BD-12 | charged creeper `root` loot table (second note) | Drop table; no loot table JSON in `assets/` | DONE | Same as BD-11: `assets/datapack/data/minecraft/loot_table/charged_creeper/root.json`, used at `crates/pumpkin-core/src/entity/mob/creeper.rs:382`. |
| BD-13 | `dolphin_food` tag | Dolphin food items; tag not extracted into `assets/` | BLOCKED | No `dolphin_food` item tag under `assets/datapack/data/minecraft/tags/item/`, and no `DOLPHIN_FOOD` constant in `crates/pumpkin-data`. Only unrelated dolphin files exist (`tags/fluid/dolphin_tries_to_find.json`, `loot_table/entities/dolphin.json`). |
| BD-14 | `dolphin_food` tag (second note) | Dolphin food items; tag not extracted into `assets/` | BLOCKED | Confirmed absent as for BD-13. |
| BD-15 | container-entity loot tables (1.21.5+) | Chest boat and similar drops; no loot table JSON in `assets/` | STALE | `assets/datapack/data/minecraft/loot_table/` now holds 1452 JSON files (including `entities/shulker.json`, `charged_creeper/root.json`, `chests/**`). The "count 0" premise is false; no `*_boat*` loot table was found, so the specific chest-boat wiring remains unclear. |
| BD-16 | `generated/tracked_data.rs:17973-17978` | `tadpole_variant` not generated; only `AGE_LOCKED` | BLOCKED | `crates/pumpkin-data/src/generated/tracked_data.rs:17973+` `pub mod tadpole` contains `AGE_LOCKED` (id 17) and the standardLiving fields only; no `VARIANT`/`TADPOLE_VARIANT` anywhere in the file. Data still needs the Extractor. |
| L-1 | `light` / block shape | Daylight detector occlusion is a full cube, not a 6px column | STILL-OPEN | No `use_shape_for_light_occlusion` / `USE_SHAPE_FOR_LIGHT_OCCLUSION` in `crates/` or `assets/`; `crates/pumpkin-core/src/world/brightness.rs` has no shape-based occlusion path. |
| L-2 | `shelf.rs` (absent) | Shelf shape verification; engine and codegen work | STALE | `crates/pumpkin-core/src/block/blocks/shelf.rs:1-30` and `crates/pumpkin-core/src/block/entities/shelf.rs` now exist with generated properties and tag registration; "absent" is false. |
| L-3 | `particle.rs:157-197` | Particle position source not encoded; no encoder helper | DONE | `crates/pumpkin-core/src/command/commands/particle.rs:114-117` (`block_position_source`) and `:237-245` encode the vibration block position source; test at `:672`. |
| L-4 | `pumpkin-protocol/.../particle.rs:280-320` | Particle packet layout; needs byte-exact tests | DONE | `crates/pumpkin-protocol/src/java/client/play/particle.rs:275-325` has the full versioned write path, `:333-386` the matching read path, and `mod tests` at `:414` with 10 byte-assertion tests. |
| L-5 | loot functions (`minecraft:set_*`) | Only `set_damage` and `set_count` implemented; rest skipped silently | DONE | `crates/pumpkin-core/src/world/loot/generation.rs:549-634` handles `SetCount`, `ApplyBonus`, `EnchantedCountIncrease`, `LimitCount`, `ExplosionDecay`, `FurnaceSmelt`, `SetStewEffect`, `CopyComponents`. Only an unknown-table warning exists (`:535`), no unknown-function warning. |
| L-6 | `locate.rs` (`wanted_structures`) | Structure lookup by name; needs a dynamic registry | STILL-OPEN | `crates/pumpkin-core/src/command/commands/locate.rs:123-139` still uses `StructureKeys::from_name(id.path())` gated by `id.is_vanilla()`; datapack-only structures are still unreachable. |
| L-7 | `mod.rs:2079`, `mod.rs:1384` (datapack loader) | Pack scanning; needs a read-only directory/zip filesystem abstraction | STILL-OPEN | `crates/pumpkin-core/src/data/datapack/mod.rs` now 3963 lines with scanning (`scan_datapacks_dir` at `:2425`, `load_pack_contents` at `:2391`) but only `fs::read_dir`/`fs::read_to_string` (`:1593`, `:2521`, `:2908`); no zip support. |
| L-8 | `types.rs:118-122` (piglin sensors) | Nearby and nearest-visible adult piglin/hoglin sensors + counts | STALE | Memory keys moved to `crates/pumpkin-core/src/entity/ai/brain/memory/types.rs:71-80`. `hoglin_specific.rs:12-63` populates `NEAREST_VISIBLE_ADULT_PIGLIN`, `NEAREST_VISIBLE_ADULT_HOGLINS` and both counts (tests at `:157-167`); `NEARBY_ADULT_PIGLINS` is declared at `types.rs:117` but has no populator. |
| L-9 | `lib.rs:599-636` (shutdown) | Player stats not saved on shutdown | DONE | `crates/pumpkin-core/src/lib.rs:634-654` calls `save_all_players` on shutdown; `crates/pumpkin-core/src/entity/player.rs:9282-9289` writes stats into player NBT. Format differs from vanilla `world/stats/<uuid>.json`, but the data is no longer lost. |
| L-10 | `player_auth_input.rs` vehicle branch | Vehicle rotation and paddling input | DONE | `crates/pumpkin-core/src/net/bedrock/play/player_auth_input.rs:122-129` sets paddle state; `:499-558` applies vehicle movement, `vehicle_rotation` yaw/pitch and rider repositioning. |
| L-11 | command tree (plugin reload) | Plugin command nodes stay after reload | DONE | `crates/pumpkin-command/src/node/tree.rs:430` (`remove_root_command`) and `crates/pumpkin-command/src/node/dispatcher.rs:176` (`remove_commands_from_source`), with tests at `:1200` and `:1229`. |
| L-12 | `ticker.rs:97-118` | Tick panic handling; needs crash report + stop path | STILL-OPEN | `crates/pumpkin-core/src/server/ticker.rs` has no `catch_unwind`/crash-report path (only `stop_token` checks at `:81`, `:194`, `:200`, `:281`); cited line range now holds overload-skip logic. |
| L-13 | `ticker.rs:62-139` | Paused server ticking; empty-tick counter + paused branch | DONE | `crates/pumpkin-core/src/server/ticker.rs:60-66` (`EmptyTickClock::observe`/`paused`) and `:105-140` (`pause_when_empty_seconds`, `is_empty`, `paused` branch skipping world ticks). |
| L-14 | entity radius collector (dragon) | Dragon parts skipped in hit detection | DONE | `crates/pumpkin-core/src/world/mod.rs:351` stores `dragon_parts`; `:6170-6177` merges them into the entity enumeration used by area/target queries. |
| L-15 | `beacon.rs:34-39`, `:322-353` | Beam sections | DONE | `crates/pumpkin-core/src/block/entities/beacon.rs:28-130` defines `BeamSection`, `BeamState` (`checking`/`sections`) and `append_beam_block` with colour averaging and opaque-block cuts. |
| L-16 | `block/entities/beehive.rs:1-123` | Bee nest occupants and server tick | DONE | `crates/pumpkin-core/src/block/entities/beehive.rs:73-74` `tick()` calls `tick_occupants`; `:160-235` adds occupants and releases them with `HONEY_DELIVERED` nectar reset; `MAX_OCCUPANTS` at `:135`. |
| L-17 | `entity/passive/bee.rs:63-72` | Bee goals, nectar and pollination | DONE | `crates/pumpkin-core/src/entity/passive/bee.rs:28-45` registers `PollinateFlowerGoal`, `GrowCropGoal`, `GoToKnownFlowerGoal`, hive goals; `:60-161` implements nectar/pollination/crop rules; goals added at `:245+`. |
| L-18 | `daylight_detector.rs` plus `world/brightness.rs` | Shape-based light occlusion (duplicate of L1) | STILL-OPEN | Same as L-1: no `use_shape_for_light_occlusion` flag or generated shape-occlusion path found in `crates/`/`assets/`. |
| L-19 | `block/entities/structure_block.rs:188-262`, `:358-461` | Structure block entities (capture and place) | DONE | `crates/pumpkin-core/src/block/entities/structure_block.rs:228-344` captures a template (palettes + block entities + entities) and `:368-381` saves/registers it. |
| L-20 | `structure_block.rs:207-281` | Capture of entities into template NBT | DONE | `crates/pumpkin-core/src/block/entities/structure_block.rs:305-341` runs the `fillEntityList` pass (excludes players, writes entity NBT into `StructureEntityInfo`). |
| L-21 | `structure_block.rs:377-472` | Placement of template entities | DONE | `crates/pumpkin-core/src/block/entities/structure_block.rs:461-515` `place_template_entities` (spawns roots + passengers, applies rotation); `:743-744` gates it on `ignore_entities`. |
| L-22 | goat AI | Ram attack activity | DONE | `crates/pumpkin-core/src/entity/passive/goat.rs:29-30` imports `RamTargetGoal`; `:99-100` registers it; goal in `crates/pumpkin-core/src/entity/ai/goal/ram_target.rs`. |
| L-23 | goat AI | Long jump activity | DONE | `crates/pumpkin-core/src/entity/passive/goat.rs:48-57` (screamer/normal jump sound) and `:101-105` registers `LongJumpGoal` with 600..1200 cooldown and 3.5714288x jump; goal in `crates/pumpkin-core/src/entity/ai/goal/long_jump.rs`. |
| L-24 | `happy_ghast.rs:158-163`, `:263-269` | Harness body slot and four-seat riding | DONE | `crates/pumpkin-core/src/entity/passive/happy_ghast.rs:108-123` (`is_wearing_harness`, `can_add_passenger` with `MAX_PASSENGERS = 4`), `:255-261` controlling passenger, `:419-443` mount path, test at `:577`. |
| L-25 | `strider.rs` constructor | Per-mob pathfinder malus overrides | STILL-OPEN | The pathfinder now has the API (`get/set_pathfinding_malus` in `crates/pumpkin-core/src/entity/ai/util/goal_utils.rs:55-63`, used by e.g. `follow_mob.rs:85-98`), but `crates/pumpkin-core/src/entity/passive/strider.rs:50-84` sets no malus override. |
| L-26 | item use on entity (`item/mod.rs:71`) | Success/pass result for use on entity | DONE | `crates/pumpkin-core/src/item/mod.rs:79-85` now returns `BlockActionResult` (default `Pass`), so the client result can be signalled; cited line moved from 71 to 79. |

## Rows whose TODO.md text should be corrected

- **BD-1** — The flag `HAS_ANALOG_OUTPUT_SIGNAL` is still absent from `assets/blocks.json`, but the comparator output is implemented directly in the block (`get_comparator_output`). Rewrite as DONE: "analog/comparator output implemented via stored `power` state, no Extractor needed."
- **BD-2** — Premise is wrong: vanilla does not put `bubble_column` in `washed_away_by_fluids`. Remove the row or mark it verified against `VanillaBlockTagsProvider`.
- **BD-3 / L-2** — "`shelf.rs` (absent)" is false; both `block/blocks/shelf.rs` and `block/entities/shelf.rs` exist. Update paths and drop the "absent" wording.
- **BD-4** — Constants are now present in `new_level_spreader()`; remove the "needs decompiled constants" reason.
- **BD-5** — Line `living.rs:5504` is stale; point to `living.rs:4089` / `:5729` and `pumpkin-data/src/generated/tag.rs:20069`.
- **BD-6** — "No decompiled reference in the repo" is stale; the constants and `is_halloween()` are at `spawn.rs:18-27`.
- **BD-7** — Guard now exists at `zombie/mod.rs:564-586`; update line range.
- **BD-8** — Field name/index is stale: the synced byte is `DATA_CLIENT_FLAGS` (id 15), not `DATA_ARMOR_STAND_FLAGS` (id 14).
- **BD-10** — Portal handling is no longer skipped; cite `minecart.rs:322-324`.
- **BD-11 / BD-12 / BD-15** — "No loot table JSON in `assets/` (count 0)" is false: 1452 loot tables now exist, including `charged_creeper/root.json`. BD-11/12 are DONE; BD-15 needs a narrower claim (which container entities still lack a drop table).
- **BD-16** — Keep as open but update the line reference: the `tadpole` module now starts at `tracked_data.rs:17973` and still has only `AGE_LOCKED`.
- **L-3** — "No encoder helper exists yet" is false; `block_position_source` and the vibration encoding are in `command/commands/particle.rs:114-117`, `:237-245`.
- **L-4** — Cite `pumpkin-protocol/.../particle.rs:414` for the 10 existing byte-assertion tests; the "needs tests" reason is stale.
- **L-5** — "Only `set_damage` and `set_count`" is false; eight `LootFunctionKind` variants are handled in `world/loot/generation.rs:549-634`.
- **L-8** — Lines `118-122` moved to `types.rs:71-80`; note that only `NEARBY_ADULT_PIGLINS` remains unpopulated.
- **L-9** — Stats are saved, but in player NBT rather than `world/stats/<uuid>.json`; correct the expected mechanism.
- **L-11** — Node removal now exists (`node/tree.rs:430`, `node/dispatcher.rs:176`).
- **L-12** — Cited lines `97-118` now hold overload-skip logic; the panic-handling gap has no fixed location yet.
- **L-13** — Line range `62-139` is still close, but the relevant items are `EmptyTickClock` (`:60-66`) and the `paused` branch (`:105-140`).
- **L-18** — Mark as an explicit duplicate of L-1, both still open.
- **L-25** — Reword from "needs malus overrides in the pathfinder" (now present) to "strider does not set its per-mob malus overrides."
- **L-26** — Line 71 moved to 79 and the trait no longer returns `()`; it returns `BlockActionResult`.

## Still-open work (no code change made)

- BD-13/BD-14: `minecraft:dolphin_food` item tag is not extracted into `assets/` or generated data.
- BD-16: `tadpole_variant` tracked data is not generated.
- L-1/L-18: shape-based light occlusion for the daylight detector (6px column vs full cube).
- L-6: `/locate` lookup for datapack-defined structures.
- L-7: zip-pack support in the datapack loader (directory scanning works).
- L-12: tick panic crash-report/stop path.
- L-25: strider pathfinding malus overrides.

Status: PASS done=29 open=6 stale=4 blocked=3 file=docs/investigations/todo-main.md
