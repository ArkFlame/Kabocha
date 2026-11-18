# Final rows from docs/tracker/tpar-own-tentities-fox-195403.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: mod.rs

- Sole former STILL OPEN row, `mod.rs:877` (`get_ambient_sound`): VERIFIED (matches vanilla). Decompiled `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/fox/Fox.java:693` gates the screech on `!this.level().isBrightOutside() && this.random.nextFloat() < 0.1F`. Rust `mod.rs:877` uses `!world.is_bright_outside() && rand::rng().random_range(0..10) == 0` - same condition and same 10 percent chance. Nearby-player/spectator check (Fox.java:694-698 vs mod.rs:879-884) and screech volume/pitch (Fox.java:680-681 vs mod.rs:897-899) also match. The row's `isDay()` premise was incorrect; no code change warranted.
- THIRD-PASS re-verification (this pass): independently re-read `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/fox/Fox.java:678-702` and Rust `crates/pumpkin-core/src/entity/passive/fox/mod.rs:871-903`; parity holds. No source file needed an edit, so no code/test changes and no rustfmt this pass. No STILL OPEN rows remain.

## Result
VERIFIED: `get_ambient_sound` matches `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/fox/Fox.java:678-702`. No open rows.
