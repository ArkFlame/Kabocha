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
