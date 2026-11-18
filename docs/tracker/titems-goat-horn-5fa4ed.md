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

## h2 parity sweep (status)
- S rows STILL OPEN - need a stack-aware use-duration trait method (API change); not changed this pass.

## Result
Verified all STILL OPEN gaps against the decompiled vanilla tree at
`.vendor/vanilla-decompiled/net/minecraft/`. This version folds the goat horn into
`world/item/InstrumentItem.java` (no separate `GoatHornItem.java`). No source file was edited:
no row has a correct fix inside this unit's editable files that does not need a framework,
generated-data, or cross-cutting change.

- 1 Used stat double increment: CONFIRMED, left OPEN. Vanilla `ServerGamePacketListenerImpl.handleUseItem`
  (net/minecraft/server/network/ServerGamePacketListenerImpl.java:2387-2441) never awards
  `Stats.ITEM_USED`; `InstrumentItem.use` (:47) awards it exactly once. Pumpkin awards it in
  `use_item.rs:40` on every right-click AND in ~40 item behaviours (`goat_horn.rs:79`, bow, bucket, ...);
  food relies solely on the packet increment. Removing the packet increment regresses every item that
  does not self-award, so the correct fix needs a net-layer audit of all self-incrementing items across
  files outside this unit. Local removal from goat_horn.rs was deliberately not done (unit doc forbids it).
- 2 TOOT_HORN use animation: OPEN, out of unit / client-only. Vanilla `InstrumentItem.getUseAnimation`
  (:65-68) returns `ItemUseAnimation.TOOT_HORN`; needs a use-animation hook on the `ItemBehaviour` trait
  (`item/mod.rs`, not an editable basename here) plus third-person pose plumbing.
- 3 Per-stack use duration: OPEN, out of unit. Vanilla `getUseDuration(ItemStack, LivingEntity)`
  (InstrumentItem.java:54-58) reads the stack instrument and returns 0 without one. Pumpkin's
  `ItemBehaviour::get_use_duration(&self)` has no stack parameter, so this needs a trait signature change
  (`item/mod.rs`). Masked today: all 8 vanilla instruments are 140 t, the live use path reads
  `ItemStack::get_max_use_time`, and `registry.rs:84` only uses the value to gate component cooldowns.
- 4 Instrument dispatch for non-goat-horn items: OPEN, out of unit. `item/registry.rs:69-75` is not an
  editable file in this unit; deleting the generic `InstrumentImpl` branch needs a check that nothing
  relies on it.
- 5 Datapack/custom instruments: OPEN, blocked. `pumpkin-data/src/generated/instrument.rs` is generated
  (forbidden to edit) and `Instrument::from_name` only resolves the 8 vanilla names; a real fix needs the
  synced instrument registry.
- 6 Cooldown when duration is 0: CLOSED as not-a-gap. The row premise is wrong: decompiled
  `InstrumentItem.use` (:42-45) guards with `if (instrumentCooldown > 0.0F)`, so `goat_horn.rs:74-78`'s
  `if duration > 0` already matches vanilla. No code change.
- 7 Fail interaction result: OPEN, out of unit. Vanilla returns `InteractionResult.FAIL` with no instrument
  (InstrumentItem.java:49-51); `ItemBehaviour::normal_use` returns `()`, so an interaction-result model is
  needed in the item framework.

No .rs file touched, so no rustfmt run and no unit test added (no behaviour change to test).

## Result
Re-verified every STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/world/item/InstrumentItem.java`
(this version folds the goat horn into `InstrumentItem`; `Instrument.java` and
`world/item/component/InstrumentComponent.java` confirm the component shape). The current HEAD already
contains the in-unit fixes from the pass-2 workers, so the earlier "no fix possible" verdict is stale:

- 1 Used stat double increment: CLOSED. `use_item.rs:39-42` no longer awards `ITEM_USED` generically on
  right-click; each path now awards once (`entity/living.rs:4986` on `Consumable` completion,
  `use_item.rs:168-169` on equip, `goat_horn.rs:102` in the instrument use path). Vanilla's
  `InstrumentItem.use` (`Stats.ITEM_USED`) awards exactly once. No further net-layer change needed.
- 2 TOOT_HORN use animation: OPEN, out of unit. Vanilla `InstrumentItem.getUseAnimation` returns
  `ItemUseAnimation.TOOT_HORN` (`:65-68`); needs a hook on `ItemBehaviour` (`item/mod.rs`, not an
  editable basename here) plus third-person pose plumbing. Client-visible only.
- 3 Per-stack use duration: CLOSED. `ItemBehaviour::get_use_duration(&self, &ItemStack)` and
  `ItemRegistry::get_use_duration` now exist; `goat_horn.rs:105-110` reads the stack instrument and
  returns 0 without one, matching `InstrumentItem.getUseDuration` (`:54-58`). Unit tests added.
- 4 Instrument dispatch for non-goat-horn items: CLOSED. `item/registry.rs` no longer has the generic
  `InstrumentImpl` dispatch; it uses `get_pumpkin_item(item.id)` with explicit spear / knowledge-book
  fallbacks, so only the registered `GoatHornItem` plays.
- 5 Datapack/custom instruments: CLOSED for the representable case. `InstrumentImpl::custom_definition`
  and `use_duration_ticks` (`data_component_impl/basic.rs:603-629`) decode an inline datapack
  instrument, and `goat_horn.rs` uses it for sound, range, duration and durability. A true
  `Holder<Instrument>` from the synced registry would be a generated-data / cross-crate change, out of unit.
- 6 Cooldown when duration is 0: CLOSED as not-a-gap. Vanilla guards `if (instrumentCooldown > 0.0F)`
  (`:42-45`), so `goat_horn.rs:99-101`'s `if duration > 0` matches.
- 7 Fail interaction result: CLOSED. `ItemBehaviour::use_result(&self, &ItemStack)` returns
  `BlockActionResult::Consume`/`Fail`; `goat_horn.rs:112-122` returns `Fail` without an instrument,
  matching `InstrumentItem.use` (`:49-51`). `item/registry.rs` skips cooldown side effects on `Fail`.
  Unit tests added.

No .rs file was edited this pass (all in-unit fixes are already present in HEAD), so no rustfmt run
and no new test; `goat_horn.rs` already carries tests for `get_use_duration` and `use_result`.
Only row 2 remains, and it needs a framework hook outside this unit.

## Result
Re-verified against `.vendor/vanilla-decompiled/net/minecraft/world/item/InstrumentItem.java` (goat horn
is folded into `InstrumentItem` in this version) and `world/level/Level.java`. Rows 1 and 3-7 were already
fixed at HEAD; row 1 re-checked in `net/java/play/use_item.rs:39-42` (the only remaining `ITEM_USED`
award there is the equip one at `:169`) and row 7 re-checked in `item/registry.rs:87-97` (cooldown side
effects skipped on `Fail`) plus `use_item.rs:80-87` (no swing on `Fail`).

- 1 CLOSED, unchanged.
- 2 TOOT_HORN use animation: STILL OPEN, out of unit. Needs a use-animation hook on `ItemBehaviour`
  (`item/mod.rs`, not an editable basename here). Client-visible only.
- 3-7 CLOSED, unchanged.

New in-unit parity bug fixed this pass: the horn sound was broadcast to the horn player as well.
Vanilla `InstrumentItem.play` (`InstrumentItem.java:73`) calls
`level.playSound(player, player, soundEvent, RECORDS, volume, 1.0F)`, whose first argument is the
excluded listener (`Level.java:1671-1686`), so the server never sends the sound packet back to the
blower (the vanilla client plays it locally). `goat_horn.rs` now uses
`World::play_sound_raw_expect(player, ...)` for built-in instruments, matching vanilla. The datapack
inline-instrument branch still uses `play_custom_sound`, which has no exclude-listener variant in
`world/mod.rs` (not an editable file here), so a custom horn sound still repeats for its player.

Files edited: `crates/pumpkin-core/src/item/items/goat_horn.rs`. No unit test: the changed seam is
`World` sound broadcast plus a `Player`, no pure seam exists in this crate for it.
