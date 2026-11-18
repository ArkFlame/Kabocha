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

## h6 gap status

- Row 1 TOOT_HORN use animation: STILL OPEN (item framework has no use-animation hook, M).
- Row 2 per-stack use duration: STILL OPEN (needs stack-aware get_use_duration in ItemBehaviour, S but framework-wide).
- Row 3 datapack instruments: STILL OPEN (needs runtime instrument registry, M).
- Row 4 cooldown always applied: STILL OPEN (no observable difference with vanilla data).
- Row 5 fail interaction result: STILL OPEN (no interaction-result model, M).


## Result

Re-verified all STILL OPEN rows against decompiled vanilla
`.vendor/vanilla-decompiled/net/minecraft/world/item/InstrumentItem.java`
(the 1.21.x name for the old `GoatHornItem`; `GoatHornItem.java` is absent from the decompile).

- Row 1 (TOOT_HORN use animation): STILL OPEN. Vanilla `getUseAnimation` returns
  `ItemUseAnimation.TOOT_HORN` (`InstrumentItem.java:65-68`), but `ItemBehaviour` has no
  use-animation hook. Needs a hook in `item/mod.rs` plus third-person client packet plumbing,
  files outside this unit. Client-visible only.
- Row 2 (per-stack use duration): STILL OPEN. Vanilla `getUseDuration(ItemStack, LivingEntity)`
  returns the instrument's duration or 0 (`InstrumentItem.java:54-58`). Pumpkin's
  `ItemBehaviour::get_use_duration(&self)` has no stack parameter and is read by
  `item/registry.rs:127`, both outside this unit. Masked in practice by
  `ItemStack::get_max_use_time`, which is already per-instrument.
- Row 3 (datapack/custom instruments): STILL OPEN. Vanilla resolves a `Holder<Instrument>` from
  the `INSTRUMENT` registry via `Instrument.CODEC` (`InstrumentItem.java:60-63`). Pumpkin stores
  only a string and resolves it through the vanilla-only `Instrument::from_name`. Needs a runtime
  instrument registry and datapack codec (cross-crate/generated data), outside this unit.
- Row 4 (cooldown): FIXED. The previous assumption was wrong: decompiled vanilla adds the
  cooldown only when `useDuration > 0` (`InstrumentItem.java:42-45`:
  `if (instrumentCooldown > 0.0F) { addCooldown(stack, floor(useDuration * 20)) }`), not
  unconditionally. The unconditional `player.start_cooldown` and its comment were corrected to
  `if duration > 0 { ... }` in `goat_horn.rs:74-78`. All vanilla instruments have duration 7.0s
  (140 ticks), so no observable difference with vanilla data, but the code now matches vanilla.
- Row 5 (fail interaction result): STILL OPEN. Vanilla returns `InteractionResult.FAIL` when the
  stack has no instrument (`InstrumentItem.java:49-50`); `normal_use` returns `()`, and there is no
  interaction-result model in `ItemBehaviour` (`item/mod.rs`, outside this unit).

No unit test added: the behaviour is entirely side-effectful (world sound/event, player cooldown
and stats) with no pure seam in `goat_horn.rs`; extracting a `duration > 0` helper would be a
formula-mirroring test, which the rules exclude.

## Result

Re-verified all STILL OPEN rows against decompiled vanilla
`.vendor/vanilla-decompiled/net/minecraft/world/item/InstrumentItem.java`.

- Row 1 (TOOT_HORN use animation): STILL OPEN. Vanilla `getUseAnimation` returns
  `ItemUseAnimation.TOOT_HORN` (`InstrumentItem.java:65-68`), but `ItemBehaviour` has no
  use-animation hook. Needs a trait hook in `item/mod.rs` plus third-person client packet plumbing,
  both outside this unit. Client-visible only.
- Row 2 (per-stack use duration): ADDRESSED OUTSIDE UNIT. `ItemBehaviour` now exposes
  `get_use_duration(&self, stack: &ItemStack)` (`item/mod.rs:109`), `ItemRegistry::get_use_duration`
  is stack-aware (`item/registry.rs:136`), and `goat_horn.rs` implements it from the instrument
  component (140 ticks for builtins, 0 without, own value for inline datapack instruments), matching
  vanilla `getUseDuration` (`InstrumentItem.java:54-58`).
- Row 3 (datapack/custom instruments): STILL OPEN. Vanilla resolves a `Holder<Instrument>` from the
  `INSTRUMENT` registry (`InstrumentItem.java:60-63`). Pumpkin stores only a string resolved via the
  vanilla-only `Instrument::from_name`, with an inline encoding for custom instruments; a registry
  reference to a datapack instrument still resolves to no duration. Needs a runtime instrument
  registry and codec (cross-crate/generated data), outside this unit.
- Row 4 (cooldown): FIXED (prior pass, confirmed). Cooldown added only when `duration > 0`, matching
  `InstrumentItem.java:42-45`.
- Row 5 (fail interaction result): ADDRESSED OUTSIDE UNIT. `ItemBehaviour` now has
  `use_result(&self, stack: &ItemStack)` (`item/mod.rs:61`), wired through `ItemRegistry`
  (`item/registry.rs:83`); `goat_horn.rs` returns `Fail` when the stack has no resolvable instrument,
  matching vanilla `InteractionResult.FAIL` (`InstrumentItem.java:49-50`).

Additional fix this pass: moved the `InstrumentPlay` game event emit to run with the sound, before
durability damage, so the call order matches vanilla `play()` being invoked before `hurtAndBreak`
(`InstrumentItem.java:35-40,70-75`). The use position is precomputed, so there was no observable
delta; the ordering now matches vanilla exactly.

No unit test added: the use path is entirely side-effectful (world sound/event, player cooldown,
hand stack, stats) with no pure seam. The existing tests already cover the pure duration and
use-result helpers.

## Result (latest re-review)

Re-verified every STILL OPEN row against
`.vendor/vanilla-decompiled/net/minecraft/world/item/InstrumentItem.java`.
The current `goat_horn.rs` already carries the framework-integrated fixes; no
further source edit was needed in this unit this pass.

- Row 1 (TOOT_HORN use animation): STILL OPEN. Vanilla `getUseAnimation`
  returns `ItemUseAnimation.TOOT_HORN` (`InstrumentItem.java:65-68`). `ItemBehaviour`
  has no use-animation hook and `SwingAnimationType` has no `TootHorn` variant;
  generated data, `item/mod.rs` and third-person client packet plumbing are all
  outside this unit. Client-visible only.
- Row 2 (per-stack use duration): FIXED. `ItemBehaviour::get_use_duration(&self,
  &ItemStack)` now takes the stack (`item/mod.rs:109`); `GoatHornItem` returns the
  instrument's duration in ticks or 0 (`goat_horn.rs:106-111`), matching vanilla
  `getUseDuration` (`InstrumentItem.java:54-58`). `ItemStack::get_max_use_time`
  and `ItemRegistry::get_use_duration` consume the stack-aware value.
- Row 3 (datapack/custom instruments): STILL OPEN. Vanilla resolves a
  `Holder<Instrument>` from the `INSTRUMENT` registry (`InstrumentItem.java:23-27,60-63`).
  Pumpkin encodes an inline custom definition in the component string and
  resolves built-ins with `Instrument::from_name`; registry-loaded datapack
  instruments need a runtime instrument registry and datapack codec, which are
  cross-crate/generated-data changes outside this unit.
- Row 4 (cooldown): FIXED. Vanilla adds the cooldown only when the instrument's
  use duration is positive (`InstrumentItem.java:42-45`); `goat_horn.rs:100-102`
  matches with `if duration > 0 { player.start_cooldown(..) }`.
- Row 5 (fail interaction result): FIXED. Vanilla returns `InteractionResult.FAIL`
  for an instrument-less horn (`InstrumentItem.java:49-50`). `ItemBehaviour` now
  has a `use_result(&self, &ItemStack) -> BlockActionResult` hook wired in
  `item/registry.rs`, and `GoatHornItem::use_result` returns `Fail`/`Consume`
  (`goat_horn.rs:113-123`), so the registry skips use-cooldown on FAIL.

Three pure unit tests in `goat_horn.rs` cover stack-aware duration, custom
instrument acceptance and instrument-less FAIL. Could not run them: worker shell
is denied; the orchestrator compiles once after all workers finish.

Rows 2, 4, 5 fixed; rows 1, 3 left open as framework/cross-crate work outside
this unit. No source edit made this pass.

## Result

Re-verified both remaining STILL OPEN rows against
`.vendor/vanilla-decompiled/net/minecraft/world/item/InstrumentItem.java`.
No source edit made: both rows require files outside this unit (only `goat_horn.rs`
is editable here), and the current `goat_horn.rs` already matches vanilla for
every fixable behaviour.

- Row 1 (TOOT_HORN use animation): STILL OPEN. Vanilla `getUseAnimation` returns
  `ItemUseAnimation.TOOT_HORN` (`InstrumentItem.java:65-68`), but `ItemBehaviour`
  (`item/mod.rs:24-`) has no use-animation hook and there is no third-person
  animation packet path - a framework + generated-data + client-packet change
  outside this unit. Client-visible only.
- Row 3 (datapack/custom instruments via registry): STILL OPEN. Vanilla
  `getInstrument` resolves a `Holder<Instrument>` from the component
  (`InstrumentItem.java:60-63`), whereas Pumpkin stores a string resolved through
  `Instrument::from_name` plus an inline custom encoding in
  `pumpkin-data/src/data_component_impl/basic.rs` (outside this unit); resolving a
  registry-loaded datapack instrument needs a runtime instrument registry and
  datapack codec (cross-crate/generated data), outside this unit.

`goat_horn.rs` already implements the in-unit fixes (stack-aware duration,
`duration > 0` cooldown, `Fail` use-result, context-attributed game event) and
carries pure unit tests for the duration/custom/use-result seams; no new seam
exists for the two open client/registry rows. Worker shell is denied, so the
tests could not be run here; the orchestrator compiles once after all workers.
