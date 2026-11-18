# Final rows from docs/tracker/tentities-bee-8270b4.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

No STILL OPEN rows remain; no generated-data or whole-subsystem blocker. No unwrap/expect/panic
(MC 26.3), and re-read the current source. No STILL OPEN rows existed on entry and none remain;
No row remains STILL OPEN. No BLOCKED-DATA: no missing generated `pumpkin-data` field and no

## Result

FINAL PASS (2026-11-18). Every row re-verified against the decompiled vanilla source
`.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/bee/Bee.java` and
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/BeehiveBlockEntity.java`
(MC 26.3) and against the current Pumpkin source. No source file needed an edit this pass: the six
rows were FIXED in earlier passes and remain VERIFIED. No row is STILL OPEN, no BLOCKED-DATA, and
no unwrap/expect/panic was added.

- **Gap 1 (hive behaviour): VERIFIED.** Vanilla `Bee.java:360-369, 769-782, 1051-1130` and the
  `BeehiveBlockEntity.java` occupant API. `bee.rs` persists `hive_pos` as the vanilla `X`/`Y`/`Z`
  compound and decrements `cannot_enter_hive_ticks` each server tick; `block/entities/beehive.rs`
  `add_occupant`/`tick_occupants` respawn from the front face after 600/2400 ticks; `goal/hive.rs`
  registers EnterHive/LocateHive/GoToHive at vanilla priorities 1/5/5.
- **Gap 2 (flower memory / pollination): VERIFIED.** Vanilla `Bee.java:360-369, 693-697,
  1129-1188`. `bee.rs` stores `flower_pos`, resets `ticks_without_nectar` on nectar gain and at
  pollination start, and folds `is_pollinating` into `wants_to_enter_hive_gate`;
  `goal/pollinate_flower.rs` uses `MINECRAFT_BEE_ATTRACTIVE`, a +/-5 Manhattan search, strict
  `> 400` pollination, the 20% hover roll, once-only nectar in `stop()`, waterlogged and
  lower-sunflower filtering, and `World::is_raining()` stops in both `can_start`/`should_continue`.
- **Gap 3 (crop growth): VERIFIED.** Vanilla `Bee.java:978-1030`. `goal/grow_crop.rs` priority 7,
  30% `canBeeUse` reject, 1-in-30 `tick` roll, `isHiveValid` gate, age+1 growth, ten-crop cap that
  keeps nectar, and `PITCHER_CROP` exclusion (vanilla has no `PitcherCropBlock` branch) all match.
- **Gap 4 (hive exit / TicksSincePollination): VERIFIED.** Vanilla `Bee.java:346-348, 408-410,
  491-495, 519-525`. The counter increments while nectar-free and resets on nectar gain and at
  pollination start; `cannot_enter_hive_ticks` decrements each tick and the tired-at-3600 return is
  driven by the hive goals.
- **Gap 5 (goal priorities): VERIFIED.** Vanilla `Bee.java:192-213`. `bee.rs` registers 0 attack,
  1 enter-hive, 2 breed, 3 tempt/validate, 4 pollinate, 5 follow/locate/go-hive, 6 go-to-flower,
  7 grow-crop, 8 wander, 9 float.
- **Gap 6 (spawn cluster / rain): VERIFIED.** Vanilla `Bee.java` has no per-species
  `getMaxSpawnClusterSize`/`isDarkEnoughToSpawn` override; rain avoidance lives in
  `pollinate_flower.rs` via `World::is_raining()` (`Bee.java:1134, :1154`).

No code edits, no generated-data or whole-subsystem blocker, no new unwrap/expect/panic. Source
files verified unchanged this pass: `entity/passive/bee.rs`, `entity/ai/goal/pollinate_flower.rs`,
`entity/ai/goal/grow_crop.rs`, `block/entities/beehive.rs`, `entity/ai/goal/hive.rs`.
