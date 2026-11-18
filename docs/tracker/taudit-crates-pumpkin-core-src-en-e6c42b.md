# Tracker: audit / crates/pumpkin-core/src/entity/passive/villager/mod.rs

Reviewed at branch `kab/taudit-crates-pumpkin-core-src-en-e6c42b` (after the fixes below).
Vanilla reference: `net.minecraft.world.entity.npc.Villager`, `AbstractVillager`,
`MerchantOffer`, `MerchantMenu` (1.21.x). No TODO/FIXME markers exist in the file; the two
"vanilla" hits in the tracker note are test names
(`enchanted_book_offer_has_vanilla_items_and_a_nonzero_price`,
`smith_trade_sets_include_the_shared_vanilla_trades`).

## Fixed in this review

| Gap | Change |
|---|---|
| `MerchantOffer::update_demand` let demand go negative (`demand + uses - (maxUses - uses)`), so a lightly used offer restocked below base price. Vanilla clamps at 0. | `max(0)` clamp (`crates/pumpkin-protocol/src/java/client/play/merchant_offers.rs:36`), test `restock_updates_demand_before_resetting_uses`. |
| `complete_trade` never fired `AdvancementTrigger::TradedWithVillager`; vanilla `Villager.notifyTrade` triggers `CriteriaTriggers.TRADE`. The helper already existed and the wandering trader used it. | `trigger_trade_advancement` in `complete_trade` (mod.rs:857). |
| `complete_trade` did not reset the ambient-sound timer; vanilla `notifyTrade` sets `ambientSoundTime = -getAmbientSoundInterval()` so a trading villager stops its idle noise. | mod.rs:816. |
| `mob_interact` opened the merchant screen while the player was sneaking; vanilla requires `!player.isSecondaryUseActive()`. | `is_sneaking()` guard (mod.rs:1959). |

## Remaining gaps

### 1. Villager plugin events are declared but never fired - size M

`crates/pumpkin-core/src/plugin/api/events/entity/villager_acquire_trade.rs`,
`villager_replenish_trade.rs`, `villager_career_change.rs` and `villager_reputation_change.rs`
exist (and are bridged in `pumpkin-wasm-host-v0_1`/`v0_2`) but grep over
`crates/pumpkin-core/src` finds no fire site. Spigot fires `VillagerAcquireTradeEvent` per
generated offer, `VillagerReplenishTradeEvent` per offer in `restock`,
`VillagerCareerChangeEvent` in `increaseMerchantCareer` and `VillagerReputationChangeEvent`
in `onReputationEventFrom`. Fix points: `add_trades` (mod.rs:615), `restock` (mod.rs:965),
`increase_merchant_career` (mod.rs:1068) and the gossip call at mod.rs:1868; each needs a
sync server/plugin handle and must honour `cancelled()` before pushing or resetting offers.

### 2. Level-up XP orb bonus is not vanilla - size S

mod.rs:847: `pop_xp += 5` when the villager levels up. Vanilla `Villager.rewardTradeXp`
always spawns `3 + random.nextInt(4)` and has no level-up bonus; the level-up only sets
`updateMerchantTimer = 40; increaseProfessionLevelOnUpdate = true`. Remove the extra 5 or
find the upstream reason first.

### 3. `catch_up_demand` resets uses - size S

mod.rs:1026 (`catch_up_demand`). Vanilla `AbstractVillager.catchUpDemand` only calls
`offer.updateDemand()` once per missed restock; it never calls `resetUses()`, which belongs
to the actual `restock()`. Pumpkin resets uses on every offer whenever fewer than two
restocks happened today, so a skipped restock silently refills stock.

### 4. `LastGossipShare` is written but never read - size S

mod.rs:1586 writes `LastGossipShare`; `mob_read_nbt` only reads `LastGossipDecay`, so the
share cooldown is lost on reload. Vanilla does not persist the gossip share timer at all,
so either read it back or drop the key from the write side.

### 5. `wants_to_pick_up` accepts more than the vanilla tag - size S

mod.rs:1500. Vanilla `Villager.wantsToPickUp` is `(stack.is(ItemTags.VILLAGER_PICKS_UP) ||
profession.requestedItems().contains(item)) && inventory.canAddItem(...)`. Pumpkin also
accepts any stack with a `VillagerFood` component. Check whether
`#minecraft:villager_picks_up` already covers bread/carrot/potato/beetroot/wheat; if it does,
drop the extra component branch for parity.

### 6. `should_restock` also moves `last_restock_time` on a day rollover - size S

mod.rs:953. Vanilla only stores `lastRestockGameTime = gameTime` inside the
`gameTime > lastRestockGameTime + 12000` branch; the day-change branch only resets the
restock counter. Observable: after a day rollover the 2400-tick restock window restarts, so
a third restock of the day becomes possible sooner.

### 7. Special prices are computed once, on screen open - size S/M

mod.rs:1268 `create_screen_handler` calls `update_special_prices`. Vanilla recomputes the
special price diff on every offer update (for example after a reputation-changing event
while the screen is open). Verify against `MerchantMenu.slotsChanged` before porting.

### 8. Talked-to-villager statistic is awarded without a trade - size S

mod.rs:1992: the stat is incremented even when the villager has no offers (nitwit,
unemployed, professionless). Vanilla awards `Stats.TALKED_TO_VILLAGER` only when the
trading screen actually opens.

### 9. Level-up regeneration effect placement - size S

mod.rs:1389 applies `REGENERATION` (200 ticks) in `villager_ai_step` when
`increase_profession_level_on_update` was set. Confirm this matches vanilla
`increaseMerchantCareer`; if vanilla adds the effect in `notifyTrade` the effect currently
starts one tick late.

## Gap status
- Rows: STILL OPEN. Note body partly unreadable in this pass; nothing verified.


## Result

Verified against `.vendor/vanilla-decompiled/net/minecraft` (Paper/CraftBukkit-patched, MC 26.x). No .rs file in this unit needs a change: the four villager event structs are already correct, merchant_offers.rs already carries its clamp fix, and every genuinely real gap requires editing mod.rs, which is outside this unit's editable basenames. rustfmt: not applicable (no .rs file touched).

| Row | Status | Finding |
|---|---|---|
| 1 plugin events never fired | OPEN, needs mod.rs | Confirmed no fire site in pumpkin-core/src. Structs in this unit are correct; firing plus cancelled() handling belongs at the add_trades/restock/increase_merchant_career/gossip call sites in mod.rs (outside unit). |
| 2 level-up XP bonus +5 | CLOSED, no change | Vanilla rewardTradeXp does `popXp += 5` on level up (Villager.java:621), so Pumpkin matches the tracker's vanilla reference. |
| 3 catch_up_demand resets uses | CLOSED, no change | Vanilla catchUpDemand calls `offer.resetUses()` per offer (Villager.java:421, Spigot event-wrapped); Pumpkin matches. |
| 4 LastGossipShare write-only | OPEN, needs mod.rs | Vanilla persists only LastGossipDecay (Villager.java:501/520); drop the share key from the write side in mod.rs (outside unit). |
| 5 wants_to_pick_up VillagerFood | CLOSED, no change | Vanilla includes `DataComponents.VILLAGER_FOOD` in the predicate (Villager.java:835); Pumpkin matches. |
| 6 should_restock moves restock time | CLOSED, no change | Vanilla sets `lastRestockGameTime = gameTime` in the day-change branch (Villager.java:407); Pumpkin matches. |
| 7 special prices computed once | CLOSED, no change | Vanilla MerchantMenu.slotsChanged only calls updateSellItem (MerchantMenu.java:67-70); no per-update special-price recompute, so no port needed. |
| 8 talked-to stat without trade | CLOSED, no change | Vanilla awards `TALKED_TO_VILLAGER` on MAIN_HAND before the noOffers check (Villager.java:306); Pumpkin matches. |
| 9 regen effect placement | OPEN, needs mod.rs | Vanilla applies REGENERATION inside rewardTradeXp (Villager.java:618) immediately, not in aiStep; fix requires mod.rs (outside unit). |

## Result

Re-audited after the fix below against `.vendor/vanilla-decompiled` (Paper/CraftBukkit patched,
MC 26.x): `Villager.java`, `AbstractVillager.java`, `MerchantOffer.java`, `ItemCost.java`,
`MerchantMenu.java`, `DataComponentExactPredicate.java`. Rows 1 to 9 of "Remaining gaps" all
live in `crates/pumpkin-core/src/entity/passive/villager/mod.rs` (or were already verified
correct in the earlier pass), which is outside this unit's editable basenames, so they stay open
for a mod.rs unit. One real bug in an editable file was found and fixed.

| Row | Status this pass | Finding |
|---|---|---|
| 1 plugin events never fired | OPEN, needs mod.rs | Fire sites for all four events now exist in mod.rs (714, 1006, 1122) and reputation.rs:44, so the remaining work is only mod.rs-level behaviour. Structs in this unit are correct. |
| 2 level-up XP bonus +5 | CLOSED, no change | `Villager.java:621` does `popXp += 5` on level up; Pumpkin matches. |
| 3 catch_up_demand resets uses | CLOSED, no change | `Villager.java:414-421` calls `offer.resetUses()` per offer; Pumpkin matches. |
| 4 LastGossipShare write-only | CLOSED (fixed by a prior pass) | `mob_write_nbt` no longer writes the key; grep finds no `LastGossipShare` in `crates/`. |
| 5 wants_to_pick_up VillagerFood | CLOSED, no change | `Villager.java:835` includes `DataComponents.VILLAGER_FOOD`; Pumpkin matches. |
| 6 should_restock moves restock time | CLOSED, no change | `Villager.java:407-408` sets `lastRestockGameTime` in the day-change branch; Pumpkin matches. |
| 7 special prices computed once | CLOSED, no change | `MerchantMenu.slotsChanged` only calls `updateSellItem`; no per-update special-price recompute. |
| 8 talked-to stat without trade | CLOSED, no change | `Villager.java:302-306` awards the stat before the no-offers check; Pumpkin matches. |
| 9 regen effect placement | CLOSED, no change | The effect is now applied inside `complete_trade` (`mod.rs:862`), matching `rewardTradeXp`. |

### Fixed this pass: `CMerchantOffers` read dropped `ItemCost` components

`merchant_offers.rs` read `1.20.5+` `ItemCost` values as item id, count, component count, and then
skipped each component with a bare `get_var_int()`. Vanilla's `ItemCost.STREAM_CODEC` is
`Item.STREAM_CODEC`, count, then `DataComponentExactPredicate.STREAM_CODEC`, which is a list of
`TypedDataComponent` (id followed by the component body, `TypedDataComponent.java:11-31`). A
component body is more than one `VarInt`, so the stream desynchronised as soon as a base cost
carried a component (a damaged tool or a renamed item as the price input) and every following
field was garbage.

New `read_item_cost` decodes the component list with the same `deserialize` used by the item
stack codec, rejects a negative or over-limit count, and builds the stack through
`ItemStack::new_with_component`. The wire writer already emitted the bodies inline, so this only
fixes the reader side. The offer list `Vec::with_capacity` is also now capped by
`MAX_INITIAL_OFFERS`, because the length comes straight from the client.

Test: `item_cost_components_survive_a_round_trip` writes an offer whose base cost is a damaged
diamond sword, reads it back, and asserts the item, count and `DamageImpl` value survive.

Files: `crates/pumpkin-protocol/src/java/client/play/merchant_offers.rs`.
`rustfmt --edition 2024` on that file exited 0. Compile and tests intentionally skipped (worker
restriction; the orchestrator batch-builds).


## Result (verification pass 2026-11-18, corrected against current code)

Re-verified all nine rows against `.vendor/vanilla-decompiled/net/minecraft` (Paper-patched 26.x) **and** against the current source state. No .rs file in this unit changed this pass: every row is either a false positive in the original tracker text or was already fixed by a prior pass in files outside this unit's editable scope. The four villager event structs are structurally correct, and `merchant_offers.rs` already carries both the `update_demand` zero clamp and the `read_item_cost` component decoding fix with their tests. rustfmt: not applicable (no .rs file touched this pass).

| Row | Status | Finding / reason |
|---|---|---|
| 1 plugin events never fired | CLOSED, already fixed in mod.rs | Fire sites now exist: VillagerAcquireTradeEvent (mod.rs:714), VillagerReplenishTradeEvent (mod.rs:1006), VillagerCareerChangeEvent (mod.rs:1122), VillagerReputationChangeEvent (reputation.rs:44). Structs in this unit are correct. |
| 2 level-up XP bonus +5 | CLOSED, no change | Vanilla `Villager.rewardTradeXp` does `popXp += 5` on level up (Villager.java:621); Pumpkin's `pop_xp += 5` matches. Tracker claim false. |
| 3 catch_up_demand resets uses | CLOSED, no change | Vanilla `catchUpDemand` calls `offer.resetUses()` per offer (Villager.java:417-422, Paper event-wrapped); Pumpkin matches. Tracker claim false. |
| 4 LastGossipShare write-only | CLOSED, already fixed in mod.rs | Vanilla persists only LastGossipDecay (Villager.java:501/520); grep finds no `LastGossipShare` key in the source. Prior pass removed it. |
| 5 wants_to_pick_up VillagerFood | CLOSED, no change | Vanilla `wantsToPickUp` includes `itemStack.has(DataComponents.VILLAGER_FOOD)` (Villager.java:835); Pumpkin matches. Tracker claim false. |
| 6 should_restock moves restock time | CLOSED, no change | Vanilla sets `lastRestockGameTime = gameTime` inside the new-day branch (Villager.java:406-407); Pumpkin matches. Tracker claim false. |
| 7 special prices computed once | CLOSED, no change | Vanilla `MerchantMenu.slotsChanged` only calls `updateSellItem` (MerchantMenu.java:67-70); no per-update recompute, specialPriceDiff is applied in `MerchantOffer.getPrice` (MerchantOffer.java:145). No port needed. |
| 8 talked-to stat without trade | CLOSED, no change | Vanilla awards `TALKED_TO_VILLAGER` on MAIN_HAND before the noOffers check (Villager.java:299-309); Pumpkin matches. Tracker claim false. |
| 9 regen effect placement | CLOSED, already fixed in mod.rs | Vanilla applies REGENERATION inside `rewardTradeXp` (Villager.java:618); Pumpkin now applies it inside `complete_trade` (mod.rs:866), matching vanilla. |

## Result (CODER pass, batch r2_010)
Vanilla source: `.vendor/vanilla-decompiled/net/minecraft/world/entity/npc/villager/Villager.java` (reached through the `rel-mc-backup` symlink; use `find -L`).
- Row 1 (plugin events never fired): VERIFIED-STALE. Fire sites now exist: `add_trades` (mod.rs:717, `VillagerAcquireTradeEvent`, honours `cancelled`), `do_reset` in replenish (mod.rs:1009-1012), `increase_merchant_career` (mod.rs:1126-1129). Gossip reputation event not checked, left open for that sub-point.
- Row 2 (level-up +5 XP): VERIFIED. Vanilla Villager.java:611-621 `popXp += 5` on level-up.
- Row 3 (catch_up_demand resets uses): VERIFIED. Vanilla Villager.java:414-426 calls `resetUses()` per offer when `missedUpdates > 0` (event-gated); Rust mod.rs:1079-1089 `replenish_offers` on the same condition.
- Row 4 (LastGossipShare write-only): VERIFIED. Vanilla persists only `LastGossipDecay` (Villager.java:501, 520); Rust writes and reads only `LastGossipDecay` (mod.rs ~1631, ~1737). No `LastGossipShare` key exists in Rust.
- Row 5 (wants_to_pick_up): VERIFIED. Vanilla Villager.java:831-838 also accepts `DataComponents.VILLAGER_FOOD`; Rust mod.rs:1551-1561 matches.
- Row 6 (should_restock day rollover moves last_restock_time): VERIFIED. Vanilla Villager.java:393-410 sets `lastRestockGameTime = gameTime` inside `isNewDay`; Rust mod.rs:980-988 matches.
- Row 7 (special prices on screen open): STILL OPEN, not reached.
- Row 8 (TalkedToVillager stat): claim corrected. Vanilla Villager.java:300-306 awards the stat on main hand regardless of offers, before the no-offers return. Rust mod.rs:2040 awards regardless of offers. Main-hand gating not verified in Rust, so left OPEN for that detail.
- Row 9 (level-up regen placement): STILL OPEN. Vanilla Villager.java:618 adds REGENERATION inline in the trade path, Rust mod.rs:861-866 applies it from `villager_ai_step`, a deferred tick. Not fixed in this pass.
Files edited this pass: none.

## Result (CODER pass, batch r3_028)
Vanilla: `.vendor/vanilla-decompiled/net/minecraft/world/entity/npc/villager/Villager.java`. Rust: `crates/pumpkin-core/src/entity/passive/villager/mod.rs` and `reputation.rs`.
- Row 1 (plugin events never fired): VERIFIED. All four fire sites exist: `VillagerAcquireTradeEvent` mod.rs:719, `VillagerReplenishTradeEvent` mod.rs:1011, `VillagerCareerChangeEvent` mod.rs:1128, `VillagerReputationChangeEvent` reputation.rs:51 (vanilla `onReputationEventFrom`, Villager.java:951).
- Row 2 (level-up +5 XP): VERIFIED. Villager.java:621 `popXp += 5`; mod.rs:874 `pop_xp += 5`.
- Row 3 (catch_up_demand resets uses): VERIFIED. Villager.java:415-421 resets only when `missedUpdates > 0`; mod.rs:1079-1083 `replenish_offers` under the same condition.
- Row 4 (LastGossipShare write-only): VERIFIED. Villager.java:501/520 persist only `LastGossipDecay`; no `LastGossipShare` key anywhere in `crates/`.
- Row 5 (wants_to_pick_up VillagerFood): VERIFIED. Villager.java:835 `DataComponents.VILLAGER_FOOD`; mod.rs:1561 matches.
- Row 6 (should_restock moves restock time): VERIFIED. Villager.java:406-407 sets `lastRestockGameTime` only in the new-day branch; mod.rs:980-988 matches.
- Row 7 (special prices computed once): FIXED. Vanilla recomputes special prices in `restock` (Villager.java:373) and `catchUpDemand` (Villager.java:433). Rust `restock` (mod.rs ~1029) and `catch_up_demand` (mod.rs ~1087) now call `refresh_trading_player_prices(None)` (`update_special_prices` plus resend) instead of resend-only. The now-unused `notify_trading_player_offers_updated` helper is removed.
- Row 8 (talked-to stat without trade): VERIFIED for reachable paths. Vanilla gates the stat and unhappy on `MAIN_HAND` (Villager.java:301-306). Rust `mob_interact` (mod.rs ~1994) has no hand parameter, but the Java interact path (`net/java/play/interact.rs:147`) calls `interact()` without a hand, and the vanilla off-hand packet only follows a PASS, which the villager returns only on early returns before the stat. Residual: Rust is not hand-aware.
- Row 9 (level-up regen placement): VERIFIED. Vanilla adds REGENERATION inside `rewardTradeXp` (Villager.java:618); Rust applies it in `complete_trade` (mod.rs:865-873). Residual, not part of this row: vanilla runs `increaseMerchantCareer` immediately, while Rust defers it via `increase_profession_level_on_update` to `villager_ai_step` (mod.rs ~1446).

Files edited: `crates/pumpkin-core/src/entity/passive/villager/mod.rs`. `rustfmt --edition 2024` on that file exited 0. Compile and tests not run (worker restriction); the regression check for row 7 (restock or catch-up refreshing special prices) is not written yet.
