# Final rows from docs/tracker/tblocks-creaking-heart-0a1e6a.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

| 5 hurt particle / `setChanged` | STILL OPEN (client-only, out of unit) | Vanilla `creakingHurt` only emits client particles and seeds an emitter; no server divergence. |
Re-checked the three STILL OPEN rows against `.vendor/vanilla-decompiled`. One small in-unit parity
Re-checked every previously STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/`.

## Result (final pass - all rows closed)

Re-checked the sole STILL OPEN row against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/`. No source change was needed: the row was
already fully matched and the original `setChanged` claim was stale.

| row | terminal outcome | evidence (.vendor path + Pumpkin code) |
|---|---|---|
| 5 hurt particle / `setChanged` | VERIFIED (no-op, client-only) | `world/level/block/entity/CreakingHeartBlockEntity.java:228-250` `creakingHurt` guards on a bound creaking and `emitter <= 0`, emits only client `TrailParticleOption` particles through `emitParticles` (`:306-307` `serverLevel.sendParticles`, no state change), and seeds the client sound-emitter fields (`emitter = 100`, `emitterTarget`, replayed at `:82-99`). It NEVER calls `setChanged` (the calls at `:157/:162/:168` are in unrelated methods), so the tracker claim was stale. All server-observable parts (2-3 resin clumps via `spreadResin`, `RESIN_PLACE` sound, `BLOCK_PLACE` game event, only while AWAKE) are already reproduced in `crates/pumpkin-core/src/block/entities/creaking_heart.rs:256-280` `creaking_hurt`. No server-observable divergence; nothing to fix. |

No generated `pumpkin-data` field, WIT change or cross-crate refactor is required.
No source file was changed in this pass. Not compiled in this role; orchestrator batch check.
