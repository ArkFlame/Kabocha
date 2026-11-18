# Final rows from docs/tracker/tentities-strider.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: mod.rs strider.rs

- Row `strider.rs:178` passenger attachment: STILL OPEN. UNVERIFIED offset, no vanilla source in tree.
- STILL OPEN `strider.rs:35` navigation: needs a per-mob `create_navigation` hook (Mob trait) plus `AmphibiousNodeEvaluator` wiring in `entity/ai/pathfinder/mod.rs`, both outside this unit's files.
- STILL OPEN `strider.rs:96` jockey spawn: needs a `finalizeSpawn` hook, `SpawnGroupData` baby-chance override and spawning separate jockey entities (zombified piglin / baby strider), cross-cutting spawn infrastructure outside this unit.
- STILL OPEN `strider.rs:178` passenger attachment: vanilla applies the rider offset only when `level().isClientSide()`, i.e. client-only visual effect; no server behaviour to port here.

## Result
## Result

Final pass. All four open rows verified against `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Strider.java`; no source change needed, no BLOCKED-DATA.

- Row `strider.rs:178` passenger attachment (both entries): VERIFIED no server change. `Strider.getPassengerAttachmentPoint` (Strider.java:190-199) returns `super` immediately when `!level().isClientSide()`; the `0.12F * cos(animPos*1.5F) * 2 * animSpeed` rider bob is added only on the client. Pumpkin is authoritative server-side, so there is no offset to port.
- Row `strider.rs:35` navigation: VERIFIED no server change. `Strider.createNavigation` (Strider.java:388-389) returns `Strider.StriderPathNavigation extends GroundPathNavigation` whose `createPathFinder` installs a `WalkNodeEvaluator` (Strider.java:523-532), i.e. Pumpkin's default ground navigator. The proposed `Mob::create_navigation` hook + `AmphibiousNodeEvaluator` would be a parity regression. Every vanilla extra is already in tree: constructor maluses WATER -1.0 / LAVA 0.0 / FIRE_IN_NEIGHBOR 0.0 / FIRE 0.0 (Strider.java:93-100) = `strider.rs:60-63` (`DangerFire`/`DamageFire`), `isStableDestination` lava-stable (Strider.java:534-537) = `strider.rs:255` `is_stable_destination`, `getWalkTargetValue` seeks lava (Strider.java:393-399) = `strider.rs:267`.
- Row `strider.rs:96` jockey spawn: FIXED (already in tree at `strider.rs:309-386`, commit d188984b3), verified against `Strider.finalizeSpawn` / `spawnJockey` (Strider.java:451-487): 1/30 zombified piglin when monsters can spawn (0.05 baby odds, warped fungus on a stick + saddle + guaranteed drop), 1/10 baby strider (age -24000), else 0.5 baby chance; each jockey branch zeroes the group's baby chance. All helper methods confirmed present.
- No pure unit-test seam (navigation/world/RNG/client-only/plugin-driven); skipped per MVP test rules.
- No `.rs` edits this pass; no rustfmt needed.

Status: PASS fixed=0 open=0 files=docs/tracker/tfin-tentities-strider-1.md
