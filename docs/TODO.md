# TODO (deferred items only)

## Final verdict blockers (2026-11-19 final pass)

Final verdicts over 1369 tracker ids: FIXED 51, ALREADY-FIXED 826, WONTFIX 194, BLOCKED 298. Blocked ids are grouped below by named reason. Duplicates of a blocked primary and tracker-text defects are not listed here (see `docs/DONE.md`).

- **BedRule fields (6 ids)**: generated `BedRule` (`crates/pumpkin-data/src/generated/dimension.rs:14-18`) lacks `destroy_on_use` and `error_message`. Needs the `assets` dimension_type `gameplay/bed_rule` extraction, then `pumpkin-codegen`.
- **StructureKeys::from_name (1 id)**: no `from_name` in the generated `StructureKeys`. Needs codegen of the lookup.
- **WIT bucket event fields (11 ids)**: the bucket event payload needs additive WIT fields, host/SDK/runtime updates and `cargo run --locked -p pumpkin-codegen -- wit`.
- **Bedrock climate_variant and runtime checks (24 ids)**: Bedrock is separate and incomplete (`AGENTS.md`); needs Bedrock protocol data and runtime checks.
- **Generic inside-block sweep for all entities, en-f12c04-0#3 (1 id)**: needs an Entity-level movement ring (`Entity.java` `addMovementThisTick` / `applyEffectsFromBlocks`, 1521-1580) hooked into every move path, plus about 15 `tick_block_collisions` callers.
- **Cushion thunder_hit lightning drop (1 row, PARITY `entities/Cushion`)**: no entity `thunder_hit` dispatch exists in `crates/pumpkin-core/src/entity` (`docs/tracker/tentities-cushion.md` BLOCKED-DATA).
- **Missing subsystems (about 130 ids)**: engine-level block moveability for piston push, advancement and entity predicate subsystems, dimension and level registration for command suggestions, zip pack reader crate, registry element codec normalization (`da-4f0dec`), signed chat chain (`ne-61496d-0#1`), live-server gametests for suggestions. Each needs its own pass.


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

- Minecart stopped powered-rail nudge (0.02 toward a redstone conductor, vanilla `MinecartBehavior`): worker branch kab/f4-veh 43ca3b501 conflicts with the rail friction already integrated; redo on top of current minecart.rs. Also open: OCEAN_FLOOR heightmap (chunk NBT), light dampening `mergedFaceOccludes`, driver move-intent vector (see docs/tracker/tverify-vehicles.md).
## Issues
| # | title | labels |
|---|---|---|
| #4598 | Worldborder transition broken | bug,protocol,Java Edition |
| #4596 | Frozen ocean icebergs turn into grass/dirt in strips of chunks | bug,Java Edition,worldgen |
| #4593 | stop and SIGTERM never return while a v0.2 plugin keeps an endless spawn_local future | bug,api,panic,plugin |
| #4592 | WASM dynamic server-side tab completions throw the server into a panic and crash | bug,panic,command,plugin |
| #4583 | Server deadlocks under load when a plugin handles PacketSentEvent | bug,panic,performance,plugin |
| #4582 | Java 26.3 client disconnected ~3s after join: failed to decode clientbound level_particles (IndexOut | bug,protocol,Java Edition,particles |
| #4580 | Mobs do not teleport inside of boats if near. | bug,Java Edition,mobai,entity |
| #4579 | failure to bind to assigned port | bug,Java Edition,config |
| #4560 | Tracking: Items | enhancement,item |
| #4559 | Tracking: Blocks | enhancement,block |
| #4558 | At an extremely long render distance (64 chunks), the server may stop sending data packets. | bug,protocol,Java Edition,performance |
| #4550 | When clicking on mob that has variants with spawn eggs, child is default variant instead of similar  | bug,Java Edition,mobai,entity |
| #4547 | Server never checks if the block rotation and position is possible from the players perspective. | bug,protocol,block,security |
| #4546 | Clicking on a fence with lead causes time out of player connection. Server doesn't crash | bug,Java Edition,player,entity |
| #4545 | Lead doesn't drop after attached mob is killed | bug,good first issue,Java Edition,item,entity |
| #4537 | Shulker box doesn't check for blocks on top | bug,good first issue,Java Edition,block,inventory |
| #4529 | Book Ban Exploit fix? | enhancement,inventory,security,data |
| #4528 | Shuker crafting grid bug | bug,Java Edition,inventory,recipe |
| #4524 | tree generates with dirt blocks as leaves | bug,worldgen,block |
| #4522 | Painting variant metadata is sent without the Holder +1 offset | bug,good first issue,protocol,Java Edition,entity |
| #4521 | Expose transfer queue results to plugins | enhancement,api,player |
| #4518 | Are there plans to support Alpine Linux? | question,docker |
| #4515 | Players can equip nautilus armor in their own armor slots | bug,Java Edition,item,player |
| #4513 | Client shows wrong max health for horses (26 hearts) after spawn | bug,protocol,Java Edition,entity |
| #4498 | crafted items disappear from inventory | bug,Bedrock Edition,item,inventory |
| #4496 | Furnace does not award experience after smelting when the smelted product is extracted | bug,block,player |
| #4492 | Naturally spawned skeletons have no bow, and skeleton arrows deal no damage | bug,mobai,combat,entity |
| #4487 | [Vanilla Parity 26.3] Missing and outdated mechanics in `ArrowEntity` (`arrow.rs`) | bug,combat,entity,data |
| #4485 | Worldspawn Always set to ground, regardless of highest Y-level | bug,player,world |
| #4484 | Gamerule immediate_respawn non-functioning | bug,good first issue,Java Edition,command,player |
| #4483 | Completing a villager trade deadlocks the whole server (on_trade relocks a mutex held by on_slot_cli | bug,panic,inventory,entity |
| #4478 | After the flame is extinguished, it reappears out of thin air. | bug,block |
| #4477 | The sword's sweeping effect appeared in the wrong location. | bug,good first issue,Java Edition,combat |
| #4475 | Anvil chunk serializer drops 'structures' compound (starts and References) | bug,world,data |
| #4474 | Chunk read errors silently trigger regeneration and overwrite existing disk chunks | bug,panic,world,data |
| #4473 | [WASM v0.3] Improve Wasm Plugin API versioning | enhancement,api |
| #4472 | [WASM v0.2] Publish the v0.1 to v0.2 plugin migration guide | documentation,good first issue,api |
| #4471 | [WASM v0.2] Deprecate Wasm Plugin API v0.1 | enhancement,api |
| #4470 | [WASM v0.2] Enable Wasm Plugin API v0.2 by default | enhancement,api |
| #4466 | Container block entities send full inventory in CBlockEntityData on chunk load, crashing Java client | bug,protocol,Java Edition,inventory |
| #4465 | Enchantments serialization missing +1 registry ID offset, causing client disconnects (Invalid tag id | bug,protocol,Java Edition,enchantments |
| #4463 | systemctl Unit File | documentation,good first issue,question |
| #4458 | stone cutter bug | bug,block,inventory |
| #4457 | smithing table bug | bug,Java Edition,item,inventory |
| #4456 | riptide animation bug | bug,Java Edition,combat,enchantments |
| #4455 | wither dispawns bug | bug,Java Edition,mobai,entity |
| #4454 | lunge enchantment doesnt work on the spear | bug,Java Edition,combat,enchantments |
| #4452 | copper golem crash | bug,protocol,panic,entity |
| #4451 | The way the fire is spreading doesn't seem quite right. | bug,Java Edition,block |
| #4450 | The ignition radius of the lava is too large. | bug,Java Edition,block,fluid |
| #4447 | Copying vanilla server data leaves packet error. | bug,protocol,Java Edition,data |
| #4423 | fix(entity): preserve pending knockback on player-controlled mounts | bug,player,entity |
| #4422 | Process fall distance and landing for client-controlled mounts | bug,player,entity |
| #4421 | Fix movement for Bedrock-controlled saddled mounts | bug,Bedrock Edition,player,entity |
| #4420 | Fix wolf armor shearing advancement trigger | bug,good first issue,entity,advancements |
| #4411 | tty hangs up after using 'stop' in "Starting save" | bug,command,world,data |
| #4406 | Time is reset on server start | bug,world,data |
| #4398 | Advancements granted without conditions: effects_changed ignores conditions, location predicate igno | bug,player,advancements |
| #4388 | Bundle does not work as it should. | bug,Java Edition,item,inventory |
| #4386 | Lava does not burn boats | bug,Java Edition,entity,fluid |
| #4383 | Picking up excess items in creative mode creates "ghost items" | bug,Java Edition,item,player |
| #4367 | /worldborder center error shows a raw %s | bug,good first issue,localization,command |
| #4366 | /scoreboard objectives setdisplay/modify/list and players list missing | bug,command |
| #4365 | Scoreboard, bossbars, command storage and world border lost on restart | bug,world,data |
| #4364 | Movement is not validated (speed and flying) | bug,protocol,player |
| #4362 | /effect give and /effect clear reject non-player entities | bug,good first issue,command,entity |
| #4361 | New teams have friendlyFire and seeFriendlyInvisibles off | bug,good first issue,command |
| #4360 | /data modify ... insert <index> is unusable | bug,good first issue,command,data |
| #4359 | New world saves SpawnY=200, console commands run at y=200 | bug,command,world |
| #4358 | Console /say and /msg are not written to the server log | bug,good first issue,Java Edition,command |
| #4357 | /setblock and /fill report success when no block changes | bug,command,block |
| #4356 | /gamerule accepts out-of-range integer values (e.g. negative) | bug,good first issue,command,config |
| #4355 | /xp add with a negative amount: negative level and wrong message | bug,good first issue,command,player |
| #4354 | /spreadplayers with respectTeams puts all teams on the same spot | bug,good first issue,command,player |
| #4353 | Entity NBT: NoGravity/Silent/Glowing ignored and /data remove has no effect | bug,command,entity,data |
| #4352 | /fill with destroy mode deadlocks the server | bug,panic,command,block |
| #4348 | Older chucks not loading | bug,world,data |
| #4345 | Infinite "Loading Terrain" loading screen | bug,Java Edition,panic,player |
| #4343 | Hopper does not pull items from a chest minecart above it | bug,Java Edition,inventory,entity |
| #4342 | Hoppers do not insert items into composters | bug,block,inventory |
| #4340 | Side hopper puts non-fuel items into furnace input slot | bug,block,inventory |
| #4339 | Hopper pulls input and fuel from furnace above | bug,block,inventory |
| #4336 | Entity 'cat' being called causes a crash client side | bug,protocol,Java Edition,entity |
| #4331 | `PlayerSpawnLocationEvent` is not fired on join, so plugins cannot choose where a joining player spa | enhancement,Java Edition,api,player |
| #4328 | Server never checks if the block placment is blocked | bug,block,security |
| #4321 | Bedrock block pick (middle click) is always ignored | bug,good first issue,protocol,Bedrock Edition,inventory |
| #4320 | Ghost totem | bug,Java Edition,combat,player |
| #4317 | Plugin `send-packet` drops `c-player-info-update`, so plugins cannot add tab list entries | bug,protocol,api,player |
| #4315 | A plugin's per-player scoreboard objective reaches a joining client twice, which disconnects it | bug,protocol,api,player |
| #4314 | Natural mob spawning fires no `CreatureSpawnEvent` (feature request) | enhancement,api,mobai,entity |
| #4313 | `player.set-player-weather` has no effect | bug,protocol,player |
| #4312 | Cancelling `EntityTargetEvent` does not stop the mob from targeting | bug,api,mobai,entity |
| #4310 | A command node cannot require a permission without a callback into the plugin (feature request) | enhancement,api,command,permissions |
| #4309 | The Java command tree only hides top-level commands a player cannot use | bug,Java Edition,command,permissions |
| #4300 | Plugin `send-packet` silently drops packets the generated serializer does not know (objectives, scor | bug,protocol,api |
| #4299 | `text-component.from-json` rejects vanilla JSON (`show_text` with one component, `shadow_color` as a | bug,api |
| #4297 | A plugin cannot register a handler for `add-custom-ai-goal` | bug,api,mobai |
| #4296 | Splash potion damage fires no `entity-damage-by-entity` event (attacker lost) | bug,api,combat |
| #4295 | `PlayerLeaveEvent` has no leave reason (feature request) | enhancement,api,player |
| #4294 | `entity.get-nearby-entities` never includes players | bug,good first issue,api,player,entity |
| #4292 | A plugin that trapped keeps logging an error for every event and task run | bug,api |
| #4290 | [BUG] Crafter's disabled slots reset when the player opens the UI. | bug,Java Edition,block,redstone |
| #4288 | Chest changes are lost when the last player leaves (item duplication) | bug,inventory,player,data |
| #4283 | Problems with Mouse middle click | bug,Java Edition,item,inventory |
| #4281 | Issues with pistons | bug,redstone |
| #4280 | [TRACKER] Mob Vanilla Parity: Wither | good first issue,mobai,entity |
| #4279 | [TRACKER] Mob Vanilla Parity: Ender Dragon (Jean?) | good first issue,mobai,entity |
| #4277 | [TRACKER] Mob Vanilla Parity: Zolign | good first issue,mobai,entity |
| #4275 | [TRACKER] Mob Vanilla Parity: Wither Skeleton | good first issue,mobai,entity |
| #4274 | [TRACKER] Mob Vanilla Parity: Witch | good first issue,mobai,entity |
| #4273 | [TRACKER] Mob Vanilla Parity: Warden | good first issue,mobai,entity |
| #4272 | [TRACKER] Mob Vanilla Parity: Vindicator | good first issue,mobai,entity |
| #4271 | [TRACKER] Mob Vanilla Parity: Vex | good first issue,mobai,entity |
| #4270 | [TRACKER] Mob Vanilla Parity: Stray | good first issue,mobai,entity |
| #4269 | [TRACKER] Mob Vanilla Parity: Silverfish | good first issue,mobai,entity |
| #4268 | [TRACKER] Mob Vanilla Parity: Slime | good first issue,mobai,entity |
| #4267 | [TRACKER] Mob Vanilla Parity: Skeleton | good first issue,mobai,entity |
| #4266 | [TRACKER] Mob Vanilla Parity: Shulker | good first issue,mobai,entity |
| #4265 | [TRACKER] Mob Vanilla Parity: Ravager | good first issue,mobai,entity |
| #4264 | [TRACKER] Mob Vanilla Parity: Pillager | good first issue,mobai,entity |
| #4263 | [TRACKER] Mob Vanilla Parity: Piglin Brute | good first issue,mobai,entity |
| #4261 | [TRACKER] Mob Vanilla Parity: Parched | good first issue,mobai,entity |
| #4260 | [TRACKER] Mob Vanilla Parity: Magma Cube | good first issue,mobai,entity |
| #4259 | [TRACKER] Mob Vanilla Parity: Hoglin | good first issue,mobai,entity |
| #4257 | [TRACKER] Mob Vanilla Parity: Guardian | good first issue,mobai,entity |
| #4256 | [TRACKER] Mob Vanilla Parity: Ghast | good first issue,mobai,entity |
| #4254 | [TRACKER] Mob Vanilla Parity: Evoker | good first issue,mobai,entity |
| #4253 | [TRACKER] Mob Vanilla Parity: Elder Guardian | good first issue,mobai,entity |
| #4252 | [TRACKER] Mob Vanilla Parity: Creeper | good first issue,mobai,entity |
| #4251 | [TRACKER] Mob Vanilla Parity: Creaking | good first issue,mobai,entity |
| #4250 | [TRACKER] Mob Vanilla Parity: Bogged | good first issue,mobai,entity |
| #4249 | [TRACKER] Mob Vanilla Parity: Breeze | good first issue,mobai,entity |
| #4247 | Can't waterlog stairs or slabs | bug,Java Edition,block,fluid |
| #4246 | [TRACKER] Mob Vanilla Parity: Blaze | good first issue,mobai,entity |
| #4244 | [TRACKER] Mob Vanilla Parity: Zombie Nautilus | good first issue,mobai,entity |
| #4243 | [TRACKER] Mob Vanilla Parity: Wolf | good first issue,mobai,entity |
| #4241 | [TRACKER] Mob Vanilla Parity: Spider | good first issue,mobai,entity |
| #4239 | [TRACKER] Mob Vanilla Parity: Polar Bear | enhancement,good first issue,mobai,entity |
| #4238 | [TRACKER] Mob Vanilla Parity: Piglin | enhancement,good first issue,mobai,entity |
| #4237 | [TRACKER] Mob Vanilla Parity: Panda | enhancement,good first issue,mobai,entity |
| #4236 | [TRACKER] Mob Vanilla Parity: Nautilus | enhancement,good first issue,mobai,entity |
| #4233 | [TRACKER] Mob Vanilla Parity: Goat | enhancement,good first issue,mobai,entity |
| #4231 | [TRACKER] Mob Vanilla Parity: Fox | enhancement,good first issue,mobai,entity |
| #4230 | [TRACKER] Mob Vanilla Parity: Drowned | enhancement,good first issue,mobai,entity |
| #4229 | [TRACKER] Mob Vanilla Parity: Dolphin | enhancement,good first issue,mobai,entity |
| #4228 | [TRACKER] Mob Vanilla Parity: Cave Spider | enhancement,good first issue,mobai,entity |
| #4227 | [TRACKER] Mob Vanilla Parity: Bee | enhancement,good first issue,mobai,entity |
| #4219 | [TRACKER] Mob Vanilla Parity: Strider | enhancement,good first issue,mobai,entity |
| #4217 | [TRACKER] Mob Vanilla Parity: Snow Golem | enhancement,good first issue,mobai,entity |
| #4216 | [TRACKER] Mob Vanilla Parity: Sniffer | enhancement,good first issue,mobai,entity |
| #4209 | [TRACKER] Mob Vanilla Parity: Ocelot | good first issue,mobai,entity |
| #4207 | [TRACKER] Mob Vanilla Parity: Mooshroom | good first issue,mobai,entity |
| #4206 | [TRACKER] Mob Vanilla Parity: Happy Ghast | good first issue,mobai,entity |
| #4201 | [TRACKER] Mob Vanilla Parity: Copper Golem | good first issue,mobai,entity |
| #4197 | Entities that pass through nether portals don't appears at the other side | bug,protocol,entity |
| #4194 | Can't turn off a nether portal with water | bug,block,fluid |
| #4192 | [TRACKER] Mob Vanilla Parity: Axolotl | enhancement,good first issue,mobai,entity |
| #4191 | [TRACKER] Mob Vanilla Parity: Armadillo | enhancement,good first issue,mobai,entity,data |
| #4188 | [TRACKER] Mob Vanilla Parity: Pig | enhancement,good first issue,mobai,entity |
| #4187 | Ender pearl don't works when you try to interract with a block with it | bug,item,player |
| #4185 | [TRACKER] Vanilla Parity: Mobs | enhancement,mobai,entity |
| #4178 | Campfire dupes food | bug,good first issue,block,item,inventory |
| #4177 | `world.get-id` returns the same id for the overworld, the Nether and the End | bug,api,world |
| #4176 | First chunk generation in the Nether can deadlock its whole generation pool (stronghold rings comput | bug,worldgen,panic,performance |
| #4169 | Disconnecting with the player inventory open loses the cursor item and 2x2 crafting grid items | bug,inventory,player,data |
| #4168 | Some squares re-appear "is under spawn protection" | bug,Java Edition,player |
| #4166 | Generate WIT types and Rust conversions for packets and events from one model | enhancement,protocol,api |
| #4156 | Plugin callback during `pump_blocking` can deadlock the whole server | bug,api,panic,performance |
| #4154 | `player.play-sound` / `play-sound-at` derive the wrong sound name and trap the plugin | bug,good first issue,api,sound |
| #4148 | Offline-mode UUIDs differ from vanilla | bug,Java Edition,player |
| #4147 | Permission confirmation blocks plugin loading without a console | bug,api,config,permissions |
| #4146 | No way to resend a player's command tree | enhancement,api,command,permissions |
| #4145 | Host calls inside a blocking join handler can hang the join | bug,api,panic,player |
| #4144 | `game-profile` command arguments arrive as empty strings | bug,good first issue,api,command |
| #4143 | Names come with or without the `minecraft:` namespace depending on the event | bug,api,block,item |
| #4142 | `inventory-open` event has no container position | bug,api,command,inventory |
| #4141 | Explosion events: no affected block list, and one TNT fires two events | bug,api,block,entity |
| #4140 | Block, explosion and piston events carry no world | bug,api,block,world |
| #4139 | `world.set-block-entity-nbt` has no effect on loaded block entities | bug,api,block,data |
| #4138 | Bucket events fire after the world changed; cancelling does not undo it | bug,api,block,fluid |
| #4137 | Event priorities are ignored | bug,api |
| #4136 | Typing a plugin command with server-side suggestions into the console crashes the server | bug,api,panic,command |
| #4135 | Evokers do not drop totems | bug,mobai,item,loot |
| #4134 | Totems do not appear to work | bug,item,player |
| #4132 | Server appears to assume every mob has a baby varient | bug,Java Edition,mobai,entity |
| #4130 | Several issues | bug,Bedrock Edition,block,inventory |
| #4129 | Composter does not take seeds | bug,good first issue,Java Edition,block,inventory |
| #4128 | Jukebox music disc duplication | bug,block,item,inventory |
| #4127 | Low RAM usage but High CPU usage | bug,performance |
| #4126 | Dolphins have several issues relating to their behavior. | bug,Java Edition,mobai,entity |
| #4124 | Villages do not appear to spawn villagers or iron golems | bug,Java Edition,worldgen,mobai |
| #4123 | Server does not appear to send the packet for the animation when a warden hits you | bug,good first issue,protocol,Java Edition,entity |
| #4122 | Wardens do not have a sonic boom | bug,Java Edition,mobai,entity |
| #4121 | Mobs have a unusual collision | bug,Java Edition,mobai,entity |
| #4109 | Server freezes on server.create_world method | bug,api,panic,world |
| #4090 | Floting Fish | bug,mobai,entity |
| #4089 | Firework Rockets All being Power 1 | bug,good first issue,combat,item |
| #4084 | TNT explodes with delay | bug,block,performance |
| #4079 | Nether trees can generate through solid terrain | bug,Java Edition,worldgen |
| #4078 | some dungeons are generated differently than in vanilla. | bug,worldgen |
| #4077 | Dripstone caves are not generating correctly. | bug,worldgen |
| #4076 | geodes do not generate | bug,worldgen |
| #4075 | Empty command in console should be ignored not error | bug,good first issue,command |
| #4070 | Parts of trees generating across a chunk border get replaced with grass | bug,worldgen,world |
| #4063 | Fish go wild on land | bug,mobai,entity |
| #4052 | Clarity Regarding Docker Image Tags | question,docker |
| #4048 | Memory grows by about 3 MB with every player join and leave | bug,performance,player |
| #4036 | Spawn Egg problems | bug,protocol,Java Edition,entity |
| #4035 | deep dark generates practically nothing in terms of sculk and only sculk veins | bug,worldgen,block |
| #4034 | Woodland Mansion generates buried while the vanilla counterpart does not | bug,worldgen |
| #4027 | shipwreck stairs have a incorect block state | bug,worldgen,block |
| #4021 | Support for Experimental Minecart Improvements | enhancement,Bedrock Edition,entity |
| #4020 | Loot table generation uses non deterministic RNG instead of seed-based random sequences (1.20+) | bug,Java Edition,loot |
| #4011 | Extending a chest from diagonally above does not work properly | bug,block,inventory |
| #4006 | Bug: PacketSentEvent handlers are rarely or never invoked by the core event dispatcher | bug,protocol,Bedrock Edition,api |
| #4005 | Aquatic mobs bobbing instead of swimming | bug,Java Edition,mobai,entity |
| #3989 | Trial Vault crashes the server | bug,good first issue,Java Edition,panic,block |
| #3974 | fix(entity): correct hoglin and zoglin throw guards and velocity behavior | bug,mobai,combat,entity |
| #3973 | Placing a powder snow bucket by hand consumes the bucket instead of returning an empty one | bug,good first issue,Java Edition,block,item |
| #3972 | Powder snow placed from a bucket by a dispenser is not shown to clients | bug,Java Edition,block,item |
| #3971 | Mob buckets do not keep the mob data, so a released axolotl gets a random color | bug,item,entity,data |
| #3970 | Dispensers drop filled buckets in front of replaceable blocks (short grass) instead of replacing the | bug,Java Edition,block,item |
| #3969 | Filled water and mob buckets create a water block above seagrass instead of using its water | bug,block,item,fluid |
| #3933 | Catching a fish, axolotl or tadpole with a water bucket does not give the mob bucket | bug,Java Edition,item,entity |
| #3926 | Structure generation and /locate have no per-dimension structure state | bug,worldgen,performance |
| #3917 | Crafting Table Duplication Glitch | bug,Java Edition,inventory,recipe |
| #3908 | feat(api/wit) EntityType from/to registry key helpers | enhancement,api |
| #3907 | https://modrinth.com/mod/footprint | enhancement,worldgen,performance |
| #3902 | Chest contents and structure loot tables are lost on `stop` in chunks away from spawn (only an expli | bug,inventory,world,data |
| #3900 | Piston problem | bug,Java Edition,block,redstone |
| #3899 | Flying in creative while flying with a elytra you move strangly | bug,Java Edition,player |
| #3888 | Taking damage/getting killed while sleeping doesn't wake the player up | bug,Java Edition,player |
| #3881 | Hopper-to-hopper cooldown compares cooldown_time instead of tickedGameTime | bug,Java Edition,block |
| #3880 | Hopper minecart picks up a dropped stack one item at a time | bug,Java Edition,entity |
| #3879 | Floating snow in Snowy Taiga village | bug,Java Edition,worldgen |
| #3867 | Tracking: GameTest framework issues | bug,testing,entity,world |
| #3858 | Indiscriminate use of `tokio::sync` throughout the codebase | enhancement,performance,refactor |
| #3855 | Breaking block below snow carpet bugs out in random ways | bug,block,world |
| #3847 | dupe with chiseled bookshelfs | bug,Java Edition,item,inventory |
| #3844 | Crossbow Loading results in client kick | bug,panic,combat,item |
| #3841 | Plugin API: Mutable (bedrock) resource pack configuration | enhancement,Bedrock Edition,api |
| #3838 | Button noise fires twice | bug,good first issue,block,sound |
| #3837 | Standing on one block above a magma block with snow placed on it does damage | bug,Java Edition,block,player |
| #3836 | When I teleport with /tp into not generated chunks i die in the void | bug,command,player,world |
| #3833 | Pistons can be powered from the head when the head is facing down. | bug,block,redstone |
| #3831 | In Bedrock edition, if try to move inside a 1 block whole you can Crawl | bug,Bedrock Edition,player |
| #3830 | Fix shearing animation | bug,Java Edition,entity |
| #3822 | Dispensers can only shear sheep and beehives | bug,Java Edition,block,item |
| #3818 | Old Growth Spruce missing Blocks in 3rd layer | bug,Java Edition,worldgen,block |
| #3802 | First time world join kicked for packet spam | bug,protocol,performance,player |
| #3782 | Failed to decode packet 'clientbound/minecraft:container_set_content' | bug,protocol,Java Edition,inventory |
| #3779 | /setblock returns syntax error when adding block data | bug,Java Edition,command,block |
| #3777 | Packet Decode failure for clientbound/minecraft:level_particles (Client Disconnect) | bug,protocol,panic,particles |
| #3772 | Fire propagation from natural lava source is infinite | bug,block,world |
| #3767 | Item frames lack placement and survival validation | bug,block,entity |
| #3761 | Villager trading | bug,mobai,inventory,player |
| #3760 | /tick sprint shows completion message when starting | bug,good first issue,command |
| #3759 | Recursive /function calls crash the server with a stack overflow | bug,Java Edition,panic,command |
| #3758 | /execute store rejects valid vanilla commands | bug,Java Edition,command |
| #3742 | Villagers move while sleeping | bug,Java Edition,mobai,entity |
| #3738 | Bedrock 1.26.51 cannot join via NetherNet – TLS ClientHello receives plaintext HTTP 400 | bug,protocol,docker,Bedrock Edition |
| #3737 | Breaking Block Sound only plays shortly after starting to break a block | bug,Java Edition,block,sound |
| #3735 | I would like to be able to support more Bedrock versions of Java in parallel, instead of just one fi | enhancement,Bedrock Edition |
| #3734 | Plugin API: per-player entity visibility (show/hide entity, visible-by-default) and non-persistent e | enhancement,api,player,entity |
| #3712 | PumpkinMC does not generate some structures correctly | bug,Java Edition,worldgen |
| #3702 | TypeScript Wasm plugin fails after registering a command during `on-load` | bug,api,panic,command |
| #3701 | Building a plugin with pumpkin-plugin-utils' init fails | bug,dependencies,api |
| #3690 | End crystals damage, knockback and placement are not working as the intended way | bug,Java Edition,combat,entity |
| #3688 | WIT: player-action-add-player.properties reuses attribute-shaped property struct | bug,protocol,Java Edition,api |
| #3672 | Falling blocks don't slow with cobwebs | bug,Java Edition,block,entity |
| #3659 | Fire may stay floating mid air after burning down a tree/forest | bug,Java Edition,block |
| #3646 | Chiseled bookshelf book changes emit no block-change game event | bug,good first issue,block,inventory,sound |
| #3643 | Inserting or ejecting a jukebox record emits no block-change game event | bug,good first issue,block,sound |
| #3631 | Bells never resonate or reveal nearby raiders | bug,block,entity |
| #3628 | Add support for arm/v7 | enhancement,docker |
| #3627 | Dispensers drop bone meal instead of applying it as fertilizer | bug,block,redstone,item |
| #3626 | Creepers with status effects leave no area-effect cloud when they explode | bug,mobai,entity |
| #3624 | Experience orbs sink instead of floating in water | bug,good first issue,entity,fluid |
| #3623 | Tripwire state changes emit no game events | bug,good first issue,block,redstone,sound |
| #3622 | Broken paintings do not drop a painting item | bug,good first issue,item,entity |
| #3621 | Opening and closing containers emits no game events | bug,good first issue,block,inventory,sound |
| #3618 | Slimes cannot spawn in swamps or mangrove swamps | bug,Java Edition,worldgen,mobai |
| #3614 | Armor stands cannot equip or swap items through player interaction | bug,item,player,entity |
| #3603 | Add "mob rate" options in pumpkin.toml | enhancement,mobai,config |
| #3593 | Native `#[plugin_method]` futures are polled outside `GLOBAL_RUNTIME`, causing tokio reactor panics | bug,api,panic |
| #3581 | Players cannot use beds occupied by sleeping villagers | bug,mobai,block,player |
| #3580 | Farmland and dirt paths turning into dirt can leave entities embedded | bug,block,player,entity |
| #3575 | Pass the used hand down to item behaviours (normal_use, normal_use_with_rotation) | api,item,refactor |
| #3574 | Breaking tripwire with shears does not consume durability | bug,good first issue,block,item |
| #3567 | Waxing copper trapdoors resets their orientation and open state | bug,good first issue,block |
| #3566 | Endermen place carried blocks where the block cannot survive | bug,good first issue,mobai,block,entity |
| #3561 | Attempt to attack invalid entity when hitting ender dragon | bug,panic,combat,entity |
| #3558 | Sugar cane cannot be planted beside water or waterlogged blocks | bug,good first issue,block |
| #3557 | Lily pads cannot be placed on waterlogged blocks | bug,good first issue,block |
| #3551 | Scheduled block/fluid ticks execute in chunks outside `active_chunks` (asymmetric with random ticks) | bug,block,world,fluid |
| #3548 | Jukebox playback state never resumes after chunk or server reload | bug,block,data,sound |
| #3542 | Curse of Binding armor can be removed in non-Creative modes | bug,good first issue,item,inventory,enchantments |
| #3538 | `/gamemode` uses the wrong variables | bug,command,player |
| #3522 | Successful fishing yields no item, experience, or rod durability cost | bug,item,player,loot |
| #3521 | Bed and respawn-anchor explosions do not create fire | bug,block,world |
| #3520 | Shields cannot block ordinary player melee attacks | bug,combat,player |
| #3519 | Tracking: implement Wilderness Bound gameplay and world generation from Java 26.3 | enhancement,worldgen,block,item |
| #3518 | Dispensers lose aquatic mobs from filled buckets | bug,block,item,entity |
| #3516 | Crops continue growing below vanilla light level | bug,block,world |
| #3515 | Explosion knockback ignores Blast Protection and the resistance attribute | bug,combat,entity,enchantments |
| #3514 | Uncharged respawn anchors explode outside the Nether | bug,good first issue,block,world |
| #3513 | Attack packets bypass server-side entity interaction range | bug,protocol,combat,player |
| #3512 | Older periodic player snapshots can overwrite newer disconnect saves | bug,player,data |
| #3511 | Player data saves truncate the last good file before the new write succeeds | bug,player,data |
| #3489 | Arrows dont hurt | bug,Java Edition,combat,entity |
| #3487 | Cant Connect on Bedrock | bug,Bedrock Edition,config |
| #3468 | Tracking: Mob AI | mobai,entity,data |
| #3464 | Pressing against a mob after taking knockback launches the player upwards (server keeps sending the  | bug,protocol,player,entity |
| #3456 | Implement the minecraft:copy_components loot function (block entity data is lost on break) | bug,block,item |
| #3455 | Define authoritative scoreboard source for gameplay team membership | api,mobai,player,entity |
| #3448 | Select the center block for collapsed world border clamp ranges | bug,good first issue,world |
| #3446 | Deduplicate Java experience synchronization packets | good first issue,protocol,Java Edition,refactor,player |
| #3431 | Configuration load failures are completely silent: the server starts with default values and prints  | bug,config |
| #3396 | Bedrock: multiple bugs (checking) | bug,Bedrock Edition,panic,inventory |
| #3392 | Prevent datapack loaders from recursing into symbolic-link directories | bug,security,data |
| #3391 | Dying deletes (or does insane durability to) armor equipped | bug,Java Edition,item,player |
| #3389 | Teleporting across dimensions causes visual and client bugs | bug,protocol,Java Edition,player |
| #3388 | Arrows have glitchy particles | bug,combat,particles |
| #3385 | Entities persist after leaving simulation distance causing them to become invalid | bug,Java Edition,entity,world |
| #3382 | Attacking ender dragon disconnected me | bug,panic,combat,entity |
| #3376 | Mobs do not aggro on player, and only sometimes do when attacked with weapon | bug,Java Edition,mobai,entity |
| #3368 | Items floating in water for an extended amount of time can't be picked up | bug,Java Edition,item,fluid |
| #3367 | game_rules/weather/wandering_trader read from root only, Paper-Purpur worlds silently lose them | bug,world,config,data |
| #3340 | Problems with the proxy server | bug,protocol,Bedrock Edition |
| #3276 | Datapack load functions execute during reload while game ticks are frozen | bug,command,data |
| #3246 | GUI dashboard | enhancement,api |
| #3239 | Lava and water interaction | bug,Bedrock Edition,Java Edition,fluid |
| #3166 | Soulspeed does not work | bug,item,enchantments |
| #3163 | Plugin API: get-command-sender-type and require-with-handler-id are stubs that trap the plugin insta | bug,api,command |
| #3144 | Extended piston client desync | bug,block,redstone |
| #3143 | Free affect's - Switch Exploit. | bug,combat,item,player |
| #3133 | Lever: on_state_replaced ignores the piston `moved` flag, and normal_use returns Consume instead of  | bug,good first issue,block,redstone |
| #3126 | Trees Got Clipped at Chunk Border | bug,Java Edition,worldgen |
| #3124 | Village Generated Under Ground | bug,Java Edition,worldgen |
| #3116 | Chunk delivery to a moving client stalls at ~1 chunk/tick while generation runs far ahead | bug,performance,world |
| #3114 | Sulfur Cube mobs do not have AI | bug,mobai,entity |
| #3108 | Player disconnects when picking up any painting into the inventory. | bug,Java Edition,item,inventory |
| #3107 | Sculk Shrieker doesnt activate | bug,good first issue,block,redstone |
| #3105 | Observers doesnt work | bug,block,redstone |
| #3103 | Cant place paintings or itemframes | bug,item,entity |
| #3100 | Swamp huts dont generate | bug,worldgen |
| #3099 | Magma blocks generated in the terrain dont produce bubbles underwater. | bug,block,fluid |
| #3098 | Village paths are not generating correctly. | bug,worldgen |
| #3096 | Tracking: API | enhancement,api |
| #3092 | XP Orbs do not absorb | bug,item,player |
| #3089 | `player_server` no reactor running | bug,panic |
| #3086 | Http Output Plugin isssue | bug,api |
| #3082 | Chest on chest boat doesnt work | bug,inventory,entity |
| #3073 | Can't open inventory when item has custom data | bug,api,item,data |
| #3071 | Minecarts not going uphill | bug,entity,world |
| #3069 | Trident Drowned Only Attack in Melee Range | bug,mobai,combat,entity |
| #3068 | Boats cannot be broken by hand | bug,item,entity |
| #3065 | Eyeblossom state changes send malformed Trail particle data and disconnect 26.2 clients | bug,protocol,particles |
| #3064 | take damages when you get closer to a block | bug,Bedrock Edition,player |
| #3062 | Sweeping attack effect appears in the wrong position | bug,good first issue,combat,particles |
| #3061 | Incorrect boat placement. Boats do not drop boat item after breaking. | bug,item,entity |
| #3060 | Mushrooms break unexpectedly | bug,block,world |
| #3053 | Spectator mode interactions | bug,redstone,player |
| #3051 | Fishing rod is broken | bug,item,entity |
| #3049 | Windows Defender | documentation,enhancement |
| #3040 | Seagrass floating on-top of water ravines | bug,worldgen,block |
| #3020 | Campfire block cannot cook and cannot be extinguished by potion | bug,block,item |
| #3005 | Stronghold ring placement samples biomes from the triggering chunk | bug,worldgen |
| #3004 | Server logs interpolation | bug,good first issue,localization,command |
| #3002 | Bell animation missing | bug,good first issue,block |
| #2981 | Player not kicked on server stop | bug,Bedrock Edition,command |
| #2976 | Bug: Furnace Interaction Restores Tool Durability in Survival Mode | bug,item,inventory |
| #2901 | Advancements show up in unparsed form | bug,Bedrock Edition,advancements |
| #2885 | Incorrect Glow Lichen generation (full blocks and surface level) | bug,worldgen,block |
| #2883 | set-max-health allows values below 1 | bug,good first issue,api,player |
| #2879 | Mineshaft has too many fences | bug,worldgen |
| #2877 | Trail ruins generate with only suspicious gravel | bug,worldgen |
| #2872 | Enderman floating and walking in mid-air | bug,mobai,entity |
| #2871 | Banned Player Join Event | enhancement,api,player |
| #2861 | Build fails with rustc SIGSEGV while compiling cranelift-codegen | bug,question |
| #2854 | Numa Awareness | enhancement,performance |
| #2827 | Crash related to Storage blocks | bug,worldgen,panic |
| #2810 | Plugin API: world-editing follow-ups for WorldEdit-style plugins | enhancement,api,world |
| #2781 | Birch tress generating next to each other | bug,worldgen |
| #2741 | Lock issue with high player count - concurrent command run | bug,command,performance |
| #2699 | Redstone state updates are lost / desynced when activated | bug,redstone |
| #2632 | Add mutable enchanting-table events to the plugin API | enhancement,api,enchantments |
| #2626 | Chunk writer drops block_entities, structures and PostProcessing when saving a vanilla 26.2 world | bug,world,data |
| #2601 | Redstone is freaking out | bug,redstone |
| #2600 | Spectator Mode / 2 Accounts | bug,player |
| #2592 | execute if block ignores its predicate; blockstate args, forceload and CLightUpdate also broken | bug,command,block,world |
| #2573 | Saturation effect tweaking | bug,player |
| #2552 | Server crashes when testing | bug,panic,world |
| #2520 | /Effect command desynchronizing, When Leaving / Joining | bug,command,player |
| #2516 | Add functionality to resolve file-based secrets for Velocity configuration. (Example @forwarding.sec | enhancement,security,config |
| #2501 | Structures are generated incorrectly | bug,worldgen |
| #2478 | Proposal: Adopt AGPLv3 for future contributions | documentation,question |
| #2466 | Something wrong in the nether | bug,worldgen |
| #2427 | Lighting not updating properly | bug,world |
| #2370 | Bedrock Connection Issues | bug,Bedrock Edition,player |
| #2362 | Incorrect terrain generation in the Jungle | bug,worldgen |
| #2359 | a few bugs i found | bug,mobai,item,entity |
| #2291 | Bedrock entity spawn crashes during chunk generation on Gen-Pool threads (no Tokio runtime) | bug,Bedrock Edition,panic |
| #3484 | plugin-wit: Not all interfaces are imported | bug,api |
| #2232 | Mobs ignore hitbox after getting hit by player + ignore bamboo | bug,mobai,entity |
| #2222 | No keyrepeat in text fields | bug,player |
| #2220 | Ender Dragon collision bug | bug,combat,entity |
| #2211 | Block IDs table for plugin developers | enhancement,api,block |
| #2194 | mobs can walk in air/ground | bug,mobai,entity |
| #2174 | I see SNK wall | bug,worldgen |
| #2162 | Drowned Zombie burn in water when jumping | bug,mobai |
| #2151 | Visible clear-cut biome borders directly at chunk boundaries in Badlands/mesa biome | bug,worldgen |
| #2104 | Player floating and stuck in swim state(WARN: Terible english) | bug,protocol,Bedrock Edition |
| #2098 | Feat: dynamic storage types | enhancement,api,data |
| #2094 | XP points seemingly stop adding after the 32-bit signed integer limit | bug,player |
| #2092 | Caves generate in the ocean | bug,worldgen |
| #2065 | When the ender dragon fight ends the portal back to the overworld doesn't open | bug,world |
| #2057 | Panic when Docker start | bug,panic |
| #2056 | WASM Plugin re-entry/concurrency issue | bug,api,panic |
| #2027 | Weird terrain generation, random blocks generating in the middle of grass and underfilled trees | bug,worldgen |
| #1960 | Blazes can't fly | bug,mobai |
| #1949 | Worldgen: Incorrect ore/stone generation | bug,worldgen |
| #1922 | player bugs | bug,player,entity,sound |
| #1871 | cannot place Scaffolding like usual | bug,block |
| #1800 | attemping to attack an invalid entity | bug,combat,entity |
| #1781 | Bot Connection Issue - Compression Protocol Incompatibility with mineflayer | enhancement,protocol,Java Edition |
| #1726 | Tracking: Translations | enhancement,localization |
| #1679 | Players in boats don't move | bug,player,entity |
| #1674 | When spawning water based animals on land. The water based animals will not suffocate and despawn. | bug,entity,fluid |
| #1583 | Dying in the Nether without setting a spawnpoint, respawns on top of nether roof. | bug,player,world |
| #1564 | Flying machines bring unpredictable behavior | bug,block |
| #1555 | Memory leak when loading void chunks | bug,performance,world |
| #1552 | Riptide Tridents inconsistently work while it's raining | bug,combat,item,fluid |
| #1551 | Elytra State & Using Fireworks with (and without) one equipped. | bug,item,player |
| #1482 | Tracking: Codebase Docs | documentation,enhancement |
| #1404 | Tracking: Combat | enhancement,good first issue,combat |
| #1336 | Zombie path block detection broken | bug,mobai |
| #1330 | When you Crit even tho the cooldown is fully charged (you can make a crit in normal minecraft) it ma | bug,combat |
| #1300 | Coral reef biome spawns broken coral patterns | bug,worldgen |
| #1280 | Configurable API server endpoints | enhancement,api,config |
| #1274 | Elytra Bug , Fire work 2&3 rocket crash the server , wind charge Not working ,Potions crash the serv | bug,panic |
| #1220 | A fog problem in my server | bug,invalid,world |
| #1199 | Unable to quit/infinite loop during save after client fails to connect | bug,panic,data |
| #877 | Naturally generated water sometimes doesn't flow. | bug,world,fluid |
| #731 | Create unified registry system to load data packs, including the vanilla data pack | enhancement,worldgen,data |
| #635 | Add Persistent Data Container | enhancement,api,data |
| #626 | KnockbackSync, custom Knockback, legacy PVP | enhancement,api,combat,config |
| #449 | Release Pumpkin 1.0.0 |  |
| #118 | Support for multiple World/Chunk formats | enhancement,help wanted |

## Deferred

Tech-debt pass (Phase 3, `docs/GOAL_FINAL.md`). Each entry is a `TODO`/`FIXME`/`HACK`/`XXX` comment in `crates/` (excluding `generated/`) left in place, with the reason. Grep: `grep -rnE '\b(TODO|FIXME|HACK|XXX)\b' crates/ --include='*.rs' --exclude-dir=generated`.

- **Bedrock protocol enums and placeholder fields** (`pumpkin-protocol/src/bedrock/**`): typed enums (`CurrentCmdVersion`, `InventoryLayout`, `PacketViolationType`, `ModalFormCancelReason`, `ItemVersion`, `CompressionAlgorithm`, RuleValue, LE handling in `start_game.rs`) need vanilla Bedrock values that are not extracted into `assets/`. Blocked on an Extractor.
- **Bedrock inventory packets** (`creative_content.rs`, `add_player.rs`, `add_item_actor.rs`): "update inventory" needs the Bedrock inventory sync path, not started.
- **Java protocol custom impls** (`teleport_entity.rs`, `player_position.rs`, `player_info_update.rs`, `player_chat_message.rs`, `recipe.rs`, `data_component.rs`, `dialog.rs`, `lib.rs` Style write): each needs a vanilla codec comparison before the derive can replace the hand impl. Unverified, so kept.
- **Typed data components** (`pumpkin-data/src/data_component_impl/combat.rs`, `item_stack/mod.rs`): block predicates and trim material/pattern registries are not modelled; `#[must_use]` on `item_stack/mod.rs` waits for all callsites.
- **Config and data validation** (`pumpkin-config/src/world.rs`, `pumpkin-core/src/data/{whitelist,banned_ip,banned_player}.rs`): needs a validation design for config and list files; not scoped to this pass.
- **Wasm host gametest async** (`pumpkin-wasm-host-v0_2/src/gametest.rs`, 14 sites): needs `bindings.rs` async support; owned by the plugin workstream.
- **Wasm host event conversion** (`pumpkin-wasm-host-v0_1/src/events/server.rs:70`): reverse WIT-to-raw conversion is only needed if a plugin path requires it. Public WIT, additive changes only.
- **Mob and passive entities** (`entity/mob/**`, `entity/passive/**`, `entity/ai/goal/**`): AI goal ordering, baby state, path retries and spawn-reason attributes. Owned by the mob AI workstream and audited via `MobEntity::audited_reach`.
- **Player entity** (`entity/player.rs`, `entity/living.rs`): gliding, team colour, world-change event ordering, difficulty scaling, delta-movement relative flags, metadata broadcast, and the atomic-slot ABA note. Each needs a vanilla parity audit; no change made here.
- **Core world** (`world/mod.rs`, `world/time.rs`, `command/mod.rs`, `plugin/mod.rs`): packet scoping to loaded chunks, light updates, spawn compass, ender dragon, bedrock frozen time, command-block naming, and plugin command-tree cleanup on reload. Behaviour changes, not cleanups.
- **Block entities and blocks** (`block/entities/hopper.rs`, `mob_spawner.rs`, `trial_spawner.rs`, `block/blocks/piston`, `infested.rs`, `shulker_box.rs`): SpawnData NBT, hopper block-state on load, and vanilla break-event ordering each change observable behaviour.
- **Bedrock chat and player actions** (`net/bedrock/play/chat_message.rs`, `player_action.rs`): the chat note is an unresolved question; `player_action.rs:278` has no context in the comment.
- **Chunk sending** (`net/chunk_sender.rs:576`): the lifecycle is built twice (Java and Bedrock); merging needs a design, not a cleanup.
- **Use item and teleport** (`use_item.rs:192`, `teleport_to_entity.rs:21`): fishing rod durability on retrieval and the cross-world teleport search are gameplay parity items.
- **Inventory** (`pumpkin-inventory/**`): player screen handler offhand and crafting result, drag handler stack limits, bundle support, entity equipment ticks, and inventory NBT load. Each needs its own vanilla port.
- **World generation and chunk system** (`pumpkin-world/**`): height provider min>max investigation, surface caching, noise router test coverage, heightmap tracking, palette serialization decision, and the chunk-system ticket lifetime. These are performance or correctness investigations.
- **Test TODOs** (`pumpkin-util/src/random/mod.rs`, `pumpkin-world/.../density_function/test.rs`, `pumpkin-world/src/lib.rs`): missing unit tests and extracted noise values. Owned by the test workstream.
- **Util and misc** (`pumpkin-util/src/noise/simplex.rs`, `text/color.rs`, `translation.rs`): simplex duplication extraction, terminal true-colour detection, and empty or missing translation file handling. Low risk but no-op for the goal; deferred.

## Deferred worker commits (not ported)

Old `kab/*` worker branches were triaged and removed. These commits were judged too large or contradicting vanilla; fetch by hash if still reachable (`git show <hash>`):

- `c9c056d52` nautilus goal sets (new 264-line `nautilus_goals.rs`, tempt stop distance): L, needs its own pass.
- `eaa5a3001` guardian goal refactor: would duplicate master's fuller `GuardianAttackGoal`; port only the missing parts.
- `35302769c` piglin baby roll and baby-gated weapon: needs `equipment.rs` `equip_mob_on_spawn` gating.
- `2de14f0a1` uniform bonus level-0 guard: rejected, contradicts vanilla `ApplyBonusCount`.
- `fdc682aaa` compass lodestone unlink: rejected, contradicts vanilla `LodestoneTracker.tick`.
- `d456eb83c` generated configured features lint change: superseded by the codegen allow list fix.
