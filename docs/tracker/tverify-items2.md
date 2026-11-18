# tverify-items2: vanilla audit of piglin, sulfur cube, zombie villager, bat, and items

Reference: `.vendor/vanilla-decompiled/net/minecraft/` (decompiled). Not compiled or run in this pass (CODER worktree, no cargo); fixes are UNVERIFIED by build and tests.

| Subject | Status | Evidence / reason |
|---|---|---|
| Piglin (`entity/mob/piglin.rs`, `piglin_ai.rs`) | STILL OPEN | Scope exceeds one pass (1343 + 309 lines). Crossbow power skipped per brief (verified 1.6); hoglin skipped per brief. Not audited this pass. |
| Sulfur Cube (`entity/mob/sulfur_cube.rs`) | STILL OPEN | Scope exceeds one pass (1510 lines). Not audited this pass. |
| Zombie Villager (`entity/mob/zombie/zombie_villager.rs`) | FIXED 11fb171b7 | `getConversionProgress` loop guard is `specialBlocksCount < 14`, so the cap is exclusive. Pumpkin allowed 15 counted blocks (`> 14`). Changed to `>=`. Cure Strength `min(difficulty-1, 0)`, conversion wait, NAUSEA 200/0, and equipment handling were checked against `ZombieVillager.java` and match. |
| Bat (`entity/mob/bat.rs`) | STILL OPEN | Spawn rules, flying target selection and RNG order, velocity/yaw steering, roost wake/liftoff, invulnerability ordering, and sounds match `Bat.java`. Open: resting-check uses `is_solid_block`; vanilla uses `isRedstoneConductor` (differs for non-full solids such as slabs). No Pumpkin conductor predicate exists yet. Needs a predicate (codegen or block data) before the fix. |
| Fishing Rod (`item/items/fishing_rod.rs`) | FIXED 4f6511b07 | Vanilla `FishingRodItem.use` awards `ITEM_USED` only on the cast branch. Pumpkin never awarded it. Added `increment_stat(Used, ...)` on cast. Open: vanilla `PlayerFishEvent` cancel on cast (`player.fishing = null`, PASS) is not wired. Pumpkin's `swing_hand` on reel-in cites vanilla, but the decompiled `use` has no server swing; the client sends the swing. Needs a client-behaviour check before changing it. |
| Mace (`item/items/mace.rs`, `entity/combat.rs`, `entity/player.rs`) | VERIFIED | `canSmashAttack` (fall > 1.5, not gliding), damage bonus curve (4x / 12+2x / 22+x), enchantment term, knockback radius 3.5, power `(3.5-d)*0.7*(heavy?2:1)*(1-res)`, Y velocity 0.01, sounds by `fallDistance > 5` and `onGround`, fall reset, and target filters all match `MaceItem.java`. |
| Compass (`item/items/compass.rs`) | VERIFIED | `replaceExistingStack = !hasInfiniteMaterials && count == 1` matches the in-place vs transmute-copy branch. Creative always takes the copy branch, as in vanilla. Not covered: compass angle tick (lives outside `compass.rs`), so it is not part of this verdict. |
| Recovery Compass (`item/items/compass.rs`) | VERIFIED | Vanilla `RecoveryCompassItem` does not override `useOn`, so it falls through to `Item.useOn` (pass). Pumpkin gates the lodestone branch on `Item::COMPASS` only. |
| Goat Horn (`item/items/goat_horn.rs`) | FIXED 66922cb8b | Vanilla `InstrumentItem.use` adds the cooldown only when `useDuration > 0`. Pumpkin always added it, and its comment wrongly cited vanilla. Now guarded by `duration > 0`. Open: `get_use_duration` hardcodes `PonderGoatHorn` because `ItemBehaviour::get_use_duration` takes no stack. Vanilla returns the held horn's duration (0 when no instrument). Needs a stack-aware trait method. |
| Lodestone (`block/blocks/lodestone.rs`) | STILL OPEN | Vanilla lodestone is a plain block. Its POI comes from `PoiTypes` block-state mapping, not block hooks. Pumpkin adds and removes POI in `placed` and `on_state_replaced`. Open: chunk-load POI rebuild from block states is not audited. |

Summary: verified 3, fixed 3, open 4 (piglin, sulfur cube, bat, lodestone). Open follow-ups inside fixed rows: fishing cast cancel, goat horn `get_use_duration`, bat conductor predicate.

Unverified: no build or test run in this pass. Fix commits add no tests. The cheap pure-test path did not apply: each fix needs world or player state (gametest territory).

## Result (batch x_10, CODER)
- Bat roost conductor check (open follow-up): VERIFIED. Vanilla `.vendor/vanilla-decompiled/net/minecraft/world/entity/ambient/Bat.java:136,178` uses `isRedstoneConductor`; Rust `crates/pumpkin-core/src/entity/mob/bat.rs:99,201` uses `is_redstone_conductor()`. Row closed.
- Lodestone POI (open): partly VERIFIED. Rust `crates/pumpkin-core/src/block/blocks/lodestone.rs:8-9` derives POI from block-state ids in `World::update_poi`, matching vanilla's `PoiTypes` mapping. Chunk-load POI rebuild not audited: STILL OPEN.
- Piglin, sulfur cube: STILL OPEN, not audited this pass (scope).
- Files edited: this doc only. No Rust edited; no checks run.
