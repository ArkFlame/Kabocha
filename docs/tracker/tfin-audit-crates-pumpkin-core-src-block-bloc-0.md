# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-block-bloc.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Re-checked every STILL OPEN row against .vendor/vanilla-decompiled (SnowyBlock.java,
Re-verified each STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/world/level/block/SnowyBlock.java`, `SpreadingSnowyBlock.java`, `GrassBlock.java`, `MyceliumBlock.java`, and `Blocks.java` (Podzol registration).
- `:121-150` (randomTick coverage): STILL OPEN, out of unit. Existing pure tests cover `can_stay_alive_with_above` (snow layer 1, full fluid, clear air, opaque cover). Full `random_tick` fade/spread behavior requires a `pumpkin-gametest` scenario with live world light/fluid/rng; no safe pure seam remains in these files.
Re-verified all four STILL OPEN rows against .vendor/vanilla-decompiled 1.21

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
