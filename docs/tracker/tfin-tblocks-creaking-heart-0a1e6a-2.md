# Final rows from docs/tracker/tblocks-creaking-heart-0a1e6a.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: creaking.rs creaking_heart.rs

| 5 hurt particle / emitter trail | STILL OPEN (client-only) | Vanilla `CreakingHeartBlockEntity.java:228-250` `creakingHurt` emits `TrailParticleOption` particles and seeds `emitter` so `CREAKING_HEART_HURT` replays 10x over 100 ticks (`:82-99`); all of that is client-side sound/particles. `creaking.rs:619-627` calls `creaking_hurt` only for a bound protector, and `creaking_heart.rs:221-245` plays the hurt sound and grows resin while awake. No server-observable divergence; client-only effects do not block. |
Re-checked every STILL OPEN row against `.vendor/vanilla-decompiled`. No source files needed changing.
| 4 creaking wander/melee goals | STILL OPEN (out of unit) | Vanilla `CreakingAi.java:30-85` uses a Brain/Activity/Behaviour system plus a 32-block-to-home path constraint (`Creaking.java:601-611`, `MAX_DISTANCE_TO_HOME_SQ=1024`). `creaking.rs:162-189` already registers Swim, canMove-gated Melee (interval 40) and Wander (0.3), LookAt(8), RandomLook and ActiveTarget, reproducing the observable gating/speed/interval. Full parity needs new goal/navigation types under `entity/ai/` (files outside this unit). |
Re-checked the sole STILL OPEN row (4) against decompiled vanilla (`CreakingAi.java:39-55,79-85`,

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
