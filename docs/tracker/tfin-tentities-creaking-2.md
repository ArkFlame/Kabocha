# Final rows from docs/tracker/tentities-creaking.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: creaking.rs

- unused eye-glow fields (was STILL OPEN): VERIFIED. Client-only: vanilla runs `setupAnimationStates`/`checkEyeBlink` only when `level().isClientSide()` (Creaking.java:262-265); server `creaking.rs` has no eye-glow fields/methods.
- row 25 `is_looking_at_me` (h2 STILL OPEN): FIXED+VERIFIED. Rust now raycasts LOS per gaze height (eye/feet/mid) with ClipBlock::Visual and 128-block cap, matching `LivingEntity.isLookingAtMe` (LivingEntity.java:2276-2299) and `Creaking.checkCanMove` (Creaking.java:475-478, coneSize 0.5, adjustForDistance=false). LOS endpoint corrected from eye-to-eye to per-gaze-height point.
- row 30 death-sound ordering (h2 STILL OPEN): verified NO change. `creakingDeathEffects` -> `die()`
- row 31 unused eye-glow fields (h2 STILL OPEN): already FIXED; fields/methods absent from `creaking.rs`.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
