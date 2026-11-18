# Parity rows for crates/pumpkin-core/src/entity/mob/skeleton/stray.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Stray** [partial/COVERED-DONE] The shared bow attack constructs a plain ARROW item, and the wrapper does not override it with slowness. 

## Result
- **entities/Stray** - FIXED (VERIFIED). Vanilla `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/skeleton/Stray.java:60-66` (`getArrow`) calls `super.getArrow` then, for an `Arrow`, `addEffect(new MobEffectInstance(MobEffects.SLOWNESS, 600))`. `MobEffectInstance(effect, duration)` in `.../world/effect/MobEffectInstance.java:53-58` resolves to amplifier 0, ambient false, visible true, showIcon true, no blend. `crates/pumpkin-core/src/entity/mob/skeleton/stray.rs` overrides `Mob::get_arrow` to `add_effect(Self::slowness_effect())`, where `slowness_effect()` builds exactly that `Effect` (`SLOWNESS`, duration 600, amplifier 0, ambient false, show_particles/show_icon true, blend false). The shared bow goal calls `mob.get_arrow(arrow)` in `entity/ai/goal/bow_attack.rs:127`, so the override is actually reached; the projectile stays a plain ARROW item, matching vanilla which adds the effect to the arrow entity rather than using a tipped item. Added the pure unit test `stray_arrow_slowness_matches_vanilla` asserting the effect fields. No generated-data or WIT dependency.
