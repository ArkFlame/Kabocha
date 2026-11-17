# items / Fishing Rod — re-review

Audit of the current fishing rod + bobber source after the bobber rewrite (97beb89a3).
Vanilla reference: `FishingRodItem`, `FishingHook` (`FishingBobberEntity`). No decompiled
sources in this worktree; comparisons use the vanilla call chain.

## Fixed in this review

| Gap | Evidence | Commit |
|---|---|---|
| Bobber never despawned after leaving the water. Vanilla `FishingHook.tick` discards when `outOfWaterTime >= MAX_OUT_OF_WATER_TIME` (10). Rust clamped the counter but never acted on it. | `crates/pumpkin-core/src/entity/projectile/fishing_bobber.rs:404-411` (before), now `:418` | abd8a4c96 |
| Reeling in did not swing the hand. Vanilla `FishingRodItem.use` calls `player.swing(hand)` on the reel branch. | `crates/pumpkin-core/src/item/items/fishing_rod.rs:99` | e27d8b212 |

Issue #3522 (no catch item/xp/durability) is implemented: loot + xp in
`fishing_bobber.rs:166-251`, durability application in `fishing_rod.rs:92-95`, so no source
gap remains there.

Issue #3051 (malformed `set_entity_data` for FishingHook) does not reproduce in the writers:
`tracked_data.rs:8188-8203` declares `DATA_BITING` as BOOLEAN and `DATA_HOOKED_ENTITY` as INT,
and `entity_metadata.rs:212-219`/`:262-272` write those as one byte / a VarInt. Any remaining
symptom needs a live client capture, not a source edit.

## Remaining gaps

| # | file:line | Vanilla class / method | Gap | Suggested fix | Size |
|---|---|---|---|---|---|
| 1 | `crates/pumpkin-core/src/world/loot/predicates.rs:363-378` | `FishingHook.calculateOpenWater` | `in_open_water` only checks that the hook's own block is water. Vanilla scans the surrounding volume and rejects water with solid blocks or non-water nearby, so treasure loot is rolled for a 1-block puddle. | Port the vanilla `calculateOpenWater` scan and use it in the loot predicate instead of the single block check. | M |
| 2 | `crates/pumpkin-core/src/net/java/play/use_item.rs:40` + `crates/pumpkin-core/src/item/items/fishing_rod.rs:84` | `FishingRodItem.use` stat awarding | The generic use handler awards `StatisticCategory::Used` for every right-click and the rod awards it again on cast, so casting counts twice. Vanilla awards `ITEM_USED` only on the cast branch, never on reel-in. | Remove the rod-local increment, or move the generic award into the item behaviours; a reel-in must not count. Cross-cutting: `snowball.rs:71`, `egg.rs:74`, `spyglass.rs:38` also double-award. | M |
| 3 | `crates/pumpkin-core/src/net/java/play/use_item.rs:192` | `FishingRodItem.use` | Stale TODO: "Apply fishing rod durability on retrieval based on catch type." `retrieve` already returns 1/2/3/5 and `fishing_rod.rs:92-95` applies it. | Delete the TODO. | S |
| 4 | `crates/pumpkin-core/src/entity/projectile/fishing_bobber.rs:702-716` | `FishingHook(Player, Level, int, int)` | Throw origin offsets the hook 0.3 forward of the eye; the classic constructor spawns at `player.getX(), getEyeY(), player.getZ()`. Unverified against the V_26_3 reference. | Check the 26.3 constructor and drop the X/Z offset if absent. | S |

## Caveat

No cargo/test run was allowed in this pass (WORKER.md rule 1). The two commits are parity
edits reviewed against the vanilla call chain only; the orchestrator batch build is the gate.

