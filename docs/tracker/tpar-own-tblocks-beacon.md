# Parity rows for own-tblocks-beacon

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/tblocks-beacon** [partial/OWN-OPEN] own audit: 12 open, 2 fixed, 15 verified, 0 blocked-data (docs/tracker/tblocks-beacon.md) docs/tracker/tblocks-beacon.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)


## Result (third pass, final closure, 2026-11-18)

Re-checked every STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/`. No Pumpkin code change was needed; all four rows are at parity and the ledger table in `tblocks-beacon.md` has been updated so no row remains STILL OPEN.

- Row 36 (beam sections / client beam): VERIFIED no change. The server-side model matches vanilla `BeaconBlockEntity.tick` (BeaconBlockEntity.java:166-211) and `getBeamSections` (line 406-409): `append_beam_block` pushes when `size <= 1`, grows on equal colour, averages on colour change (`ARGB.average` is per-channel midpoint, equal to `u32::midpoint`), and clears on opaque non-bedrock; `get_beam_sections()` returns empty at level 0. The vanilla CLIENT runs the same scan itself (`BeaconBlock.getTicker` registers `BeaconBlockEntity::tick` with no client-side guard, BeaconBlock.java:34-37) and `getUpdateTag` sends only `saveCustomOnly` (BeaconBlockEntity.java:416-419), so no beam data crosses the network. A client `BeaconRenderer` is a vanilla-client subsystem absent from this server; there is no server-side gap to fix. Existing pure tests (merge / average / cut / average-math / beam colour) cover the seam.
- Row 37 (spectator filter): VERIFIED no change; `is_spectator()` skips stay. `EntityGetter.getEntitiesOfClass(Class, AABB)` defaults to `EntitySelector.NO_SPECTATORS` (EntityGetter.java:79-81; EntitySelector.java:17 `= !entity.isSpectator()`), used by `getHumansInRange` for ranges <= 128 (BeaconBlockEntity.java:369-370; beacon range caps at 50) and by the construct-beacon loop (line 243). The table's original "drop the checks" fix was wrong: removing them would break parity.
- Row 38 (deferred BEACON_POWER_SELECT sound): VERIFIED already fixed. `set_property(DATA_PRIMARY)` plays the sound immediately via the `OnceLock<Weak<World>>` populated by `set_world` (called in `World::add_block_entity`, world/mod.rs:8491), matching `dataAccess.set` case 1 (BeaconBlockEntity.java:95-101). No `play_select_sound` flag remains. No pure unit-test seam (needs a live world).
- Row 39 (stale Levels while beam blocked): VERIFIED no change. Vanilla gates `updateBase` on `!beamSections.isEmpty()` (BeaconBlockEntity.java:215); Pumpkin gates on `beam_active`, committed from the same scan as `beam.sections`, so parity holds.
- Row 40: not STILL OPEN; documented upstream-vs-Paper difference (`Levels` set in the Paper-patched `loadAdditional`, BeaconBlockEntity.java:436, intentionally not loaded by Pumpkin), out of scope.
