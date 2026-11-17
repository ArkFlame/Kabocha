# items / Goat Horn (RE-REVIEW-UNTRACKED)

Source reviewed: `crates/pumpkin-core/src/item/items/goat_horn.rs` (branch kab/titems-goat-horn).
Vanilla reference: `net.minecraft.world.item.GoatHornItem` (1.21.x). No `.vendor/` decompile in
this worktree, so parity was checked against the vanilla class behaviour.

## Fixed in this pass

- Game event source entity: Rust emitted `GameEvent::InstrumentPlay` with no context, vanilla does
  `level.gameEvent(GameEvent.INSTRUMENT_PLAY, player.position(), GameEvent.Context.of(player))`.
  Now emitted through `emit_game_event_with_context` with `source_entity: Some(player)`
  (`goat_horn.rs:67-73`), so sculk sensors/warden anger attribute the vibration to the player.
- Instrument lookup: the `CustomDataImpl` `instrument`/`Instrument` fallback was pre-1.20.5 legacy;
  vanilla `GoatHornItem.getInstrument(ItemStack)` reads only the `DataComponents.INSTRUMENT` holder
  and returns empty otherwise. The fallback was removed (`goat_horn.rs:36-41`), so a horn carrying
  only custom data now fails the use like vanilla instead of playing.

## Remaining gaps

1. **Use animation TOOT_HORN missing** (`crates/pumpkin-core/src/item/mod.rs:91-95`,
   `goat_horn.rs` has no hook). Vanilla `GoatHornItem.getUseAnimation` returns `UseAnim.TOOT_HORN`.
   `ItemBehaviour` has no use-animation hook at all, so the third-person raise animation is wrong
   for the whole item class (also affects spyglass/trident). This is an item-framework change, not a
   goat horn edit. Vanilla class `GoatHornItem#getUseAnimation`. Size M. Client-visible only.
2. **Per-stack use duration** (`goat_horn.rs:80-82`). Vanilla
   `getUseDuration(ItemStack, LivingEntity)` returns the *instrument's* duration, or 0 when the stack
   has no instrument. Rust returns `Instrument::PonderGoatHorn.use_duration_ticks()` unconditionally,
   which makes `ItemRegistry::get_use_duration` (`crates/pumpkin-core/src/item/registry.rs:125`)
   report a duration for an instrument-less horn. Currently masked because the actual use path reads
   `ItemStack::get_max_use_time` (`crates/pumpkin-data/src/item_stack/mod.rs:548-552`), which is
   already per-instrument. Fixing properly needs a stack-aware `get_use_duration` in
   `ItemBehaviour`. Size S.
3. **Datapack/custom instruments** (`goat_horn.rs:36-41`). `InstrumentImpl.instrument` is a string and
   `Instrument::from_name` only resolves vanilla instrument names, so a datapack-registered
   instrument on a goat horn plays nothing and loses its range/duration. Vanilla resolves the
   `Holder<Instrument>` from the registry. Needs a runtime instrument registry. Vanilla class
   `Instrument`/registry. Size M.
4. **Cooldown always applied in vanilla** (`goat_horn.rs:74-76`). Vanilla calls
   `player.getCooldowns().addCooldown(stack, instrument.useDuration())` unconditionally; Rust skips
   the call when duration is 0. Observable only for a zero-duration instrument, so no behavioural
   difference with vanilla data. Size S.
5. **Fail path has no interaction result** (`goat_horn.rs:37-41`). Vanilla returns
   `InteractionResultHolder.fail(stack)` when the horn has no instrument; `ItemBehaviour::normal_use`
   returns `()`, so the server sends nothing and only the client's own prediction swings. No small
   fix without an interaction-result model. Size M.

Checked and matching: sound id + `SoundCategory::Records`, volume `range / 16.0`, pitch 1.0,
sound/event played on use start after `startUsingItem`, `Stats.ITEM_USED` increment, cooldown ticks
= instrument use duration, registry cooldown group from the `use_cooldown` component.
