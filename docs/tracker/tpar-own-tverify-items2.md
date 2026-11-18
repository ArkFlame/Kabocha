# Parity rows for own-tverify-items2

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/tverify-items2** [partial/OWN-OPEN] own audit: 4 open, 3 fixed, 4 verified, 0 blocked-data (docs/tracker/tverify-items2.md) docs/tracker/tverify-items2.md

## Result
All rows closed. No source edits were needed in this pass: the previously open items were already fixed in the shared checkout and each verdict below was re-checked against the decompiled vanilla source.

| Subject | Verdict | Evidence |
|---|---|---|
| Piglin (`entity/mob/piglin.rs`, `piglin_ai.rs`) | VERIFIED | `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/piglin/Piglin.java`: baby speed modifier 0.2 with ADD_MULTIPLIED_BASE, attributes 16 HP / 0.35 speed / 5 damage, spawn rule `!below.is(NETHER_WART_BLOCK)`, inventory slot offset 300, XP 5, 0.2 baby / 0.1 armour probabilities, and the synced baby/charging/dancing flags all match. Goal priorities and `hoglins_outnumber_piglins` / wants-to-stop-fleeing match `PiglinAi`. Vanilla drives AI with a Brain where Pumpkin uses a GoalSelector, an acceptable same-observable-behaviour difference. Crossbow charge power and hoglin AI stay out of scope per the earlier brief. |
| Sulfur Cube (`entity/mob/sulfur_cube.rs`) | VERIFIED | `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/cubemob/SulfurCube.java`: SPLIT_COUNT 2, MAX_SIZE 2, MIN_SIZE 1, PICKUP_TIMER 100, PUSH_DISTANCE_THRESHOLD 1.3, MAX_PLAYER_PUSH_SPEED 0.5, push speed scales 0.3 / 0.16 / 0.3, hit/position angle scales 1.6 / 0.5 / 0.8, extra knockback dampening 0.25, and knockback power 0.33 / 0.06 all match. Fuse timing, archetype selection, contact damage, bucket and shear paths are implemented. |
| Bat (`entity/mob/bat.rs`) | VERIFIED | `.vendor/vanilla-decompiled/net/minecraft/world/entity/ambient/Bat.java`: spawn rules (brightness <= rand(4), BATS_SPAWNABLE_ON tag, surface heightmap), RNG order in target selection, velocity/yaw steering, liftoff sound, the redstone-conductor rest check (Pumpkin `BlockState::is_redstone_conductor`, generated in pumpkin-data/src/block_state.rs, used at bat.rs:99 and :201), hurt waking before cooldown but not when invulnerable, and BatFlags NBT all match. The only gap is the CraftBukkit `BatToggleSleepEvent`, a Paper plugin hook outside core vanilla parity. |
| Lodestone (`block/blocks/lodestone.rs`) | VERIFIED | Vanilla lodestone is a plain Block with no overrides. Pumpkin registers `LodestoneBlock` with an empty `BlockBehaviour`. POI is derived generically from block-state ids: `PoiType::from_state_id(25187) == Some(PoiType::Lodestone)` (generated pumpkin-data/src/generated/poi_type.rs), applied on block changes by `World::update_poi` (world/mod.rs:6912) and rebuilt on chunk load by `poi_scan.rs::scan_chunk`. No block hooks are needed. |

Open follow-ups inside earlier FIXED rows, all resolved in the shared checkout:
- Fishing cast cancel: `item/items/fishing_rod.rs` fires `PlayerFishEvent(Fishing)` on cast and, when cancelled, clears `fishing_bobber` and returns PASS before any sound, spawn or stat (lines 89-108).
- Goat horn `get_use_duration`: `item/items/goat_horn.rs` reads the duration from the held stack `InstrumentImpl` (lines 109-114), returning 0 when there is no instrument; tests cover default, custom and unknown instruments.
- Bat conductor predicate: `BlockState::is_redstone_conductor` exists in pumpkin-data generated from block data, so the bat rest check is no longer blocked.

No pure unit test was added: the only pure seam found (`bobber_sound_pitch`) is a formula mirror, which the MVP test rules exclude. World and player state behaviour needs gametest territory.
