# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-entity-passive-nautilus-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: crates/pumpkin-core/src/entity/passive/nautilus.rs, crates/pumpkin-core/src/entity/passive/zombie_nautilus.rs (plus any cross-file a row needs).

All rows closed. Per row result: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field.

## Result

FINAL PASS (2026-11-18). Re-read the current Rust in `crates/pumpkin-core/src/entity/passive/nautilus.rs` and `zombie_nautilus.rs` against the decompiled vanilla tree at `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/nautilus/` (AbstractNautilus.java, Nautilus.java, NautilusAi.java, ZombieNautilus.java, ZombieNautilusAi.java) plus `.vendor/vanilla-decompiled/net/minecraft/world/inventory/AbstractMountInventoryMenu.java` and `.vendor/vanilla-decompiled/net/minecraft/world/entity/TamableAnimal.java`. Every previously applied fix is present and matches vanilla; no new source change was required. No unwrap/expect/panic or stdout added; no generated data edited.

### Row 1 - entities/Nautilus [RE-REVIEW]: VERIFIED (one sub-fix applied in an earlier pass, present)

- Constructor matches `AbstractNautilus.java:90-96`: `NautilusEntity::new` installs Navigator water_bound(false), WATER malus 0.0 (`:94`, `:126-128`), SmoothSwimmingMoveControl(85,10,0.011,0,true) and SmoothSwimmingLookControl(10) (`:92-93`).
- Goals mirror `NautilusAi.java:41-113` using Pumpkin's legacy GoalSelector (closest seam to vanilla Brain/Activity): panic 1.6 (`:44,71`), breed 0.4 (`:43,85`), tempt 1.3 on MINECRAFT_NAUTILUS_FOOD (`:42,86`), follow parent, random swim 1.0 (`:95`), look around, look-at player, melee/charge 0.6 (`:46,105`), Revenge target goal.
- Dash: `DASH_COOLDOWN_TICKS = 40` (`AbstractNautilus.java:76`), stop when `dashCooldown < 35` (`:306`), ready sound at 0 (`:310-315`) match `tick_abstract_nautilus`. Rider jump uses the controlling rider's look vector, matching `executeRidersJump` `controller.getLookAngle()` (`:348-356`), and the `(isInWater ? 1.2 : 0.5) * amount * MOVEMENT_SPEED * blockSpeedFactor` impulse (`:348-352`).
- Sound selectors branch on `isUnderWater()` per `Nautilus.java:71-117`; bubbles stay on plain in-water (`AbstractNautilus.java:317-319`) matching `touching_water`.
- Spawn rules `checkNautilusSpawnRules` (`AbstractNautilus.java:135-152`) match `entity/type.rs` water/sea-level branch; nautilus spawning is data-driven via biome water_creature spawns.

### Row 2 - own/tentities-nautilus [OWN-OPEN]: all 9 rows closed

| # | Result |
|---|--------|
| 1 | FIXED. NautilusAi goals registered in `new` (panic 1.6, breed 0.4, tempt 1.3, follow parent, swim 1.0, look, melee 0.6 + Revenge). `NautilusAi.java:41-113`. |
| 2 | FIXED. Rideable-jump hook plays the dash sound, scales impulse by momentum (1.2 water / 0.5 land), MOVEMENT_SPEED and block speed factor, sets cooldown 40 and dashes, along the rider look vector. `AbstractNautilus.java:328-363`. |
| 3 | FIXED. Dead 9-slot inventory field deleted. `AbstractNautilus.getInventoryColumns()` returns 0 (`AbstractNautilus.java:525-527`), screen wiring is client GUI (non-blocking per AGENTS.md); no server state to persist. |
| 4 | FIXED. `is_food` uses MINECRAFT_NAUTILUS_TAMING_ITEMS for a wild adult and MINECRAFT_NAUTILUS_FOOD otherwise. `AbstractNautilus.java:98-101`. Both tags exist in generated tag.rs. |
| 5 | FIXED. `new` installs water_bound navigator, WATER malus 0.0, SmoothSwimmingMoveControl(85,10,0.011,0,true), SmoothSwimmingLookControl(10). `AbstractNautilus.java:90-96,126-128`. |
| 6 | VERIFIED, no server seam. `SoundEvents.NAUTILUS_RIDING` is only registered and never played in the decompiled tree; the riding loop is client code and not vendored. The dash sound IS played server-side in the jump hook, matching `handleStartJump` (`AbstractNautilus.java:358-363`). Client-only per AGENTS.md, non-blocking. |
| 7 | VERIFIED. Data-driven water_creature biome spawning plus `check_spawn_rules` sea-25..sea-5 with water below and above, matching `checkNautilusSpawnRules` (`AbstractNautilus.java:135-152`). Zombie nautilus is MONSTER/Unrestricted, absent from biome assets. |
| 8 | FIXED. `mob_write_nbt` writes only Owner (vanilla `TamableAnimal.addAdditionalSaveData`); taming derives from owner, dash cooldown transient; tame/owner sync via DATA_FLAGS_ID/DATA_OWNERUUID_ID. |
| 9 | VERIFIED. `TamableAnimal.feed` heals 2.0 * nutrition (fallback 1.0) and nautilus calls `feed(...,2.0,1.0)` at `AbstractNautilus.java:420-422`; Rust heal matches. |

Tests `taming_persists_as_vanilla_owner_only` and `wild_adult_only_eats_taming_items` are present in nautilus.rs.

### Row 3 - own/tentities-zombie-nautilus [OWN-OPEN]: all 5 rows closed

| # | Result |
|---|--------|
| 1 | FIXED. All five sound selectors use `is_under_water()` (submerged). Vanilla selects on `isUnderWater()` (`ZombieNautilus.java:80-113`); the audit's premise of `isInWater()` was wrong. Bubbles stay on plain in-water (`AbstractNautilus.java:317`). |
| 2 | FIXED. `handle_rider_start_jump` (zombie_nautilus.rs:322-347) gates on cooldown, plays dash sound, applies the `(isInWater ? 1.2 : 0.5) * amount * MOVEMENT_SPEED * blockSpeedFactor` impulse along the rider look vector, sets cooldown 40 and dashes. `AbstractNautilus.java:328-363`. |
| 3 | FIXED. ZombieNautilusAi goals: tempt 0.9, random swim 1.0, look, melee 0.5 + Revenge, no panic/breed. `ZombieNautilusAi.java:23-81`, `NautilusAi.java:115-149`. |
| 4 | VERIFIED, non-blocking. `getInventoryColumns()` returns 0 (`AbstractNautilus.java:525-527`), so `getInventorySize(0)` = 0 (`AbstractMountInventoryMenu.java:105-107`): the mount container has zero slots and no server state. The dead 9-slot field was correctly removed. `player.openNautilusInventory` (`AbstractNautilus.java:508-513`) is a client GUI effect, non-blocking per AGENTS.md. |
| 5 | FIXED. Submerged/land ambient-hurt sound test and `finalize_spawn` biome-variant test present in zombie_nautilus.rs; variant selection matches `ZombieNautilus.java:163-169`. |

Helper paths, struct initializers, closure param types and the `map_or_else` rider-look call were re-checked against the Worker compile pitfalls table; they resolve in-tree and add no new imports. Not compiled here (worker shell denied); the orchestrator compiles once after all workers finish.
