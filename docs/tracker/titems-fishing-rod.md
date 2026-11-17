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
| FIXED (UNVERIFIED) 1 | `crates/pumpkin-core/src/world/loot/predicates.rs:363-378` | `FishingHook.calculateOpenWater` | `in_open_water` only checked the hook's own block, so a 1-block puddle rolled treasure. | `is_open_water` scans the 5x5 column (four layers, water down to the floor, air allowed above) at loot time; the bobber-tracked `openWater` flag is still not modelled. | M |
| 2 | `crates/pumpkin-core/src/net/java/play/use_item.rs:40` + `crates/pumpkin-core/src/item/items/fishing_rod.rs:84` | `FishingRodItem.use` stat awarding | The generic use handler awards `StatisticCategory::Used` for every right-click and the rod awards it again on cast, so casting counts twice. Vanilla awards `ITEM_USED` only on the cast branch, never on reel-in. | Remove the rod-local increment, or move the generic award into the item behaviours; a reel-in must not count. Cross-cutting: `snowball.rs:71`, `egg.rs:74`, `spyglass.rs:38` also double-award. | M |
| 3 | `crates/pumpkin-core/src/net/java/play/use_item.rs:192` | `FishingRodItem.use` | Stale TODO: "Apply fishing rod durability on retrieval based on catch type." `retrieve` already returns 1/2/3/5 and `fishing_rod.rs:92-95` applies it. | Delete the TODO. | S |
| 4 | `crates/pumpkin-core/src/entity/projectile/fishing_bobber.rs:702-716` | `FishingHook(Player, Level, int, int)` | Throw origin offsets the hook 0.3 forward of the eye; the classic constructor spawns at `player.getX(), getEyeY(), player.getZ()`. Unverified against the V_26_3 reference. | Check the 26.3 constructor and drop the X/Z offset if absent. | S |

## Caveat

No cargo/test run was allowed in this pass (WORKER.md rule 1). The two commits are parity
edits reviewed against the vanilla call chain only; the orchestrator batch build is the gate.


## h6 gap status
- Row 3 (stale durability TODO in `use_item.rs`): FIXED 50a7d1ee2 (TODO removed; durability is applied in `item/items/fishing_rod.rs:96-98`).
- Row 4 (hook origin X/Z offset): STILL OPEN, UNVERIFIED against the 26.3 constructor; not changed.

## Result

Re-review against decompiled vanilla in .vendor/vanilla-decompiled (final pass).

- Row 2 (ITEM_USED stat, scoped): FIXED. Vanilla ItemStack.use (ItemStack.java:525-536) does not award ITEM_USED; each item awards in its own use() (FishingRodItem.java:69, cast branch only). Pumpkin's generic award in use_item.rs:40 plus local awards double-counted, and the rod also counted reel-in. Changes: the generic handler now skips the fishing rod; fishing_rod.rs awards on the cast branch after the plugin-cancel check and before the start vibration; the redundant local awards were removed from snowball.rs, egg.rs and spyglass.rs. Cast now counts once, reel-in and plugin-cancelled casts count zero. The remaining systemic fix (roughly 35 other items that still double-award) spans files outside this unit and is left open as a cross-crate refactor.
- Row 3: already fixed in a prior pass, no change.
- Row 4 (hook origin X/Z offset): VERIFIED, NO CHANGE NEEDED. The row's premise was wrong. Vanilla 26.3 FishingHook(Player,Level,int,int) (FishingHook.java:100-112) spawns at getX() - ySin * 0.3, getEyeY(), getZ() - yCos * 0.3. Pumpkin's throw_setup (fishing_bobber.rs:777-782) reproduces this exactly and already has unit tests at :895-905, so no offset needs dropping.
- Row 1: already fixed in a prior pass, not re-opened.

No unit test added: the stat decision needs a live Player so there is no pure seam, and row 4's origin math is already covered by the existing throw_setup tests. rustfmt --edition 2024 passed on all five edited files. Compilation and tests are deferred to the orchestrator batch build.
