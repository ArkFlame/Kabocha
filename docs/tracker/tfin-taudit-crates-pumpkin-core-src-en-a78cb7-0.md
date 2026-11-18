# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-a78cb7.md

Files: parrot.rs

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files).

## Result (batch 09 pass, 2026-11-18)

- 1 HAPPY_GHAST EMPTY sound (parrot.rs IMITATIONS table): VERIFIED. Parrot.java:99 maps HAPPY_GHAST to SoundEvents.EMPTY; parrot.rs maps it to Sound::IntentionallyEmpty, and the ambient candidate test ambient_imitation_includes_the_happy_ghast_silent_entry covers it.
- 2 server-side flap/descent damping (parrot.rs mob_ai_step): VERIFIED. Parrot.java:217-231 calculateFlapping: flapSpeed step (+4 airborne not passenger, -1 otherwise) times 0.3, clamped 0..1, and downward movement scaled by 0.6 in y when airborne. parrot.rs matches, including the has_vehicle case. The flap/flapping accumulators are client animation only.

No .rs file changed in this pass.
