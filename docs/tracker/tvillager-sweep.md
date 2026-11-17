# Villager / trading / raid sweep

Scope: `crates/pumpkin-core/src/entity/passive/villager/`, `wandering_trader.rs`, `iron_golem.rs`, `world/raid.rs`, `world/wandering_trader_spawner.rs`, and the villager trade protocol type `pumpkin-protocol/.../merchant_offers.rs`. Zombie villager excluded. No vanilla source is vendored in the repo, so each item is checked against the Java behaviour as recalled, not a decompile.

## Matches (code reviewed, no change)

| Item | Pumpkin | Status |
|---|---|---|
| Breeding food threshold | `BREEDING_FOOD_THRESHOLD = 12`, `can_breed` = food level + inventory points >= 12, age 0 | matches |
| Digest after eating | `DIGEST_FOOD_AMOUNT = 12` | matches |
| Excess food | `EXCESS_FOOD_THRESHOLD = 24` | matches |
| Eat until full | stops at 12, consumes item count as needed | matches |
| Offspring type roll | <0.5 biome, <0.75 own, else partner | matches |
| Restock gate | `allowed_to_restock`: 0 restocks, or <2 and 2400 ticks since last | matches |
| Restock day roll | half day (12000) past last restock or day rollover resets counter | matches |
| Restock site | only from `WorkAtPoi` (workstation use) | matches |
| `catch_up_demand` | resets uses when missed, then updates demand per missed restock | matches |
| `restock` | update demand, reset uses, count++ | matches |
| `MerchantOffer::update_demand` | `demand + uses - (max - uses)`, clamped at 0 | matches |
| `needs_restock` | `uses > 0` | matches |
| Default `maxUses` | 12 (NBT read default) | matches |
| Level-up XP | `[10, 70, 150, 250]` for levels 1..4, max level 5 | matches |
| Level-up bonus XP | +5 orb XP on level up, 3..=6 base pop XP | matches |
| Hero discount | `floor((0.3 + 0.0625 * amp) * baseCostA)`, min 1 | matches |
| Reputation price | `-floor(rep * priceMultiplier)` | matches (see open items) |
| Golem spawn memory | `LAST_SLEPT` within 24000, no `GOLEM_DETECTED_RECENTLY` | matches |
| Golem search | inflate 10, agreement cap 5, panic trigger needs 3, gossip needs 5 | matches |
| Golem placement | 10 tries, range 8 xz / 6 y, legacy iron golem strategy | matches |
| Wandering trader despawn | 48000 set by spawner, decremented only when not trading | matches |
| Raid group counts | Easy 3, Normal 5, Hard 7 | matches |
| Raid bonus spawns | vindicator/pillager 0-1/1/2, witch wave rule, ravager bonus | matches |
| Raid timeout | 48000 ticks | matches |
| Bad omen cap | max 5 | matches |
| Gossip discard threshold | 2 | matches |
| Gossip daily decay | major- 10, minor- 20, major+ 0, minor+ 1, trading 2 | matches |
| Gossip decay interval | 24000 ticks | matches |

## Open items (unverified, no change made)

- FIXED: Gossip max values. `MajorPositive` 20 -> 100 and `MinorPositive` 25 -> 200 (vanilla GossipType: major_positive weight 5 max 100, minor_positive weight 1 max 200). Covered by `gossip_types_use_vanilla_names_and_values`. Reason: high-confidence recall of the wiki/Java table; the other types already matched.
- UNVERIFIED (kept): `Trading` max 25. Recall is not confident enough to change it; decay 2 per 24000 ticks is kept.
- UNVERIFIED (kept): Special prices. `update_special_prices` overwrites `special_price` (idempotent) where vanilla `updateSpecialPrices` appears to accumulate `specialPriceDiff` and rely on `resetSpecialPrices` between sessions. Results should match if reset runs before each refresh. Left as is, no recall with enough confidence.
- UNVERIFIED (kept): Breeding `can_breed` also requires not sleeping. Recalled vanilla `canBreed` may not check sleep, but not with enough confidence to remove the check. Left as is.
- Bonus raid wave rule (`raid_omen_level > 1`) recalled, not re-verified.
- Iron golem natural spawn (village spawner path) not reviewed; only the villager-triggered spawn was.

## Sweep additions (VERIFIED as recalled)

- Wandering trader spawner: default tick 1200, spawn delay 24000, chance 25 rising by 25 up to 75, one-in-10 gate, 10 attempts, search radius 48, llama radius 4, despawn 48000. Matches recall of vanilla `WanderingTraderSpawner`.
- Iron golem village spawn and cat/zombie siege (`village_siege.rs`): constants were not re-derived in this pass; left as UNVERIFIED.

## Checks

Code change: `data.rs` max values and its unit test. `rustfmt --edition 2024` on the leaf file only. No cargo run (worker policy).

## Re-verification against decompiled vanilla (x1)

- FIXED: gossip max values were wrong on master. Vanilla `GossipType` is `MAJOR_POSITIVE(5, 20, 0, 20)`, `MINOR_POSITIVE(1, 25, 1, 5)`, `TRADING(1, 25, 2, 20)`. Code now uses major_positive max 20 and minor_positive max 25 (the earlier "FIXED" 100/200 entry was wrong; those are the negative values). Test `gossip_types_use_vanilla_names_and_values` updated.
- VERIFIED: `Trading` max 25 (GossipType.java).
- VERIFIED: `canBreed` includes `!isSleeping()` (Villager.java line 693), so the sleep check stays.
- VERIFIED (equivalent): `updateSpecialPrices` calls `resetSpecialPrices` first, so overwriting `special_price` is the same result.
- Not re-verified: bonus raid wave rule, iron golem village spawn, cat and zombie siege constants.
