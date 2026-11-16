# TODO 2: parity gap plan (Pumpkin parity tracker, 2026-11-16)

Sources: the Pumpkin parity tracker `Pumpkin-MC/Pumpkin-Website/src/pages/tracker/data.json` (version 26.3-26.63, tracker commit 79c88fdb6, updated 2026-11-16; fetch: `gh api repos/Pumpkin-MC/Pumpkin-Website/contents/src/pages/tracker/data.json -H "Accept: application/vnd.github.raw"`), docs/LEDGER.tsv (upstream issues and PRs and their Kabocha state), docs/queue-issues.tsv, docs/queue-prs.tsv, docs/OUTCOMES.tsv, and a grep of master at the cited lines. Machine-readable form: docs/PARITY.tsv (564 rows). Complements docs/TODO.md and docs/QUEUE.md.

Coverage labels: **COVERED** = an active upstream issue or PR in the ledger (TODO, IN-PROGRESS, COMMITTED-UNLANDED or FAILED) addresses it. **COVERED-SKIPPED** = upstream issue exists but the ledger says SKIPPED (no slice worked; reopen it). **COVERED-DONE** = the linked upstream issue is DONE, but the tracker row or a TODO on master still shows the gap, so verify before filing. **UNCOVERED** = no upstream issue or PR found. Title matching is a heuristic (phrase match on the entity or block name); every label was checked by hand for the concrete gaps.

## 1. Scoreboard

| Category | Entries | Done | Partial | Planned | Gap rows | Concrete gaps | Covered (active / skipped / done) | Uncovered | Re-review rows (tracked / untracked) |
|---|---|---|---|---|---|---|---|---|---|
| Entities | 160 | 79 | 81 | 0 | 81 | 18 | 11 / 2 / 4 | 1 | 63 (27 / 36) |
| Blocks | 201 | 174 | 27 | 0 | 27 | 9 | 2 / 1 / 0 | 4 | 18 (10 / 8) |
| Items | 54 | 48 | 6 | 0 | 6 | 0 | 0 / 0 / 0 | 0 | 6 (2 / 4) |
| Commands | 92 | 92 | 0 | 0 | 0 | 0 | 0 / 0 / 0 | 0 | 0 (0 / 0) |
| Redstone | 27 | 27 | 0 | 0 | 0 | 0 | 0 / 0 / 0 | 0 | 0 (0 / 0) |
| Combat | 14 | 13 | 0 | 1 | 1 | 1 | 0 / 0 / 0 | 0 | 0 (0 / 0) |
| World | 16 | 16 | 0 | 0 | 0 | 0 | 0 / 0 / 0 | 0 | 0 (0 / 0) |
| **Total** | **564** | **449** | **114** | **1** | **115** | **28** | **13 / 3 / 4** | **5** | **87 (39 / 48)** |

Reading the table: 449 of 564 rows are done. The 115 partial or planned rows split into **28 concrete gaps** (the tracker note names a missing behaviour) and **87 re-review rows** (the note only says the reviewed source changed; these are audit jobs, not proven gaps). Of the 28 concrete gaps: 13 covered by an active upstream issue, 3 covered only by a SKIPPED upstream issue, 4 covered by a DONE issue but still open on master (verify), 5 uncovered, 3 dropped or out of scope (2 already fixed on master, 1 design note).

## 2. UNCOVERED gaps (file upstream issues or slices), smallest first

Each row is verified on master. Brief every slice with the vanilla class, the automated test that fails without the fix, and the rule that tests are the gate.

| id | category | name | gap | evidence (file:line on master) | vanilla reference | size | suggested issue title |
|---|---|---|---|---|---|---|---|
| T2-001 | blocks | Ender Chest | Opening an ender chest does not anger piglins near a guarded block (PiglinBrain.onGuardedBlockInteracted call is a TODO). | crates/pumpkin-core/src/block/blocks/ender_chest.rs:109 | EnderChestBlock, PiglinAi.onGuardedBlockInteracted | S | Piglins do not react when a player opens an ender chest |
| T2-002 | blocks | Moving Piston (piston extension) | Breaking the piston head passes no player: break_block gets None, so the breaking player context is lost (TODO at line 25). | crates/pumpkin-core/src/block/blocks/piston/piston_extension.rs:25-26 | PistonHeadBlock.playerWillDestroy, PistonBaseBlock. Possibly overlaps open #4281 (generic title, COMMITTED-UNLANDED): check first. | S | Piston head break drops the breaking player context |
| T2-003 | blocks | Infested blocks | Breaking constructs a generic Entity with the silverfish type, not a SilverfishEntity, and does not call spawnAfterBreak; a TODO notes the drop hack. Tracker links #3615, now closed upstream and absent from the ledger. | crates/pumpkin-core/src/block/blocks/infested.rs:85 (TODO at :68) | InfestedBlock.spawnAfterBreak, SilverfishEntity | M | Infested blocks spawn a generic entity instead of a silverfish |
| T2-004 | blocks | Sculk Catalyst | No death listener, and no sculk-spreading block entity. Only on_scheduled_tick bloom reset exists. | crates/pumpkin-core/src/block/blocks/sculk/sculk_catalyst.rs:20 (no listener in file) | SculkCatalystBlock, SculkCatalystBlockEntity, SculkSpreader | M | Sculk Catalyst does not spread sculk on mob death |
| T2-005 | entities | Illusioner | No ranged bow goal, no spell state, no blindness or invisibility spell handlers. Raid movement alone does not give illusioner combat. No tracker umbrella in the ledger. | crates/pumpkin-core/src/entity/mob/illusioner.rs:30 (goal set) | Illusioner, SpellcasterIllager, RangedBowAttackGoal, spell goals | L | Illusioner has no bow attack and no spells |

Dropped after verification (no slice): Dirt Path and Farmland "push entities up on conversion" (tracker note is stale; `push_entities_up` is called at dirt_path.rs:21 and farmland.rs:85, 97-110, 151). Sculk Sensor listener dispatch (sensor.rs:74 `trigger` is not called from the world event path) is not a separate slice: it shares the root cause of open-but-FAILED #3107 (Sculk Shrieker doesnt activate).

## 3. COVERED gaps (do not duplicate)

| id | category | name | label | upstream issues / PRs (number:ledger state) | verify before filing |
|---|---|---|---|---|---|
| T2-C01 | entities | Blaze | COVERED | 1960:open / ledger: issue#1960#SKIPPED:link;issue#4246#COMMITTED-UNLANDED:title | #4246 umbrella committed, #1960 SKIPPED |
| T2-C02 | entities | Cave Spider | COVERED-SKIPPED | - / ledger: issue#4228#SKIPPED:title | umbrella #4228 SKIPPED |
| T2-C03 | entities | Hoglin | COVERED | - / ledger: issue#3974#DONE:title;issue#4259#TODO:title | umbrella #4259 TODO; hoglin.rs:183 TODO |
| T2-C04 | entities | Parched | COVERED | - / ledger: issue#4261#TODO:title | parched.rs:24 TODO (arrow payload) |
| T2-C05 | entities | Piglin Brute | COVERED | - / ledger: issue#4263#TODO:title | umbrella #4263 TODO |
| T2-C06 | entities | Ravager | COVERED | - / ledger: issue#4265#TODO:title | umbrella #4265 TODO |
| T2-C07 | entities | Silverfish | COVERED | - / ledger: issue#4269#TODO:title | umbrella #4269 TODO |
| T2-C08 | entities | Skeleton | COVERED | 2537:closed / ledger: issue#4267#TODO:title;issue#4275#DONE:title;issue#4492#TODO:title | interval #4267/#4492 TODO; #2537 closed |
| T2-C09 | entities | Spider | COVERED-SKIPPED | - / ledger: issue#4228#SKIPPED:title;issue#4241#SKIPPED:title | umbrella #4241, #4228 SKIPPED |
| T2-C10 | entities | Stray | COVERED-DONE | - / ledger: issue#4270#DONE:title | #4270 DONE; arrow payload still open |
| T2-C11 | entities | Vex | COVERED | - / ledger: issue#4271#FAILED:title | umbrella #4271 FAILED; vex.rs:47 TODO |
| T2-C12 | entities | Vindicator | COVERED-DONE | - / ledger: issue#4272#DONE:title | #4272 DONE; vindicator.rs:89 TODO remains |
| T2-C13 | entities | Warden | COVERED | - / ledger: issue#4123#TODO:title;issue#4273#FAILED:title | #4123 TODO (sonic boom); #4273 FAILED |
| T2-C14 | entities | Witch | COVERED-DONE | 1302:closed / ledger: issue#4274#DONE:title | #4274 DONE; witch.rs:112 TODO remains |
| T2-C15 | entities | Zoglin | COVERED-DONE | - / ledger: issue#3974#DONE:title | #3974 DONE; PLAYER-only constructor still present |
| T2-C16 | entities | Ender Dragon | COVERED | - / ledger: issue#2065#DONE:title;issue#2220#DONE:title;issue#3382#TODO:title;issue#3561#DONE:title;issue#4279#SKIPPED:title | #3382 TODO; ender_dragon.rs:259 TODO (parts not registered) |
| T2-C17 | entities | Bee | COVERED | - / ledger: issue#4227#COMMITTED-UNLANDED:title | umbrella #4227 COMMITTED-UNLANDED |
| T2-C18 | blocks | Dirt Path | FIXED-ON-MASTER | 3580:open;3560:closed / ledger: issue#3580#DONE:link | #3580 DONE; push_entities_up present |
| T2-C19 | blocks | Farmland | FIXED-ON-MASTER | 3580:open;3560:closed;3111:closed / ledger: issue#3580#DONE:link | #3580 DONE; push_entities_up present |
| T2-C20 | blocks | Sculk Shrieker | COVERED | 3107:open / ledger: issue#3107#FAILED:link | #3107 FAILED |
| T2-C21 | blocks | Sculk Sensor | COVERED | - / ledger: - | shared root with #3107 (no own issue) |
| T2-C22 | blocks | Tripwire | COVERED-SKIPPED | 3623:open;3574:open / ledger: issue#3574#SKIPPED:link | #3574 SKIPPED (shears durability); #3623 open |

Entries with an open upstream issue whose vanilla gap is not in the concrete list stay in the tracker-to-ledger flow; see docs/PARITY.tsv (`covered` and `issues` columns) for every row.

Upstream issues with a SKIPPED ledger state, linked from concrete or re-review rows (reopen or re-slice them rather than filing new ones): #1960 (Blaze flight), #2872 (Enderman), #3114 (Sulfur Cube AI), #3069 (Drowned trident), #3574 (tripwire shears), #4228 and #4241 (spider leap and wall climb), #4231 (Fox), #4237 (Panda), #4216 (Sniffer), #4244 (Zombie Nautilus), #3822 (Dispenser shears).

## 4. RE-REVIEW audit list (one worker each; no gaps invented)

Each row has a tracker note that only says the source changed. A worker compares the Rust file with decompiled vanilla, returns concrete gaps with file:line, and files them under section 2. Tracked means an upstream issue or title match exists in the ledger; untracked means none was found.

| category | name | status | source | coverage | upstream issues |
|---|---|---|---|---|---|
| entities | Bogged | partial | crates/pumpkin-core/src/entity/mob/skeleton/bogged.rs | RE-REVIEW | - |
| entities | Breeze | partial | crates/pumpkin-core/src/entity/mob/breeze.rs | RE-REVIEW | - |
| entities | Creaking | partial | crates/pumpkin-core/src/entity/mob/creaking.rs | RE-REVIEW | - |
| entities | Creeper | partial | crates/pumpkin-core/src/entity/mob/creeper.rs | RE-REVIEW | - |
| entities | Drowned | partial | crates/pumpkin-core/src/entity/mob/zombie/drowned.rs | RE-REVIEW | 3069:open;2162:open |
| entities | Elder Guardian | partial | crates/pumpkin-core/src/entity/mob/elder_guardian.rs | RE-REVIEW | - |
| entities | Enderman | partial | crates/pumpkin-core/src/entity/mob/enderman.rs | RE-REVIEW | 2872:open |
| entities | Endermite | partial | crates/pumpkin-core/src/entity/mob/endermite.rs | RE-REVIEW-UNTRACKED | - |
| entities | Guardian | partial | crates/pumpkin-core/src/entity/mob/guardian.rs | RE-REVIEW | - |
| entities | Piglin | partial | crates/pumpkin-core/src/entity/mob/piglin.rs | RE-REVIEW | 3376:open |
| entities | Sulfur Cube | partial | crates/pumpkin-core/src/entity/mob/sulfur_cube.rs | RE-REVIEW | 3114:open |
| entities | Zombie Villager | partial | crates/pumpkin-core/src/entity/mob/zombie/zombie_villager.rs | RE-REVIEW-UNTRACKED | - |
| entities | Wither | partial | crates/pumpkin-core/src/entity/boss/wither.rs | RE-REVIEW | - |
| entities | Armadillo | partial | crates/pumpkin-core/src/entity/passive/armadillo.rs | RE-REVIEW | - |
| entities | Axolotl | partial | crates/pumpkin-core/src/entity/passive/axolotl.rs | RE-REVIEW | - |
| entities | Bat | partial | crates/pumpkin-core/src/entity/mob/bat.rs | RE-REVIEW-UNTRACKED | - |
| entities | Camel Husk | partial | crates/pumpkin-core/src/entity/passive/camel.rs | RE-REVIEW-UNTRACKED | - |
| entities | Copper Golem | partial | crates/pumpkin-core/src/entity/passive/copper_golem.rs | RE-REVIEW | - |
| entities | Dolphin | partial | crates/pumpkin-core/src/entity/passive/dolphin.rs | RE-REVIEW | - |
| entities | Fox | partial | crates/pumpkin-core/src/entity/passive/fox/mod.rs | RE-REVIEW | - |
| entities | Goat | partial | crates/pumpkin-core/src/entity/passive/goat.rs | RE-REVIEW | - |
| entities | Happy Ghast | partial | crates/pumpkin-core/src/entity/passive/happy_ghast.rs | RE-REVIEW | - |
| entities | Nautilus | partial | crates/pumpkin-core/src/entity/passive/nautilus.rs | RE-REVIEW | - |
| entities | Ocelot | partial | crates/pumpkin-core/src/entity/passive/ocelot.rs | RE-REVIEW | - |
| entities | Panda | partial | crates/pumpkin-core/src/entity/passive/panda.rs | RE-REVIEW | - |
| entities | Sniffer | partial | crates/pumpkin-core/src/entity/passive/sniffer.rs | RE-REVIEW | - |
| entities | Strider | partial | crates/pumpkin-core/src/entity/passive/strider.rs | RE-REVIEW | - |
| entities | Tadpole | partial | crates/pumpkin-core/src/entity/passive/tadpole.rs | RE-REVIEW | - |
| entities | Zombie Nautilus | partial | crates/pumpkin-core/src/entity/passive/zombie_nautilus.rs | RE-REVIEW | - |
| entities | Mannequin | partial | crates/pumpkin-core/src/entity/decoration/mannequin.rs | RE-REVIEW-UNTRACKED | - |
| entities | Dragon Fireball | partial | crates/pumpkin-core/src/entity/projectile/dragon_fireball.rs | RE-REVIEW-UNTRACKED | - |
| entities | Experience Bottle | partial | crates/pumpkin-core/src/entity/projectile/experience_bottle.rs | RE-REVIEW-UNTRACKED | - |
| entities | Fishing Bobber | partial | crates/pumpkin-core/src/entity/projectile/fishing_bobber.rs | RE-REVIEW-UNTRACKED | - |
| entities | Acacia Boat | partial | crates/pumpkin-core/src/entity/vehicle/boat.rs | RE-REVIEW-UNTRACKED | - |
| entities | Acacia Chest Boat | partial | crates/pumpkin-core/src/entity/vehicle/boat.rs | RE-REVIEW-UNTRACKED | - |
| entities | Bamboo Chest Raft | partial | crates/pumpkin-core/src/entity/vehicle/boat.rs | RE-REVIEW-UNTRACKED | - |
| entities | Bamboo Raft | partial | crates/pumpkin-core/src/entity/vehicle/boat.rs | RE-REVIEW-UNTRACKED | - |
| entities | Birch Boat | partial | crates/pumpkin-core/src/entity/vehicle/boat.rs | RE-REVIEW-UNTRACKED | - |
| entities | Birch Chest Boat | partial | crates/pumpkin-core/src/entity/vehicle/boat.rs | RE-REVIEW-UNTRACKED | - |
| entities | Cherry Boat | partial | crates/pumpkin-core/src/entity/vehicle/boat.rs | RE-REVIEW-UNTRACKED | - |
| entities | Cherry Chest Boat | partial | crates/pumpkin-core/src/entity/vehicle/boat.rs | RE-REVIEW-UNTRACKED | - |
| entities | Command Block Minecart | partial | crates/pumpkin-core/src/entity/vehicle/minecart.rs | RE-REVIEW-UNTRACKED | - |
| entities | Dark Oak Boat | partial | crates/pumpkin-core/src/entity/vehicle/boat.rs | RE-REVIEW-UNTRACKED | - |
| entities | Dark Oak Chest Boat | partial | crates/pumpkin-core/src/entity/vehicle/boat.rs | RE-REVIEW-UNTRACKED | - |
| entities | Jungle Boat | partial | crates/pumpkin-core/src/entity/vehicle/boat.rs | RE-REVIEW-UNTRACKED | - |
| entities | Jungle Chest Boat | partial | crates/pumpkin-core/src/entity/vehicle/boat.rs | RE-REVIEW-UNTRACKED | - |
| entities | Mangrove Boat | partial | crates/pumpkin-core/src/entity/vehicle/boat.rs | RE-REVIEW-UNTRACKED | - |
| entities | Mangrove Chest Boat | partial | crates/pumpkin-core/src/entity/vehicle/boat.rs | RE-REVIEW-UNTRACKED | - |
| entities | Oak Boat | partial | crates/pumpkin-core/src/entity/vehicle/boat.rs | RE-REVIEW-UNTRACKED | - |
| entities | Oak Chest Boat | partial | crates/pumpkin-core/src/entity/vehicle/boat.rs | RE-REVIEW-UNTRACKED | - |
| entities | Pale Oak Boat | partial | crates/pumpkin-core/src/entity/vehicle/boat.rs | RE-REVIEW-UNTRACKED | - |
| entities | Pale Oak Chest Boat | partial | crates/pumpkin-core/src/entity/vehicle/boat.rs | RE-REVIEW-UNTRACKED | - |
| entities | Poplar Boat | partial | crates/pumpkin-core/src/entity/vehicle/boat.rs | RE-REVIEW-UNTRACKED | - |
| entities | Poplar Chest Boat | partial | crates/pumpkin-core/src/entity/vehicle/boat.rs | RE-REVIEW-UNTRACKED | - |
| entities | Spawner Minecart | partial | crates/pumpkin-core/src/entity/vehicle/minecart.rs | RE-REVIEW-UNTRACKED | - |
| entities | Spruce Boat | partial | crates/pumpkin-core/src/entity/vehicle/boat.rs | RE-REVIEW-UNTRACKED | - |
| entities | Spruce Chest Boat | partial | crates/pumpkin-core/src/entity/vehicle/boat.rs | RE-REVIEW-UNTRACKED | - |
| entities | Armor Stand | partial | crates/pumpkin-core/src/entity/decoration/armor_stand.rs | RE-REVIEW-UNTRACKED | - |
| entities | Cushion | partial | crates/pumpkin-core/src/entity/decoration/cushion.rs | RE-REVIEW-UNTRACKED | - |
| entities | Leash Knot | partial | crates/pumpkin-core/src/entity/decoration/leash_knot.rs | RE-REVIEW-UNTRACKED | - |
| entities | Painting | partial | crates/pumpkin-core/src/entity/decoration/painting.rs | RE-REVIEW | - |
| entities | Item | partial | crates/pumpkin-core/src/entity/item.rs | RE-REVIEW | - |
| entities | Ominous Item Spawner | partial | crates/pumpkin-core/src/entity/ominous_item_spawner.rs | RE-REVIEW-UNTRACKED | - |
| blocks | Beacon | partial | crates/pumpkin-core/src/block/blocks/beacon.rs | RE-REVIEW-UNTRACKED | - |
| blocks | Beds | partial | crates/pumpkin-core/src/block/blocks/bed.rs | RE-REVIEW | 4344:closed;3888:open;3581:open;3521:open |
| blocks | Beehive & Bee Nest | partial | crates/pumpkin-core/src/block/blocks/beehive.rs | RE-REVIEW-UNTRACKED | - |
| blocks | Campfire | partial | crates/pumpkin-core/src/block/blocks/campfire.rs | RE-REVIEW | 4178:open;3020:open |
| blocks | Chests | partial | crates/pumpkin-core/src/block/blocks/chests.rs | RE-REVIEW | 4288:open;4011:open;3902:open;3545:closed |
| blocks | Chiseled Bookshelf | partial | crates/pumpkin-core/src/block/blocks/chiseled_bookshelf.rs | RE-REVIEW | 3847:open;3646:open |
| blocks | Creaking Heart | partial | crates/pumpkin-core/src/block/blocks/creaking_heart.rs | RE-REVIEW-UNTRACKED | - |
| blocks | Jukebox | partial | crates/pumpkin-core/src/block/blocks/jukebox.rs | RE-REVIEW | 4128:open;3643:open;3548:open |
| blocks | Lodestone | partial | crates/pumpkin-core/src/item/items/compass.rs | RE-REVIEW-UNTRACKED | - |
| blocks | Note Block | partial | crates/pumpkin-core/src/block/blocks/note.rs | RE-REVIEW-UNTRACKED | - |
| blocks | Shulker Box | partial | crates/pumpkin-core/src/block/blocks/shulker_box.rs | RE-REVIEW | 4537:open;4341:closed |
| blocks | Structure Block | partial | crates/pumpkin-core/src/block/blocks/structure_block.rs | RE-REVIEW-UNTRACKED | - |
| blocks | Vault | partial | crates/pumpkin-core/src/block/blocks/vault.rs | RE-REVIEW | 3989:open |
| blocks | Fire | partial | crates/pumpkin-core/src/block/blocks/fire/fire.rs | RE-REVIEW | 4478:open;4451:open;3772:open;3659:open |
| blocks | Piston Head | partial | crates/pumpkin-core/src/block/blocks/piston/piston_head.rs | RE-REVIEW-UNTRACKED | - |
| blocks | Pistons | partial | crates/pumpkin-core/src/block/blocks/piston/piston.rs | RE-REVIEW | 4281:open;3900:open;3833:open;3144:open |
| blocks | Daylight Detector | partial | crates/pumpkin-core/src/block/blocks/redstone/daylight_detector.rs | RE-REVIEW-UNTRACKED | - |
| blocks | Dispenser | partial | crates/pumpkin-core/src/block/blocks/redstone/dispenser.rs | RE-REVIEW | 3972:open;3970:open;3822:open;3627:open;3518:open |
| items | Fishing Rod | partial | crates/pumpkin-core/src/item/items/fishing_rod.rs | RE-REVIEW | 3522:open;3051:open |
| items | Mace | partial | crates/pumpkin-core/src/item/items/mace.rs | RE-REVIEW-UNTRACKED | - |
| items | Compass & Recovery Compass | partial | crates/pumpkin-core/src/item/items/compass.rs | RE-REVIEW-UNTRACKED | - |
| items | Elytra | partial | crates/pumpkin-core/src/entity/player.rs | RE-REVIEW | 3899:open;3648:closed;3647:closed;3070:closed;1807:closed;1551:open |
| items | Goat Horn | partial | crates/pumpkin-core/src/item/items/goat_horn.rs | RE-REVIEW-UNTRACKED | 3113:closed |
| items | Name Tag | partial | crates/pumpkin-core/src/item/items/name_tag.rs | RE-REVIEW-UNTRACKED | - |

## 5. Infrastructure and infra items

1. **Parity ledger script.** `scripts/parity.sh`: download data.json, regenerate docs/PARITY.tsv (this file is its output, 564 rows), diff against the previous snapshot, and print done/total percent per category. Hourly refresh through scripts/sync-upstream.sh.
2. **Re-review audit workers.** One OpenCode worker per row in section 4 (87 rows). Output: concrete gaps with file:line, filed into section 2 or matched to an existing issue.
3. **Math parity harness.** Differential test of `log10` and `pow` (worldgen noise) against Java `StrictMath` golden vectors (a few thousand values). If Rust `f64::powf` or `log10` differ from fdlibm, port fdlibm `pow` and `log10` into pumpkin-world so output is identical on every CPU.
4. **Block-equality benchmark.** Generate N chunks for fixed seeds and compare block hashes with stored vanilla chunk dumps. Track percent against the 99.954% vanilla self-parity target. Add a `-Xint` vanilla dump as the reference (separates intrinsic rounding from algorithmic differences).
5. **Vanilla-bug registry.** docs/VANILLA_BUGS.md lists quirks we keep on purpose (short/int overflows and others), each with the vanilla class and method, so workers never "fix" them.
6. **1.8 combat mode design note.** Combat row 1-8-combat-mode is planned with no source identified (absence is not proof). Write a short design note (attack cooldown removed, sword block) with the vanilla 1.8 class references before any code; the row is DESIGN, not a gap.
7. **Stale tracker notes.** Dirt Path and Farmland rows still say push-up is TODO; ask the tracker maintainers to refresh after the next sync (note only).

## 6. Discord lessons (kept verbatim)

- **World generation determinism.** Vanilla Java output can differ by CPU: the JVM picks between Intel math stubs and fdlibm
  by CPU. Measured (Intel i5-13420H vs AMD Zen 1, 1M inputs per function, JDK 17/21/25): `log`, `exp`, `sin`, `cos`, `tan` identical;
  `log10` differs in 18 of 1M values, `pow` in 6 of 1M, each by 1 ulp. Impact: a block only changes where a noise value lands exactly
  on a threshold, which is extremely rare. Our chunks equal an AMD vanilla server; vanilla against itself on one seed is 99.954% block-equal.
  The remaining gap is not yet attributed (rounding versus other causes); a `-Xint` (interpreter-only) vanilla run separates
  intrinsic rounding from algorithmic differences.
- **Order of terrain generation matters a lot** (feature and decoration order decides tree placement and overlaps); trees can differ
  between vanilla runs depending on the direction a chunk is generated from. Keep decoration order identical to vanilla.
- **Pumpkin should replicate vanilla bugs**, including at least two short/int overflow bugs. Do not "fix" vanilla quirks silently.
- Sculk in deep dark was fixed by upstream PR #2882; anything else seen in the thread was filed as issues. Practice worth copying:
  AI audits find candidates, humans (here: us) file concrete issues, and the audit output is actually read.


