# TODO (deferred items only)

Every upstream PR and issue is DONE or SKIPPED (`docs/LEDGER.md`), so nothing upstream is pending. What remains is the audit gaps in `docs/tracker/` that could not be closed in this pass. Each row says why it is deferred.

Kinds:
- **BLOCKED-DATA**: the fix needs data that is not in `assets/` or `pumpkin-data` (generated tables, tags, loot tables, decompiled constants). Unblock by extending the Extractor or by decompiling the reference, then regenerate with `cargo run --locked -p pumpkin-codegen`.
- **L**: large in the audit's own size scale. Each needs an engine, protocol or API change, or a new subsystem, and its own pass.

Counts: 16 BLOCKED-DATA rows, 26 L rows, 42 in total.

## BLOCKED-DATA (16)

| # | Area (Rust location) | Gap | Reason deferred |
|---|---|---|---|
| 1 | `blocks/redstone/daylight_detector.rs` | Comparator/analog output flag | `assets/blocks.json` lacks `HAS_ANALOG_OUTPUT_SIGNAL` for daylight detector; needs Extractor. |
| 2 | `physics.rs:10-12` | `bubble_column` missing from `washed_away_by_fluids` | Needs the vanilla fluid tag data; the audit note asks to confirm it first. |
| 3 | `shelf.rs` (absent) | Shelf shape and light occlusion | Needs generated shape data for `#minecraft:wooden_shelves`. |
| 4 | `pumpkin-world/.../sculk/spreader.rs:81-83` | Level spreader constants | Needs decompiled vanilla constants. |
| 5 | `living.rs:5504` | `bypasses_armor` tag as bitmask | Needs data tag generation. |
| 6 | `spawn.rs:20-22` | Halloween spawn constants | No decompiled Zombie or AbstractSkeleton reference in the repo. |
| 7 | `zombie/mod.rs:564-578` | Fire-on-hit main-hand guard | Needs decompiled 1.21 `Zombie.doHurtTarget`. |
| 8 | `generated/tracked_data.rs` | Armor stand flags not synced | `DATA_ARMOR_STAND_FLAGS` is absent from generated tracked data (index 14). |
| 9 | `armor_stand.rs:314-448` | `get_clicked_slot` hitboxes | Needs decompiled `getClickedSlot`. |
| 10 | `minecart.rs:225` | Minecart portal (skipped) | Needs an entity dimension change path. |
| 11 | charged creeper `root` loot table | Drop table | No loot table JSON in `assets/` (count 0). |
| 12 | charged creeper `root` loot table | Drop table (second note) | No loot table JSON in `assets/`. |
| 13 | `dolphin_food` tag | Dolphin food items | Tag not extracted into `assets/`. |
| 14 | `dolphin_food` tag | Dolphin food items (second note) | Tag not extracted into `assets/`. |
| 15 | container-entity loot tables (1.21.5+) | Chest boat and similar drops | No loot table JSON in `assets/`. |
| 16 | `generated/tracked_data.rs:17973-17978` | `tadpole_variant` not generated | Only `AGE_LOCKED` is generated; needs Extractor. |

## L (26)

| # | Area (Rust location) | Gap | Reason deferred |
|---|---|---|---|
| 1 | `light` / block shape | Daylight detector light occlusion is a full cube, not a 6px column | Needs shape-driven light occlusion in the light engine. |
| 2 | `shelf.rs` (absent) | Shelf shape verification | Engine and codegen work. |
| 3 | `particle.rs:157-197` | Particle position source not encoded | No encoder helper exists yet. |
| 4 | `pumpkin-protocol/.../particle.rs:280-320` | Particle packet layout | Needs a protocol audit with byte-exact tests against a vanilla packet dump. |
| 5 | loot functions (`minecraft:set_*`) | Only `set_damage` and `set_count` are implemented; the rest are skipped silently | Needs a loot function set and a warning path for unknown functions. |
| 6 | `locate.rs` (`wanted_structures`) | Structure lookup by name | Needs a dynamic structure registry instead of `StructureKeys::from_name`. |
| 7 | `mod.rs:2079`, `mod.rs:1384` (datapack loader) | Pack scanning | Needs a read-only pack filesystem abstraction (directory or zip) threaded through `load_pack_contents`. |
| 8 | `types.rs:118-122` (piglin sensors) | Nearby and nearest-visible adult piglin and hoglin sensors | The two species sensors must be ported together with their counts. |
| 9 | `lib.rs:599-636` (shutdown) | Player stats not saved on shutdown | Needs a stats manager writing `world/stats/<uuid>.json`. |
| 10 | `player_auth_input.rs` vehicle branch | Vehicle rotation and paddling input | Needs input routing into `entity/vehicle/boat.rs` and the minecart input path. |
| 11 | command tree (plugin reload) | Plugin command nodes stay after reload | Needs node removal in `Tree` and `CommandDispatcher`, including redirects and aliases. |
| 12 | `ticker.rs:97-118` | Tick panic handling | Needs a crash report and the existing stop path. |
| 13 | `ticker.rs:62-139` | Paused server ticking | Needs the empty-tick counter and the paused branch. |
| 14 | entity radius collector (dragon) | Dragon parts skipped in hit detection | Needs dragon parts in the radius collector. |
| 15 | `beacon.rs:34-39`, `beacon.rs:322-353` | Beam sections | Needs section tracking and a client-facing beam representation. |
| 16 | `block/entities/beehive.rs:1-123` | Bee nest occupants and server tick | Needs the occupant API, `tickOccupants` and bee release. |
| 17 | `entity/passive/bee.rs:63-72` | Bee goals, nectar and pollination | Needs the missing goals and the pollination tick. |
| 18 | `daylight_detector.rs` plus `world/brightness.rs` | Shape-based light occlusion (duplicate of L1) | Needs a generated `use_shape_for_light_occlusion` flag. |
| 19 | `block/entities/structure_block.rs:188-262`, `:358-461` | Structure block entities (capture and place) | Needs an entity pass in capture and placement. |
| 20 | `structure_block.rs:207-281` | Capture of entities into template NBT | Needs an entity pass in `capture_structure_template_from_world`. |
| 21 | `structure_block.rs:377-472` | Placement of template entities | Needs spawning honouring `ignore_entities`. |
| 22 | goat AI | Ram attack activity | Needs `PrepareRamNearestTarget` and `RamTarget` with their timings and speeds. |
| 23 | goat AI | Long jump activity | Needs the cooldown, height and velocity rules and the pose dimensions. |
| 24 | `happy_ghast.rs:158-163`, `:263-269` | Harness body slot and four-seat riding | No harness right-click path, seat layout or steering. |
| 25 | `strider.rs` constructor | Per-mob pathfinder malus overrides | Needs malus overrides in the pathfinder. |
| 26 | item use on entity (`item/mod.rs:71`) | Success or pass result for use on entity | The trait returns `()`, so the client result cannot be signalled; the item is still consumed correctly. |
