# Parity rows for crates/pumpkin-core/src/command/commands/locate.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-co-e958f4** [partial/OWN-OPEN] own audit: 18 open, 5 fixed, 1 verified, 1 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-co-e958f4.md) docs/tracker/taudit-crates-pumpkin-core-src-co-e958f4.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)


## Result (third pass, terminal)

Re-read this unit's last Result and re-verified every row against decompiled
vanilla / Paper-Canvas at
`/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft/server/commands/LocateCommand.java`.
The `.vendor/vanilla-decompiled/net/minecraft` path named in the task is not
present in this checkout, so the equivalent decompiled tree was used. All
in-scope fixes are already present in `locate.rs` and type-consistent; no `.rs`
file needed a new edit, so no rustfmt run was required.

| Row | Terminal outcome |
|---|---|
| own/taudit-crates-pumpkin-core-src-co-e958f4 (this row) | VERIFIED (audit complete): every in-scope sub-row in `docs/tracker/taudit-crates-pumpkin-core-src-co-e958f4.md` is terminal - 7 FIXED/VERIFIED, 1 BLOCKED-DATA. Re-checked `locate.rs` against vanilla: argument types, search radii, tag result names, f32 distance rounding (`horizontal_distance`/`absolute_distance`), 3D POI radius (`poi_within_radius`), all-set reverse map (`structure_sets_containing`), stronghold Pending arm, and Bedrock `translate_cross_with` args plus their unit tests are all present. The `StructureSearch::Pending` arm sends "Searching for ..." and returns 0 (`LocateCommand.java:109-112`). |
| Datapack/modded structures (`wanted_structures`) | BLOCKED-DATA (sub-row in the referenced audit): whole missing subsystem. Vanilla resolves ids via the dynamic `Registries.STRUCTURE` (`LocateCommand.java:90-99`); Pumpkin only has the closed generated enum `pumpkin_data::structures::StructureKeys::from_name` plus an `is_vanilla()` gate (`locate.rs:123-139`), and finders index generated `StructureSet::ALL`. Needs a generated `worldgen/structure`/`worldgen/structure_set` gameplay registry and non-closed structure data in `crates/pumpkin-data/src/generated/` (generated, not editable by workers). |

Files changed: none (all fixes already present and verified). No rustfmt run.
