# items / Goat Horn (RE-REVIEW-UNTRACKED)
Source reviewed: `crates/pumpkin-core/src/item/items/goat_horn.rs` on branch kab/titems-goat-horn-5fa4ed.
Vanilla reference: `net.minecraft.world.item.GoatHornItem` (1.21.x) + `net.minecraft.world.item.Instrument` /
`InstrumentComponent`. No `.vendor/` decompile in this worktree, so parity was checked against the
vanilla class behaviour and `crates/pumpkin-data/src/generated/instrument.rs` (generated from `assets/`,
which wins on values).
Last source change: 0b6775b7f "feat(items): match vanilla goat horn instrument lookup and game event source".

## Re-review verdict for the previous two fixes (both still correct)
- Game event source entity: `goat_horn.rs:67-73` emits `GameEvent::InstrumentPlay` via
  `emit_game_event_with_context` with `source_entity: Some(player)`, matching
  `serverLevel.gameEvent(GameEvent.INSTRUMENT_PLAY, player.position(), GameEvent.Context.of(player))`.
- Instrument lookup: `goat_horn.rs:37-42` reads only the `InstrumentImpl` data component
  (`DataComponents.INSTRUMENT`); the legacy `CustomData` instrument fallback is gone, so a horn without
  the component passes like vanilla's `InteractionResultHolder.pass`.

## Still matching vanilla
Instrument sound mapping (`item.goat_horn.sound.0..7` per instrument), `SoundCategory::Records`,
volume = `range / 16.0`, pitch 1.0, sound range 256 blocks, use start before sound
(`set_active_hand` -> `ITEM_INTERACT_START` vibration), cooldown group from the `use_cooldown`
component falling back to the item key (goat horn has no `use_cooldown` component in
`crates/pumpkin-data/src/generated/item.rs:45152`), cooldown ticks = 140 (instrument use duration),
default instrument = `minecraft:ponder_goat_horn`, and stack use duration via
`ItemStack::get_max_use_time` (`crates/pumpkin-data/src/item_stack/mod.rs:548-552`).

## Remaining gaps
1. **Used statistic is incremented twice** — `goat_horn.rs:77` adds `StatisticCategory::Used`, but
   `crates/pumpkin-core/src/net/java/play/use_item.rs:40` already increments it for every right-click
   (Bedrock: `net/bedrock/play/player_action.rs:94,191`). Vanilla increments `Stats.ITEM_USED` once,
   inside `GoatHornItem.use`; the packet handler does not. Fix belongs in the net layer (drop the
   unconditional increment there, keep the vanilla per-item award, audit the other behaviours that
   already self-increment), not as a local removal here.
   Vanilla class: `GoatHornItem#use` / `Player#awardStat`. Size M.
2. **TOOT_HORN use animation missing** — vanilla `GoatHornItem#getUseAnimation` returns
   `UseAnim.TOOT_HORN`; `ItemBehaviour` (`crates/pumpkin-core/src/item/mod.rs:39-95`) has no
   use-animation hook, so third-person pose is item-framework work (also spyglass/trident).
   Client-visible only. Size M.
3. **Per-stack use duration** — vanilla `GoatHornItem#getUseDuration(ItemStack, LivingEntity)`
   returns the stack instrument's duration (0 without an instrument); `goat_horn.rs:80-82` returns
   `Instrument::PonderGoatHorn.use_duration_ticks()` unconditionally. Masked today: every vanilla
   instrument is 140 t and the real use path reads `ItemStack::get_max_use_time`, which is
   per-instrument; `ItemRegistry::get_use_duration` only gates the component cooldown
   (`item/registry.rs:84`). Needs a stack-aware trait method. Size S.
4. **Instrument dispatch for non-goat-horn items** — `crates/pumpkin-core/src/item/registry.rs:69-75`
   runs `GoatHornItem::normal_use_with_rotation` for *any* item stack carrying an
   `InstrumentImpl` component (e.g. a stick given with `[minecraft:instrument=...]`). Vanilla only
   the `GoatHornItem` item class plays, so this branch is over-broad; it is unreachable for the real
   goat horn because that item is registered (`item/registry.rs:73`). Deleting it needs a check that
   nothing relies on the generic path. Size S.
5. **Datapack/custom instruments** — `InstrumentImpl.instrument` is a string and
   `Instrument::from_name` (`crates/pumpkin-data/src/generated/instrument.rs:17`) only resolves the
   8 vanilla names, so a datapack instrument plays nothing and loses range/duration. Vanilla resolves
   the `Holder<Instrument>` from the synced registry. Size M.
6. **Cooldown skipped when duration is 0** — `goat_horn.rs:74-76` guards `duration > 0`; vanilla calls
   `getCooldowns().addCooldown(...)` unconditionally. Only observable with a zero-duration
   instrument; no vanilla-data difference. Size S.
7. **No fail interaction result** — vanilla returns `InteractionResultHolder.fail(stack)` when the
   horn has no instrument; `ItemBehaviour::normal_use` returns `()`, so the server cannot respond
   fail/consume. Needs an interaction-result model. Size M.

Vanilla shown values: use duration 140 t for all 8 instruments, range 256 blocks, cooldown 140 t
(`~7 s`), sound category RECORDS, one `ITEM_USED` stat per play.
