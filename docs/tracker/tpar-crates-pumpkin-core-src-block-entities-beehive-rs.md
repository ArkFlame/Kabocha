# Parity rows for crates/pumpkin-core/src/block/entities/beehive.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/tentities-bee-8270b4** [partial/OWN-OPEN] own audit: 4 open, 7 fixed, 3 verified, 0 blocked-data (docs/tracker/tentities-bee-8270b4.md) docs/tracker/tentities-bee-8270b4.md

## Result

Verified against .vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/BeehiveBlockEntity.java
and .vendor/vanilla-decompiled/net/minecraft/world/entity/animal/bee/Bee.java (MC 26.3). File edited:
crates/pumpkin-core/src/block/entities/beehive.rs. No cargo (worker restriction); the orchestrator runs
rustfmt and the build. No unwrap/expect/panic added.

- **own/tentities-bee-8270b4: FIXED.** The referenced audit (docs/tracker/tentities-bee-8270b4.md)
  already closes all six gaps as VERIFIED. This pass closed the two remaining in-scope flower-transfer
  parity gaps that were never implemented in beehive.rs: (1) add_occupant now adopts the entering
  bee's remembered flower_pos when the hive has none, otherwise on a 50% coin flip (vanilla addOccupant,
  BeehiveBlockEntity.java:190-192); (2) tick_occupants now gives a released bee with no remembered
  flower the hive's flower with 90% chance (vanilla releaseOccupant, BeehiveBlockEntity.java:251-253),
  writing the vanilla X/Y/Z compound that Bee.mob_read_nbt reads. The previously NBT-only flower_pos
  field now has its vanilla gameplay purpose. Added pure helpers adopts_flower, inherits_flower,
  position_from_compound with tests hive_adopts_flower_with_coin_flip_rules,
  released_bee_inherits_flower_only_when_eligible, reads_flower_position_compound. Hive occupancy
  (600/2400-tick stays, front-face respawn), hive-goal registration and goal priorities stay VERIFIED
  against the cited paths.

No STILL OPEN rows remain. No generated-data or whole-subsystem blocker.

### Final pass verification (2026-11-18)

Re-read the current beehive.rs and the vanilla source. Both flower-transfer rules cited above
were confirmed line-by-line in
.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/BeehiveBlockEntity.java:
addOccupant at line 190 (bee.hasSavedFlowerPos() && (!this.hasSavedFlowerPos() ||
this.level.getRandom().nextBoolean())) matches adopts_flower, and releaseOccupant at line
251 (savedFlowerPos != null && !bee.hasSavedFlowerPos() && random.nextFloat() < 0.9F) matches
inherits_flower. The single row own/tentities-bee-8270b4 is FIXED/VERIFIED, no code change
needed this pass.
