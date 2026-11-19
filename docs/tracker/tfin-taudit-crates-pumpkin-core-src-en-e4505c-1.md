# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-e4505c.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- 6 death entity status: STILL OPEN, XS.
Verified each STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/throwableitemprojectile/ThrownEnderpearl.java` (Paper/Folia mappings).
- Row 1 (portal particle distribution): FIXED (stale note). `crates/pumpkin-core/src/entity/projectile/ender_pearl.rs:25-32` has a Box-Muller `next_gaussian()` with no new dependency, and `on_hit` (`:96-114`) spawns the 32 PORTAL particles at the pearl position with a 0..2 Y offset and gaussian X/Z velocity, matching `ThrownEnderpearl.java:165-175`. Test `gaussian_is_standard_normal` in the same file. No change this pass.
- Row 4 (connection-open gate, from e4505c-0): VERIFIED. `ender_pearl.rs:142` rejects a player owner whose `client.closed()` is true, matching `ThrownEnderpearl.java:199` `isAcceptingMessages()`.
- Row 3 (cancellable dismount events): STILL OPEN, intentional Pumpkin extension. Vanilla `unRide` is uncancellable; preserving Pumpkin's existing event contract, no change.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
