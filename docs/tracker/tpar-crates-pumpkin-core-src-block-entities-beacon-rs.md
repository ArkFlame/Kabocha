# Parity rows for crates/pumpkin-core/src/block/entities/beacon.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-bl-18946e** [partial/OWN-OPEN] own audit: 1 open, 3 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-bl-18946e.md) docs/tracker/taudit-crates-pumpkin-core-src-bl-18946e.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)


## Result (pass 3, 2026-11-18)

Re-verified the single OWN-OPEN row (`own/taudit-crates-pumpkin-core-src-bl-18946e`,
`docs/tracker/taudit-crates-pumpkin-core-src-bl-18946e.md`) against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/BeaconBlockEntity.java`
(MC 26.3, Paper-decompiled; Paper/CraftBukkit/Folia patches identified and stripped). The stale
"1 open" tally is resolved: all seven audit items are correct in the current tree, so no source
change was needed and rustfmt was not applicable.

- Row 1 (construct-beacon bounds / spectators): VERIFIED. `construct_beacon_bounds` (beacon.rs:302)
  reproduces vanilla `new AABB(x, y, z, x, y - 4, z).inflate(10, 5, 10)` (min `(x-10, y-9, z-10)`,
  max `(x+10, y+5, z+10)`); BeaconBlockEntity.java:243 uses `getEntitiesOfClass(ServerPlayer.class, ...)`
  which filters spectators via `EntitySelector.NO_SPECTATORS`, and `trigger_construct_beacon` skips them
  (beacon.rs:321). Test `construct_beacon_bounds_inflate_matches_vanilla`.
- Row 2 (beam gating): VERIFIED. `tick` gates `update_base` and `apply_effects`/`BlockBeaconAmbient`
  on the committed-scan `beam_active` snapshot inside the `% 80` branch (beacon.rs:488-504), matching
  BeaconBlockEntity.java:214-222 (`!entity.beamSections.isEmpty()`).
- Row 3 (beam obstruction): VERIFIED. `opacity >= 15` plus the `BEDROCK` exception (beacon.rs:466-472)
  matches `state.getLightDampening() >= 15 && !state.is(Blocks.BEDROCK)` (BeaconBlockEntity.java:200);
  `BlockState.opacity` is vanilla light dampening. Stained glass panes implement `BeaconBeamBlock`
  (StainedGlassPaneBlock.java:6), so `is_beam_block` is correct.
- Row 4 (pyramid base): VERIFIED. `update_base` (beacon.rs:231-252) matches `updateBase`
  (BeaconBlockEntity.java:253-279): `BEACON_BASE_BLOCKS` tag, `ly < min_y` break, `(2*step+1)^2` scan.
- Row 5 (effect flags / spectators): VERIFIED. Duration `(9+2*levels)*20`, range `levels*10+10`,
  amplifier 1 only at level 4 when primary == secondary, secondary only at level 4 when different
  (beacon.rs:262-297) match `applyEffects` (BeaconBlockEntity.java:323-385); `apply_effects` skips
  spectators like `getHumansInRange`'s `NO_SPECTATORS` filter (:364-380).
- Row 6 (NBT shape): VERIFIED. `storeEffect`/`loadEffect` (beacon.rs:331-342, 394-423) write/read effect
  resource-location strings only when set; `Levels` is written but not loaded, matching vanilla
  `saveAdditional`/`loadAdditional` (BeaconBlockEntity.java:421-450); the `getIntOr("Levels",0)` line is
  CraftBukkit SPIGOT-5053, not vanilla. Test `nbt_keeps_effects_and_drops_levels_on_load`.
- Row 7 (on_block_replaced): VERIFIED (client-only sound). Unconditional `BlockBeaconDeactivate`
  (beacon.rs:531-534) is the Spigot lineage; the conditional form at BeaconBlockEntity.java:287-291 is
  Paper's MC-153086 patch, so no vanilla-unpatched server behaviour differs. Client-only sound: no block.

No BLOCKED-DATA rows: every required field exists in generated data. Pure tests already pin the bounds,
beam sections, colour averaging and NBT round trip; no new pure seam exists (the gating call sites need a
live `World` with joined players). Files changed: none (verification only).
