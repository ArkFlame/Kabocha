# Advancement trigger sweep

Scope: `AdvancementTrigger` variants in `crates/pumpkin-core/src/entity/player/advancement/trigger.rs`
with no call site outside their match arm (checked with `git grep "AdvancementTrigger::<Variant>"`).

## Already fired (no change)

ConsumeItem, BredAnimal, TameAnimal, TradedWithVillager, EnchantedItem, BrewedPotion,
FishedItem, ChanneledLightning, SummonedEntity, PlacedBlock, ItemUsedOnBlock, PlayerHurtEntity.

## Fixed

- PlayerInteractedWithEntity: fired in `net/java/play/interact.rs` after a successful entity
  interaction, with the pre-interaction held item (vanilla `Player.interactOn` copies the stack first).

## Not fired, no confident vanilla point in Kabocha

- SlideDownBlock: honey block has no implementation in Kabocha (no `honey` block behaviour).
- Levitation: vanilla point (per-tick with `start` position) not confirmed against decompiled source.
- EnterBlock: no player inside-block loop in Kabocha (only bubble column `entity_inside`).
- NetherTravel: dimension-change distance semantics not confirmed.
- AvoidVibration: sneaking-vs-sculk listener point not confirmed.
- FallAfterExplosion: fall-damage cause hook not confirmed.
- DeflectedDamage: maps to `entity_hurt_player` with `blocked=true`, already covered by PlayerHurtEntity; not a separate fire.

## Checks

- `rustfmt --edition 2024 --check crates/pumpkin-core/src/net/java/play/interact.rs`: exit 0.
- Not compiled (workers do not build). Batch `scripts/integrate.sh` pending.
