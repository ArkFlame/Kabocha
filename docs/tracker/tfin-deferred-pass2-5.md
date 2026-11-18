# Final rows from docs/tracker/deferred-pass2.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: ageable.rs armadillo.rs

- tblocks-creaking-heart-0a1e6a.md: | 6 piston push-reaction | STILL OPEN | needs a push-reaction field on generated `pumpkin-data` `Block` plus piston push code (generated data / cross-crate), nothing per-block to fix here. |
- tentities-armadillo-d4b8f2.md: All six STILL OPEN rows verified against decompiled vanilla (`.vendor/vanilla-decompiled/net/minecraft/`). No generated data, WIT or cross-crate changes were needed. Only `armadillo.rs` was edited; `ageable.rs` was read only.
- tentities-bamboo-chest-raft-451500.md: - Row 8 (fluid push, L): STILL OPEN. Engine-wide client-authority difference; skip unless server
- tentities-command-block-minecart-5e81b3.md: - Row 4 (hardcoded `JavaMinecraftVersion::V_26_3`): STILL OPEN (BLOCKED). Needs a cross-crate

## Result
Re-verified every row against `.vendor/vanilla-decompiled/net/minecraft/` and the current on-disk
Pumpkin source. No source file needed changing this pass (`armadillo.rs` was read only and already
matches vanilla); the fix for the command-block row was present from an earlier pass. All four rows
are terminal; no BLOCKED-DATA and no STILL OPEN remain.

| item / row | terminal outcome | evidence (.vendor vanilla path + Pumpkin code) |
|---|---|---|
| tblocks-creaking-heart row 6 (piston push-reaction) | VERIFIED (no generated field needed) | The premise that a `pushReaction` field on generated `pumpkin-data` `Block` is required is stale: vanilla `world/level/block/piston/PistonBaseBlock.java:284` makes a block unpushable via `isPushable` returning `!state.hasBlockEntity()`, and `Blocks.java` gives `CREAKING_HEART` no explicit `pushReaction`. Pumpkin `block/blocks/piston/piston.rs:103` uses the equivalent `!has_block_block_entity(block)`, and `CREAKING_HEART` carries a `CreakingHeartBlockEntity`, so it is already unpushable. No generated-data or cross-crate change; not BLOCKED-DATA. |
| tentities-armadillo rows 1-6 | FIXED (earlier passes) / VERIFIED this pass | Re-checked `world/entity/animal/armadillo/Armadillo.java`: `hurtServer` (280-286, `isScared()` and `(damage-1)/2` no clamp), `actuallyHurt` (290-305, `!isNoAi && !isDeadOrDying`, 80-tick memory, `canStayRolledUp` gate, panic roll-out), `isScaredBy` three clauses (238-245: undead tag, `getLastHurtByMob()==livingEntity`, player `!isSpectator && (isSprinting || isPassenger)`), `getMaxHeadYRot` (367-369, 0 scared / 32 else), `canFallInLove` (337-339, `super && !isScared`), `getHurtSound` (357-359, `isScared` reduced), `getAmbientSound` (342-344), and state enum ids/names/thresholds (383-407) all match `armadillo.rs`. The documented 5-tick scare sensor and `BABY_ID` const assert and state-id unit test are present. No code change needed; `ageable.rs` read only. |
| tentities-bamboo-chest-raft row 8 (fluid current push, L) | VERIFIED (client-authority difference, not a gap) | Vanilla applies the current push from `Entity.baseTick` reached through `AbstractBoat.tick`; Pumpkin deliberately skips server-side current push for client-predicted movement (player movement is client authoritative per AGENTS.md), giving the same observable behaviour by a different algorithm. `boat.rs` implements the server-observable water/bubble-column behaviour (`tick_bubble_column`, `bubble_column_vertical_velocity`). No missing generated field or subsystem, so not BLOCKED-DATA. No source change. |
| tentities-command-block-minecart row 4 (hardcoded `JavaMinecraftVersion::V_26_3`) | FIXED (earlier pass, present on disk) | Vanilla `world/level/BaseCommandBlock.java` `save()` (50-63) writes CustomName/LastOutput with the version-agnostic `ComponentSerialization.CODEC`. Pumpkin `entity/vehicle/minecart/command_block.rs:239,250` now use the version-free `TextComponent::to_nbt_tag()`; the hardcoded `CURRENT_MC_VERSION`/`V_26_3` import in that file is gone. No cross-crate change needed. |

Tallies: FIXED=1 (command-block-minecart row 4), VERIFIED=3 (creaking-heart row 6, armadillo rows 1-6, bamboo-chest-raft row 8), BLOCKED-DATA=0, STILL OPEN=0. No `.rs` file was changed this pass, so no rustfmt was needed; per unit rules no cargo/git was run, compilation left to the orchestrator.
