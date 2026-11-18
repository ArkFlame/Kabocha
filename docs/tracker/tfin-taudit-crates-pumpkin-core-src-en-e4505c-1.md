# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-e4505c.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- 6 death entity status: STILL OPEN, XS.
Verified each STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/throwableitemprojectile/ThrownEnderpearl.java` (Paper/Folia mappings).
- Row 1 (portal particle distribution): STILL OPEN, cosmetic. Vanilla uses `nextGaussian()` for the X/Z velocity and no X/Z position spread; rand 0.10.3 core has no gaussian distribution and `rand_distr` is not a dependency, so a faithful fix needs a Cargo.toml change / shared helper outside this unit. Client-only effect, noted not fixed.
- Row 3 (cancellable dismount events): STILL OPEN, intentional Pumpkin extension. Vanilla `unRide` is uncancellable; preserving Pumpkin's existing event contract, no change.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
