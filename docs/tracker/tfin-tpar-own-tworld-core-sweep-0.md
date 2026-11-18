# Final rows from docs/tracker/tpar-own-tworld-core-sweep.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: hunger.rs player.rs

- Difficulty effects in entity code (the last former STILL OPEN row): VERIFIED against vanilla `FoodData`/`ServerPlayer`/`Player`/`RegenerationMobEffect`: `hunger.rs::tick` matches `.vendor/.../world/food/FoodData.java` `tick` (exhaustion drop 4.0, saturation-before-food non-Peaceful, saturated tick%10 spend min(sat,6) heal spent/6, unsaturated level>=18 tick%80 heal 1 +exhaustion 6, starvation 1.0 with health>10 || Hard || health>1 && Normal, MAX 40.0); `player.rs` Peaceful regen (tick%20 heal 1 + sat+1<20; tick%10 food+1<20) matches `ServerPlayer.java::tickRegeneration` (lines 1173-1189); Regeneration `50 >> amplifier` heal 1 matches `RegenerationMobEffect.java` (lines 12-24); `difficulty_scaled_damage`/`damage_scales_with_difficulty` match `Player.java::hurtServer` (lines 727-739) with test `difficulty_scaling_matches_vanilla_hurt_server` (`player.rs:10935-10954`). No generated pumpkin-data field or missing subsystem involved.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
