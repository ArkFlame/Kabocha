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

## Final gap status (supersedes the historical STILL OPEN notes below)

- Row 1 TOOT_HORN use animation: VERIFIED - no server-side gap; client-only animation.
- Row 2 per-stack use duration: FIXED - stack-aware get_use_duration in ItemBehaviour.
- Row 3 datapack instruments: BLOCKED-DATA - runtime registry-backed Holder<Instrument> resolution subsystem missing (see final Result).
- Row 4 cooldown: FIXED - cooldown added only when duration > 0.
- Row 5 fail interaction result: FIXED - use_result returns Fail for an instrument-less horn.


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

## Result (re-review, both editable files in scope)

Re-verified both remaining STILL OPEN rows against decompiled vanilla
`InstrumentItem.java`, `InstrumentComponent.java` and `Instrument.java`. Both
are editable-file-adjacent now (`basic.rs` was added to this unit) but neither
has a fix that lives wholly inside `basic.rs` / `goat_horn.rs`.

- Row 1 (TOOT_HORN use animation): STILL OPEN. Vanilla `getUseAnimation`
  returns `ItemUseAnimation.TOOT_HORN` (`InstrumentItem.java:65-68`). No
  use-animation hook exists in `ItemBehaviour` (`item/mod.rs`), no `TootHorn`
  variant in the generated `SwingAnimationType`, and no third-person animation
  packet path. All three are outside this unit; client-visible only.
- Row 3 (datapack/custom instruments): STILL OPEN. Inline datapack instruments
  are already decoded into `InstrumentImpl::from_custom` by the network codec
  (`pumpkin-protocol/src/codec/data_component.rs:2859-2888`) and resolved for
  sound/range/duration in `basic.rs` + `goat_horn.rs`, matching vanilla's
  `Instrument.DIRECT_CODEC` holder. The remaining case, a registry-referenced
  datapack instrument whose id exceeds the 8 builtins, resolves to the default
  ponder horn (`data_component.rs:2889-2897`); fixing it needs a runtime
  `INSTRUMENT` registry and datapack codec in `pumpkin-protocol` + generated
  data (outside this unit). `basic.rs` alone cannot resolve a registry id.

No source edit made: rows 2, 4, 5 remain fixed; rows 1 and 3 require framework,
protocol-codec, generated-data or client-packet changes outside this unit. The
pure seams (duration / custom / use-result) are already covered by the three
existing unit tests in `goat_horn.rs` and the two in `basic.rs`; no new pure
seam exists for the two open rows. Worker shell is denied, so tests were not run
here; the orchestrator compiles once after all workers finish.


## Result (second pass, expanded scope)

Re-verified both remaining STILL OPEN rows against decompiled vanilla
`InstrumentItem.java`, `ItemStack.java` and `Instrument.java`. No `.rs` source
edit was needed: neither row has a minimal in-scope fix.

- Row 1 (TOOT_HORN use animation): VERIFIED, no server-side gap. A full grep of
  `.vendor/vanilla-decompiled/net/minecraft` shows `getUseAnimation` is only
  declared in item classes and delegated by `ItemStack.getUseAnimation()`
  (`ItemStack.java:941-942`); there is no server call site. The third-person
  animation is derived by the vanilla client from its own item definition once
  it sees the USING_ITEM living-entity flag. Pumpkin already sets that flag and
  syncs `DATA_LIVING_ENTITY_FLAGS` in `set_active_hand`
  (`entity/living.rs:1084,1103,1151`), which `goat_horn.rs:66` calls. No
  use-animation field or packet exists on the server, so an `ItemBehaviour`
  hook would be dead code. No change.
- Row 3 (registry-referenced datapack instruments): BLOCKED. Vanilla
  `Instrument.CODEC` / `STREAM_CODEC` are holders over `Registries.INSTRUMENT`
  (`Instrument.java:40-41`), so a stack can carry a registry reference rather
  than an inline definition. Pumpkin's codec
  (`pumpkin-protocol/src/codec/data_component.rs:2889-2897`) resolves positive
  holder ids only against the 8 static generated builtins and falls back to the
  ponder horn. Resolving a registry-referenced datapack instrument requires
  preserving the registry id through `InstrumentImpl`, decoding it at use time
  via the runtime `DynamicRegistryManager` + instrument NBT, and wiring that
  into the item use path: a whole cross-crate subsystem plus generated
  instrument data, far beyond a minimal fix.

Rows 2, 4, 5 remain fixed. No new pure-test seam exists for either open row.
No shell/cargo/git was used to verify; the orchestrator compiles once after all
workers finish.

## Result (third pass, final)

Re-verified the two remaining rows against decompiled vanilla. No `.rs` source edit
was needed or possible in scope; both rows are now terminal (no STILL OPEN remains).
No shell/cargo/git was run; the orchestrator compiles once after all workers.

- Row 1 (TOOT_HORN use animation): VERIFIED, matches vanilla; no server-side gap.
  `getUseAnimation` is declared only on item classes and reached via
  `ItemStack.getUseAnimation()`
  (`.vendor/vanilla-decompiled/net/minecraft/world/item/ItemStack.java:941-942`,
  `InstrumentItem.java:65-68`, `ItemUseAnimation.java:20`); a full grep of
  `.vendor/vanilla-decompiled/net/minecraft` found no server call site beyond that
  delegate. The third-person TOOT_HORN animation is a client rendering decision
  derived from the USING_ITEM living-entity flag. Pumpkin sets that flag and syncs
  `DATA_LIVING_ENTITY_FLAGS` in `LivingEntity::set_active_hand`
  (`crates/pumpkin-core/src/entity/living.rs:1084,1103,1151`), which the goat-horn use
  path calls. No use-animation field or packet exists on the server, so an
  `ItemBehaviour` hook would be dead code. No change.
- Row 3 (registry-referenced datapack instruments): BLOCKED-DATA. Missing subsystem:
  runtime resolution of a registry-backed `Holder<Instrument>` at use time. Vanilla
  `Instrument.CODEC` / `STREAM_CODEC` are holders over `Registries.INSTRUMENT`
  (`.vendor/vanilla-decompiled/net/minecraft/world/item/Instrument.java:40-41`) with
  fields `sound_event`, `use_duration`, `range`, `durability_damage`
  (`Instrument.java:16-26`). Pumpkin discards the holder's positive raw id and falls
  back to the ponder horn
  (`crates/pumpkin-protocol/src/codec/data_component.rs:2890-2896`), `InstrumentImpl`
  stores only a name string
  (`crates/pumpkin-data/src/data_component_impl/basic.rs:555-557`), and there is no
  instrument NBT decoder nor any path from the item-use code to `DynamicRegistryManager`
  to read a datapack instrument's sound/duration/range/durability. The registry storage
  does exist (`crates/pumpkin-core/src/data/datapack/dynamic_registry_loader.rs:17`,
  `dynamic_registry_manager.rs:63-67`), but preserving the id + decoding the instrument
  NBT + wiring it into the use path is a cross-crate subsystem (protocol codec + data
  component + core use path), far beyond a minimal in-unit fix. No generated
  pumpkin-data field is missing; the gap is the resolution subsystem itself.

Rows 2, 4, 5 remain fixed from earlier passes. No new pure-test seam exists for either
terminal row (Row 1 is client-visible only; Row 3 needs a runtime registry).
No `.rs` file was touched this pass, so rustfmt was not applicable.


## Result (expanded-scope re-review, all granted files inspected)

Granted files this pass: `basic.rs`, `data_component.rs`,
`dynamic_registry_loader.rs`, `dynamic_registry_manager.rs`, `goat_horn.rs`.
Re-verified both terminal rows against decompiled vanilla. No `.rs` source edit
was made: neither row has a fix that is both correct and contained enough to be
safe in a shared checkout with a single end-of-batch compile gate. No STILL OPEN
rows remain.

- Row 1 (TOOT_HORN use animation): VERIFIED, matches vanilla; no server-side gap.
  `getUseAnimation` is declared only on item classes and reached via
  `ItemStack.getUseAnimation()`
  (`.vendor/vanilla-decompiled/net/minecraft/world/item/ItemStack.java:941-942`,
  `InstrumentItem.java:65-68`, `ItemUseAnimation.java:20`); a full grep of
  `.vendor/vanilla-decompiled/net/minecraft` found no server call site beyond
  that delegate. The third-person TOOT_HORN animation is a client rendering
  decision derived from the USING_ITEM living-entity flag, which Pumpkin already
  sets and syncs in `LivingEntity::set_active_hand`. No use-animation field or
  packet exists on the server, so an `ItemBehaviour` hook would be dead code.

- Row 3 (registry-referenced datapack instruments): BLOCKED-DATA. Whole missing
  subsystem, confirmed against all newly granted files. Vanilla
  `Instrument.CODEC` / `STREAM_CODEC` are holders over `Registries.INSTRUMENT`
  (`.vendor/vanilla-decompiled/net/minecraft/world/item/Instrument.java:40-41`)
  carrying `sound_event`, `use_duration`, `range`, `durability_damage`
  (`Instrument.java:16-26`). Pumpkin loads `minecraft:instrument` datapack JSON
  into the registry (`dynamic_registry_loader.rs:17`), but the four pieces that
  would resolve a positive holder id are all absent:
  1. `InstrumentImpl` stores only a name string
     (`pumpkin-data/src/data_component_impl/basic.rs:555-557`), so the positive
     raw id is not preserved.
  2. The codec discards that id and falls back to the ponder horn for ids above
     the 8 builtins (`pumpkin-protocol/src/codec/data_component.rs:2890-2896`).
  3. There is no NBT decoder for the `Instrument` record anywhere in `crates/`
     (grep for `use_duration` / `durability_damage` finds only the inline-custom
     network codec at `data_component.rs:2831-2898`); raw-id entry data can only
     be fetched by `entry_id` on `DatapackManager`
     (`data/datapack/mod.rs:1009-1016`), and `DynamicRegistry` has
     `get_entry_data_by_raw_id` (`dynamic_registry_manager.rs:64-68`) but the
     manager/datapack wrappers do not expose it.
  4. `goat_horn.rs` is a sync `ItemBehaviour` with no server/datapack context,
     so resolving the id at use time needs new reachability plumbing.
  No generated `pumpkin-data` field is missing; the blocker is the resolution
  subsystem itself (id preservation + Instrument NBT decoder + manager raw-id
  lookup + use-path wiring) across protocol, data and core crates, far beyond a
  minimal in-unit edit.


## Result (third pass, terminal classification)

Re-verified every row against decompiled vanilla
`.vendor/vanilla-decompiled/net/minecraft/world/item/InstrumentItem.java` and
`Instrument.java`. No `.rs` source edit was made or is needed: both previously open
rows are terminal and no STILL OPEN row remains.

- Row 1 (TOOT_HORN use animation): VERIFIED, matches vanilla; no server-side gap.
  `InstrumentItem.getUseAnimation` returns `ItemUseAnimation.TOOT_HORN`
  (`InstrumentItem.java:65-68`), but it is reached only via
  `ItemStack.getUseAnimation()` and has no server call site. The client derives the
  third-person animation from the USING_ITEM living-entity flag, which Pumpkin sets
  and syncs in `LivingEntity::set_active_hand`. No use-animation field or packet
  exists on the server, so an `ItemBehaviour` hook would be dead code. No change.
- Row 2 (per-stack use duration): FIXED. `GoatHornItem::get_use_duration(&ItemStack)`
  returns the instrument duration in ticks or 0 (`goat_horn.rs:109-114`), matching
  `InstrumentItem.java:54-58`.
- Row 3 (registry-referenced datapack instruments): BLOCKED-DATA. Whole missing
  subsystem, pinned against `Instrument.java:16-26,40-41` (the `Instrument` record
  `sound_event`/`use_duration`/`range`/`durability_damage` and
  `CODEC`/`STREAM_CODEC` as `Holder<Instrument>` over `Registries.INSTRUMENT`).
  Four links are absent: (1) `InstrumentImpl` keeps only a name string, discarding
  the positive holder raw id (`basic.rs:555-557`); (2) the codec falls back to the
  ponder horn for ids above the 8 generated builtins
  (`data_component.rs:2890-2896`); (3) there is no `Instrument` NBT decoder and no
  manager-level raw-id lookup (`dynamic_registry_manager.rs` exposes
  `DynamicRegistry::get_entry_data_by_raw_id` at :64 but only
  `get_entry_data` by entry id at :231); (4) `goat_horn.rs` is a sync
  `ItemBehaviour` with no reachability to `DynamicRegistryManager`. Preserving the
  id, decoding the NBT, exposing raw-id lookup and wiring the use path is a
  cross-crate subsystem (protocol + data + core); no single generated
  `pumpkin-data` field is missing. Cannot be fixed within this unit.
- Row 4 (cooldown): FIXED. Cooldown added only when `duration > 0`
  (`goat_horn.rs:103-105`), matching `InstrumentItem.java:42-45`.
- Row 5 (fail interaction result): FIXED. `use_result` returns `Fail` for an
  instrument-less horn (`goat_horn.rs:116-126`), matching `InstrumentItem.java:49-50`.

Rows 2, 4, 5 fixed in code; Row 1 VERIFIED; Row 3 BLOCKED-DATA. No STILL OPEN
remains. The pure seams already carry unit tests in `goat_horn.rs` and `basic.rs`.
Worker shell/cargo/git are denied; the orchestrator compiles once after all workers.


## Result (FINAL PASS - terminal classification, all rows closed)

Re-verified every row against the decompiled vanilla sources in .vendor/:
.vendor/vanilla-decompiled/net/minecraft/world/item/InstrumentItem.java and
.vendor/vanilla-decompiled/net/minecraft/world/item/Instrument.java.
No .rs source edit was needed this pass: the in-unit fixes are already present in
goat_horn.rs, basic.rs and the protocol codec, and the two previously open rows
are terminal. No STILL OPEN row remains. No shell/cargo/git was run; the orchestrator
compiles once after all workers finish.

- Row 1 (TOOT_HORN use animation): VERIFIED, matches vanilla; no server-side gap.
  Vanilla InstrumentItem.getUseAnimation returns ItemUseAnimation.TOOT_HORN
  (InstrumentItem.java:65-68), but it is reached only through
  ItemStack.getUseAnimation() and has no server call site. The third-person
  TOOT_HORN animation is a client rendering decision derived from the USING_ITEM
  living-entity flag, which Pumpkin sets and syncs in LivingEntity::set_active_hand
  (called by goat_horn.rs:66). No use-animation field or packet exists on the
  server, so an ItemBehaviour hook would be dead code. No change.
- Row 2 (per-stack use duration): FIXED. GoatHornItem::get_use_duration(&ItemStack)
  returns the instrument duration in ticks (140 for builtins, 0 without, own value
  for an inline custom instrument) at goat_horn.rs:109-114, matching vanilla
  InstrumentItem.getUseDuration (InstrumentItem.java:54-58).
- Row 3 (registry-referenced datapack instruments): BLOCKED-DATA. Missing subsystem:
  runtime resolution of a registry-backed Holder<Instrument> at use time. Vanilla
  Instrument.CODEC / STREAM_CODEC are holders over Registries.INSTRUMENT
  (.vendor/vanilla-decompiled/net/minecraft/world/item/Instrument.java:40-41) and
  the record carries sound_event/use_duration/range/durability_damage
  (Instrument.java:16-26). Four links are absent in Pumpkin: (1) InstrumentImpl
  stores only a name string and discards the positive holder raw id
  (pumpkin-data/.../data_component_impl/basic.rs:555-557); (2) the codec falls back
  to the ponder horn for ids above the 8 generated builtins
  (pumpkin-protocol/src/codec/data_component.rs:2889-2897); (3) no Instrument NBT
  decoder exists in pumpkin-core (grep for sound_event/use_duration/
  durability_damage returns no match; dynamic_registry_loader.rs:17 only lists
  the minecraft:instrument registry key) and the manager exposes raw-id lookup on
  DynamicRegistry (dynamic_registry_manager.rs:63-68) but not a decoded instrument;
  (4) goat_horn.rs is a sync ItemBehaviour with no reachability to
  DynamicRegistryManager. Preserving the raw id, decoding the instrument NBT,
  exposing manager-level raw-id data lookup and wiring the use path is a cross-crate
  subsystem (protocol + data + core), far beyond a minimal in-unit edit. No single
  generated pumpkin-data field is missing; the whole resolution subsystem is.
- Row 4 (cooldown): FIXED. Cooldown added only when duration > 0
  (goat_horn.rs:101-105), matching vanilla InstrumentItem.use
  (InstrumentItem.java:42-45).
- Row 5 (fail interaction result): FIXED. use_result returns Fail for an
  instrument-less horn (goat_horn.rs:116-126), matching vanilla
  InteractionResult.FAIL (InstrumentItem.java:49-50).

Pure unit tests cover every fixable seam: three in goat_horn.rs (stack-aware
duration, custom instrument acceptance, instrument-less FAIL) and two in
basic.rs (inline instrument round trip, builtin duration). Worker shell is
denied so the tests were not run here; the orchestrator compiles once after all
workers finish. No .rs file was touched this pass, so rustfmt was not applicable.
