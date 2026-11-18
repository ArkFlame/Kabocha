# Native agent 5: triage of 33 open parity rows

Source list: `docs/investigations/work/native-5.md` (33 lines, each cited to its unit note).

Status: no code changed in this pass. Every row was triaged against its unit note. None was closed with a compile-safe fix inside this pass, so no commits were made.

## Counts
- FIXED: 0
- NOT A GAP: 1 (row 4 of `taudit-crates-pumpkin-core-src-en-aadca5`: the note's own evidence shows `AbstractSkeleton.java:71-74` has no lava malus, so the row's premise is false. Its `WitherSkeleton.java` is where the malus lives.)
- BLOCKED or DEFERRED: 32

## Rows by blocker
- Needs a Brain/Behaviour port (large subsystem): creaking wander/melee goals and 32-block home constraint (`tblocks-creaking-heart-0a1e6a`, Creaking.java:601-611), brain-sensor threats reuse of goal path logic (`taudit-...-d76c79`).
- Needs a new navigation hook, cross-file: strider lava navigation (`tentities-strider`), skeleton path malus (see NOT A GAP above).
- Cross-crate refactor: skeleton equipment roll ignoring spawn reason (`taudit-...-a97307`, needs EntitySpawnReason threaded through every finalize_spawn caller).
- Generated data off-limits: beds dimension row 2 (`pumpkin-data/src/generated/dimension.rs`), creaking loot tables (data-driven).
- Panda AI island and `has_audited_tick` flip, needs `panda/goals.rs` (outside the unit): `taudit-...-7bf7cf` row 4, `taudit-...-entity-pas` row 5, `taudit-...-3b1660` row 1 (random source from entity-owned `panda.random`).
- Lives in `server/mod.rs` (outside unit): `se-1269c5` rows 2, 3, 4, 5, 7, 8; `server-mod` rows 2, 4.
- Lives in `net/mod.rs` (outside unit): `ne-cbdd30` rows 4, 5, 6.
- Lives in `entity/mod.rs` or `entity/player.rs`: bamboo chest raft fluid current push (`tentities-bamboo-chest-raft` #6), compass lodestone tick (`titems-compass-recovery-compass` row 1).
- Client-only or purely visual, no server parity change: creaking eye-glow fields, vault display item event, nautilus riding loop sound, creaking hurt particles.
- Design decision or tradeoff: wo-ea95b7 gap 3 (early return when no player/arena).
- Needs a new test file or gametest not in the unit: oak boat gametest, compass row 5.
- Plugin API additive change: `tmedium-batch1` row 3 (`BlockFadeEvent` new-state field).
- Other out-of-unit or unverified: world-expl row 6 (`ClientboundExplodePacket` blockCount in packet construction), world-time row 5 (`/time` marker and clock registry in `command/commands/time.rs`), oak boat air-to-water snap (`noCollision` guard).

## Why nothing was implemented
Each remaining row needs edits across several files or crates plus a compile to check. Workers were told not to compile, and the mob-AI rules (Brain-free goal `tick_reach`, `has_audited_tick` only after a full species audit) need a dedicated pass. Landing unverified cross-file changes here would add compile risk without a check.

## Recommended next step
Split by blocker class into separate briefs: (1) the panda audit, `panda/goals.rs` plus `has_audited_tick`; (2) `Mob::create_navigation` hook for strider; (3) `server/mod.rs` tick and forced-chunk rows as one batch; (4) `net/mod.rs` rows as one batch. The Brain port for creaking should be its own large task.

Risk for the compile: none from this pass, since no `.rs` file was touched.
