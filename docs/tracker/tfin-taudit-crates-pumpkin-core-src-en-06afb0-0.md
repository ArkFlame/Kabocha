# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-06afb0.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: living.rs

- 1. add_effect instance update: FIXED (verified this pass).
- 2. Negative heal without die: FIXED (verified this pass).
- 3. Fall damage prevention FIXME: FIXED (verified this pass).
- 4. Effective gravity below world: FIXED (verified this pass).

## Result

All four rows were implemented in earlier passes; this pass re-verified each against decompiled vanilla and confirmed no TODO/FIXME/open markers remain in living.rs.

- 1. add_effect instance update: FIXED. add_effect now merges an existing stored instance via update_effect (mirrors MobEffectInstance.update, including stronger-amplifier takeover, equal-amplifier duration extension, infinite duration, ambient/particles/icon), resends the effect only when the stored instance actually changed, applies attribute modifiers with the merged amplifier, runs onEffectStarted-style logic (absorption/invisibility/glowing/sound) with the incoming amplifier, and never stores instant effects as active. Unit tests in effect_update_tests cover weaker/shorter never replacing, stronger takeover/equal extension, and infinite duration. Vanilla: .vendor/vanilla-decompiled/net/minecraft/world/entity/LivingEntity.java addEffect (lines 1223-1274) and .vendor/vanilla-decompiled/net/minecraft/world/effect/MobEffectInstance.java update (lines 133-158).
- 2. Negative heal without die: FIXED. heal returns early for dead entities or zero amount, fires the regain event, uses the event possibly-modified amount, and re-checks health before set_health, which clamps to [0, max_health] so a negative heal reduces health to 0 without ever calling die. Vanilla: .vendor/vanilla-decompiled/net/minecraft/world/entity/LivingEntity.java heal (lines 1430-1455) and setHealth clamp (line 1467).
- 3. Fall damage prevention: FIXED. is_in_fall_damage_resetting checks the block under the feet for the minecraft:fall_damage_resetting tag; should_prevent_fall_damage handles scaffolding (only while sneaking), water, and the player end gateway / end portal / nether portal (nether portal gated by players_nether_portal_default_delay == 0); should_prevent_fall_damage_in_area checks the single landed-on block state (cobweb/ladder/powder snow/slime block + tag), not a radius scan. Vanilla: .vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java fall-damage reset (line 2285) and resetFallDistance on water (line 2368), plus .vendor/vanilla-decompiled/net/minecraft/world/entity/LivingEntity.java checkFallDamage (lines 424-463).
- 4. Effective gravity below world: FIXED. get_effective_gravity returns 0 when has_no_gravity, otherwise caller.get_gravity() capped at 0.01 while descending with slow falling; travel_in_air always applies gravity server-side (the vanilla hasChunkAt fallback is client-only, and Pumpkin has no chunkless travel). Vanilla: .vendor/vanilla-decompiled/net/minecraft/world/entity/LivingEntity.java getEffectiveGravity (lines 3129-3131) and getDefaultGravity (lines 3123-3126), and .vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java getGravity (lines 2231-2233).
