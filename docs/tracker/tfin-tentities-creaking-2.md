# Final rows from docs/tracker/tentities-creaking.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: creaking.rs

- unused eye-glow fields (was STILL OPEN): VERIFIED. Client-only: vanilla runs `setupAnimationStates`/`checkEyeBlink` only when `level().isClientSide()` (Creaking.java:262-265); server `creaking.rs` has no eye-glow fields/methods.
- row 25 `is_looking_at_me` (h2 STILL OPEN): FIXED+VERIFIED. Rust now raycasts LOS per gaze height (eye/feet/mid) with ClipBlock::Visual and 128-block cap, matching `LivingEntity.isLookingAtMe` (LivingEntity.java:2276-2299) and `Creaking.checkCanMove` (Creaking.java:475-478, coneSize 0.5, adjustForDistance=false). LOS endpoint corrected from eye-to-eye to per-gaze-height point.
- row 30 death-sound ordering (h2 STILL OPEN): VERIFIED NO change. `creakingDeathEffects` -> `die()` broadcasts EntityEvent.DEATH (client plays CREAKING_DEATH) then CREAKING_TWITCH; `tearDown` (Creaking.java:292-327) plays `getDeathSound()` again. Vanilla intentionally plays CREAKING_DEATH twice; Rust `on_die`/`creaking_death_effects`/`tear_down` matches. (Creaking.java:292-333)
- row 31 unused eye-glow fields (h2 STILL OPEN): VERIFIED. Fields/methods (`eyesGlowing`, `nextFlickerTime`, `hasGlowingEyes`, `checkEyeBlink`) absent from `creaking.rs`; vanilla keeps them client-only (Creaking.java:79-81, 411-421, 262-265).

## Result
All four rows resolved; no BLOCKED-DATA.

- unused eye-glow fields: VERIFIED (Creaking.java:262-265 client-only; no server fields).
- row 25 `is_looking_at_me`: FIXED+VERIFIED (LivingEntity.java:2276-2299; Creaking.java:475-478). LOS now per-gaze-height with ClipBlock::Visual. No pure unit test (needs world/player state).
- row 30 death-sound ordering: VERIFIED NO change (Creaking.java:292-333).
- row 31 unused eye-glow fields: VERIFIED (Creaking.java:79-81, 411-421, 262-265).

Files edited: crates/pumpkin-core/src/entity/mob/creaking.rs (is_looking_at_me LOS per gaze height + level check). UNVERIFIED (no compile in this role).
