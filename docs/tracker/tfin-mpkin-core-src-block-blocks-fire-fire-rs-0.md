# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-block-blocks-fire-fire-rs-0.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Final pass (2026-11-18). All rows closed, none left STILL OPEN.

## Result

- **fire tick timer (entity combustion, apply_fire_collision)**: VERIFIED against decompiled vanilla. `crates/pumpkin-core/src/block/blocks/fire/mod.rs:124-151` (`FireBlockBase::apply_fire_collision`) matches `.vendor/vanilla-decompiled/net/minecraft/world/level/block/BaseFireBlock.java:142-166` (`fireIgnite`): immunity guard, `ticks < 0` then `+1`, else `PLAYER` then random `1..3` added, `ticks >= 0` then set on fire for 8.0s. Fire vs soul fire damage (1.0 / 2.0, IN_FIRE) matches `entityInside` at BaseFireBlock.java:127-140; callers fire.rs:277 and soul_fire.rs:32 pass the correct flag.

- **fire block scheduled tick delay (get_fire_tick_delay)**: VERIFIED against decompiled vanilla. `crates/pumpkin-core/src/block/blocks/fire/fire.rs:34-35` returns `30 + rand(0..10)`, matching `.vendor/vanilla-decompiled/net/minecraft/world/level/block/FireBlock.java:336-341` (`getFireTickDelay` = `fireTickDelay` default 30 + `nextInt(10)`). Used at fire.rs:271 (`onPlace`) and fire.rs:318 (`onScheduledTick`), both `TickPriority::Normal`.

No source file needed editing (both timers already match vanilla). No generated pumpkin-data field or subsystem is missing, so BLOCKED-DATA: none. No pure unit test seam exists for either timer (each needs `World`, entity and event firing). Compile left to the orchestrator; no cargo was run by this worker.
