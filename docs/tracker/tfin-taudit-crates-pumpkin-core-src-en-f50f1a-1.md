# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-f50f1a.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: ai.rs

| Play-dead regeneration duration | `ai.rs:39,353-364` grants 200 ticks at amplifier 0 | wiki states Regeneration I while playing dead; duration is not documented in code we can read here | keep amplifier 0 (wiki: Regeneration I); only shorten to 100 ticks if 1.21.x source confirms | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
- All STILL OPEN rows resolved and verified against decompiled vanilla; see ## Result below.

## Result

- Row 5 (play-dead regeneration duration): VERIFIED, matches decompiled vanilla, no change. `PlayDead.start` grants `new MobEffectInstance(MobEffects.REGENERATION, 200, 0)` (200 ticks, amplifier 0) at `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/axolotl/PlayDead.java:32`. Pumpkin matches: `PLAY_DEAD_REGENERATION_DURATION: i32 = 200` (`axolotl/ai.rs:40`) and `start()` applies `Effect { effect_type: &StatusEffect::REGENERATION, duration: PLAY_DEAD_REGENERATION_DURATION, amplifier: 0, .. }` (`axolotl/ai.rs:356-364`). The wiki statement documented only Regeneration I without a duration; the decompiled source confirms 200 ticks at amplifier 0, so the constant stays at 200. No source files touched (no rustfmt needed).
