# Final rows from docs/tracker/tentities-creaking.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: creaking.rs

- unused eye-glow fields (was STILL OPEN): LEFT OPEN. `setupAnimationStates`/`checkEyeBlink` run only when
- row 25 `is_looking_at_me` (h2 STILL OPEN): already FIXED. Vanilla `LivingEntity.isLookingAtMe` (2276-2299)
- row 30 death-sound ordering (h2 STILL OPEN): verified NO change. `creakingDeathEffects` -> `die()`
- row 31 unused eye-glow fields (h2 STILL OPEN): already FIXED; fields/methods absent from `creaking.rs`.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
