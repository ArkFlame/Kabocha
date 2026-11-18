# Parity rows for own-tentities-fox-195403

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/tentities-fox-195403** [partial/OWN-CLOSED] own audit: 0 open, 4 fixed, 1 verified, 0 blocked-data (docs/tracker/tentities-fox-195403.md) docs/tracker/tentities-fox-195403.md

## Result
- `own/tentities-fox-195403`: CLOSED. Every row in docs/tracker/tentities-fox-195403.md is resolved: 4 FIXED (sleep-onset water/rain in `mod.rs::tick_states`, `goals.rs` SleepGoal water/rain checks, one-in-ten sleep retry, commit `82261e6bf`) and 1 VERIFIED.
- Sole former STILL OPEN row, `mod.rs:877` (`get_ambient_sound`): VERIFIED (matches vanilla). Decompiled `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/fox/Fox.java:693` gates the screech on `!this.level().isBrightOutside() && this.random.nextFloat() < 0.1F`. Rust `mod.rs:877` uses `!world.is_bright_outside() && rand::rng().random_range(0..10) == 0` - same condition and same 10 percent chance. Nearby-player/spectator check (Fox.java:694-698 vs mod.rs:879-884) and screech volume/pitch (Fox.java:680-681 vs mod.rs:897-899) also match. The row's `isDay()` premise was incorrect; no code change warranted.
- THIRD-PASS re-verification (this pass): independently re-read `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/fox/Fox.java:678-702` and Rust `crates/pumpkin-core/src/entity/passive/fox/mod.rs:871-903`; parity holds. No source file needed an edit, so no code/test changes and no rustfmt this pass. No STILL OPEN rows remain.
