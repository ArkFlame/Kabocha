# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-e4505c.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: mod.rs

- Row 3 (cancellable dismount events, S): STILL OPEN, intentional, no change. Vanilla `unRide(true)` (line 101) is uncancellable. Making Pumpkin's dismount uncancellable requires changing `remove_passenger_before_teleport` / the plugin event contract (Entity and plugin event files outside this unit), so it is left open per the prior design decision.
Re-verified every STILL OPEN row against
- Row 3 (cancellable dismount events, S): STILL OPEN, left open per task rule. Vanilla `entity.unRide(true)` (line 101) is uncancellable; Pumpkin's `remove_passenger_before_teleport` fires cancellable `EntityDismountEvent`/`VehicleExitEvent` from `entity/mod.rs:6384-6405`. Making it uncancellable requires editing `entity/mod.rs` and the plugin event contract, files outside this unit.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
