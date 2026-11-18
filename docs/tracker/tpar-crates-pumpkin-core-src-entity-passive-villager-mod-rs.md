# Parity rows for crates/pumpkin-core/src/entity/passive/villager/mod.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-en-e6c42b** [partial/OWN-OPEN] own audit: 1 open, 0 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-en-e6c42b.md) docs/tracker/taudit-crates-pumpkin-core-src-en-e6c42b.md

## Result

Verification pass 2026-11-18. This unit has one aggregation row (`own/taudit-crates-pumpkin-core-src-en-e6c42b`). Its nine underlying gaps are all CLOSED in the child audit doc; re-verified each against decompiled vanilla and against current Pumpkin source. No `.rs` file in this unit changed: every gap is either already fixed in `mod.rs`/`reputation.rs`/`merchant_offers.rs` (outside this unit's editable basenames) or was a false positive in the original tracker text. rustfmt: not applicable (no `.rs` file touched).

`own/taudit-crates-pumpkin-core-src-en-e6c42b`: VERIFIED (matches vanilla). Vanilla reference: `.vendor/vanilla-decompiled/net/minecraft/world/entity/npc/villager/Villager.java` (plus `MerchantMenu.java`, `MerchantOffer.java`). Per-row confirmation:

| Row | Status | Finding |
|---|---|---|
| 1 plugin events never fired | VERIFIED, already fixed | Fire sites present: VillagerAcquireTradeEvent (`mod.rs:714`), VillagerReplenishTradeEvent (`mod.rs:1006`), VillagerCareerChangeEvent (`mod.rs:1122`), VillagerReputationChangeEvent (`reputation.rs:44`). Structs in this unit are correct. |
| 2 level-up XP bonus +5 | VERIFIED, no change | Vanilla `rewardTradeXp` does `popXp += 5` (Villager.java:621); Pumpkin `pop_xp += 5` (`mod.rs:874`) matches. Tracker claim false. |
| 3 catch_up_demand resets uses | VERIFIED, no change | Vanilla `catchUpDemand` calls `offer.resetUses()` per offer (Villager.java:414-421); Pumpkin `reset_uses()` (`mod.rs:1021`) matches. |
| 4 LastGossipShare write-only | VERIFIED, already fixed | Vanilla persists only `LastRestock` (Villager.java:500/519); grep finds no `LastGossipShare` key in source. |
| 5 wants_to_pick_up VillagerFood | VERIFIED, no change | Vanilla `wantsToPickUp` includes `itemStack.has(DataComponents.VILLAGER_FOOD)` (Villager.java:835); Pumpkin checks `VillagerFoodImpl` (`mod.rs:1561`) in addition to the tag and requested items. |
| 6 should_restock moves restock time | VERIFIED, no change | Vanilla sets `lastRestockGameTime = gameTime` in the day-change branch (Villager.java:406-407); Pumpkin `last_restock_time.store(game_time)` (`mod.rs:986`) matches. |
| 7 special prices computed once | VERIFIED, no change | Vanilla `MerchantMenu.slotsChanged` only calls `updateSellItem`; no per-update recompute. No port needed. |
| 8 talked-to stat without trade | VERIFIED, no change | Vanilla awards `TALKED_TO_VILLAGER` before the no-offers check (Villager.java:306); Pumpkin increments stat (`mod.rs:2038`) before the `!has_offers` return (`mod.rs:2043`). |
| 9 regen effect placement | VERIFIED, already fixed | Vanilla applies REGENERATION in `rewardTradeXp` (Villager.java:618); Pumpkin applies it in `complete_trade` (`mod.rs:866`). |
