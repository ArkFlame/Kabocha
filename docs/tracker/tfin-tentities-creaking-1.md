# Final rows from docs/tracker/tentities-creaking.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- death sound ordering (was STILL OPEN): verified against decompiled vanilla, NO code change. `Creaking.creakingDeathEffects`
- unused eye-glow fields (was STILL OPEN): LEFT OPEN. Client-only effect: vanilla runs `setupAnimationStates` /
- `is_looking_at_me` gaze cone (was STILL OPEN): already FIXED in code; re-verified. Vanilla
- death sound ordering (was STILL OPEN): verified NO change. `creakingDeathEffects` -> `die()` broadcasts

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
