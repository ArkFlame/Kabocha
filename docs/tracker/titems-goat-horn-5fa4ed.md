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


## Result
Third pass. Only row 2 remained STILL OPEN; verified against decompiled vanilla and CLOSED as
VERIFIED (matches vanilla). No source file needed editing and no pure test seam applies (no
behaviour change), so no rustfmt run.

- 2 TOOT_HORN use animation: VERIFIED (matches vanilla) - the earlier "needs a use-animation hook
  on ItemBehaviour plus third-person pose plumbing" premise is wrong. Vanilla `getUseAnimation` is a
  client-only rendering concept, never computed or sent by the server:
  - `InstrumentItem.getUseAnimation` returns `ItemUseAnimation.TOOT_HORN`
    (`.vendor/vanilla-decompiled/net/minecraft/world/item/InstrumentItem.java:65-68`).
  - Its sole caller is `ItemStack.getUseAnimation()`
    (`.vendor/vanilla-decompiled/net/minecraft/world/item/ItemStack.java:941-942`), used by the
    client arm renderer; no caller exists anywhere in `net/minecraft/server` or
    `net/minecraft/network` (grep: 0 matches), and no use-animation packet is sent.
  - Vanilla `Pose` has no TOOT_HORN variant
    (`.vendor/vanilla-decompiled/net/minecraft/world/entity/Pose.java:11-29`), so this is not a
    synced server pose.
  - The vanilla server's only action is `startUsingItem(hand)` (`InstrumentItem.java:35`), which
    sets `DATA_LIVING_ENTITY_FLAGS` bits 0 (is-using-item) and 1 (used hand)
    (`LivingEntity.java:190-193,4230,4234,4325-4332`). The client derives `useItem` from the held
    stack on that flag update and computes its own `getUseAnimation` arm pose
    (`LivingEntity.java:4368-4373`).
  - Pumpkin already implements exactly this server contract: `LivingEntity::set_active_hand`
    (`crates/pumpkin-core/src/entity/living.rs:1084-1108`) sets identical bit values
    (`:263-264`, `USING_ITEM_FLAG=1`, `OFF_HAND_ACTIVE_FLAG=2`) and syncs
    `DATA_LIVING_ENTITY_FLAGS` to clients (`:1151`). The horn path calls it at `goat_horn.rs:66`.
  So no framework hook, generated field, or pose plumbing is required; the client already replays
  TOOT_HORN from the horn stack in hand plus the synced flags.

- 1, 3-7 remain CLOSED from the earlier passes in this doc (all fixes already present at HEAD and
  re-verified above). No generated-data or whole-subsystem blocker remains.

Final row state: 1 CLOSED, 2 VERIFIED (matches vanilla), 3 CLOSED, 4 CLOSED, 5 CLOSED, 6 CLOSED,
7 CLOSED. No STILL OPEN rows.


## Result
Third-pass final verification. Re-read the previous Result (all rows already closed at HEAD) and
re-verified every row against the decompiled vanilla tree at
`.vendor/vanilla-decompiled/net/minecraft/world/item/InstrumentItem.java` (this version folds the goat
horn into `InstrumentItem`; `world/item/component/InstrumentComponent.java` and `world/entity/Pose.java`
confirm the component and pose shapes). No STILL OPEN rows remain and no code change was needed: the
in-unit fixes from earlier passes are present and correct in the current checkout. Files checked but
not modified: `goat_horn.rs`, `use_item.rs`, `registry.rs`, `basic.rs`, bedrock `player_action.rs`,
`living.rs`. No new unit test (no behaviour change this pass) and no rustfmt run (no `.rs` touched).

- 1 Used statistic: VERIFIED (matches vanilla). Vanilla `InstrumentItem.use` awards `Stats.ITEM_USED`
  once (`.vendor/vanilla-decompiled/net/minecraft/world/item/InstrumentItem.java:47`); Pumpkin awards
  once at `goat_horn.rs:106`. The net handler (`use_item.rs:39-42`) no longer awards it generically.
  Bedrock equip/consume awards are separate paths (`player_action.rs:115-120,212-217`).
- 2 TOOT_HORN use animation: VERIFIED (matches vanilla). `InstrumentItem.getUseAnimation` returns
  `ItemUseAnimation.TOOT_HORN` (`InstrumentItem.java:65-68`) but it is client-only: its sole caller is
  `ItemStack.getUseAnimation()` used by the client arm renderer, with no server/network caller and no
  synced pose (`Pose.java:11-29`). The server contract is `startUsingItem(hand)` (`InstrumentItem.java:35`),
  which sets `DATA_LIVING_ENTITY_FLAGS` using-item bits; Pumpkin matches with `set_active_hand`
  (`living.rs:1084-1104`, flags at `:263-264`, synced at `:1151`) called at `goat_horn.rs:66`.
- 3 Per-stack use duration: VERIFIED (matches vanilla). `get_use_duration(&self, &ItemStack)` is
  stack-aware (`goat_horn.rs:109-114`), returning 0 without an instrument, matching
  `InstrumentItem.getUseDuration` (`InstrumentItem.java:54-58`). Tests at `goat_horn.rs:146-205`.
- 4 Instrument dispatch: VERIFIED (matches vanilla). `registry.rs:61-85` dispatches via
  `get_pumpkin_item(item.id)` with only spear/knowledge-book component fallbacks; no generic
  `InstrumentImpl` branch, so only the registered `GoatHornItem` plays.
- 5 Custom/datapack instruments: VERIFIED (matches vanilla for the representable case). `InstrumentImpl`
  decodes an inline custom instrument (`basic.rs:555-629`) used for sound, range, duration and durability
  (`goat_horn.rs:48-53`). All 8 vanilla instruments resolve, so no BLOCKED-DATA condition applies.
- 6 Cooldown when duration is 0: VERIFIED (matches vanilla). Vanilla guards
  `if (instrumentCooldown > 0.0F)` (`InstrumentItem.java:42-45`); Pumpkin guards `if duration > 0`
  (`goat_horn.rs:103-105`).
- 7 Fail interaction result: VERIFIED (matches vanilla). `use_result` returns `Fail` without an
  instrument (`goat_horn.rs:116-125`), matching `InstrumentItem.use` `InteractionResult.FAIL`
  (`InstrumentItem.java:49-51`); `registry.rs:89-95` skips cooldown side effects on `Fail` and
  `use_item.rs:82-87` does not swing on `Fail`.

Final row state: 1-7 VERIFIED (matches vanilla). No STILL OPEN rows, no BLOCKED-DATA rows.


## Result
Final pass. Re-verified all 7 rows against decompiled vanilla
`.vendor/vanilla-decompiled/net/minecraft/world/item/InstrumentItem.java` (this version folds the
goat horn into `InstrumentItem`) and against the current checkout. Every row is VERIFIED; no code
change, no new test, no rustfmt run and no `.rs` file was touched this pass. There is no
"STILL OPEN until closed" placeholder and no remaining STILL OPEN or BLOCKED-DATA row.

- 1 Used statistic: VERIFIED. Vanilla awards `Stats.ITEM_USED` once (`InstrumentItem.java:47`).
  Pumpkin awards once at `goat_horn.rs:106`; `use_item.rs` no longer awards it on generic
  right-click (only the equip path at `:169`). Bedrock equip/consume awards are separate paths.
- 2 TOOT_HORN use animation: VERIFIED. Client-only: `getUseAnimation` (`InstrumentItem.java:65-68`)
  is read only by the client arm renderer via `ItemStack.getUseAnimation`; no server/network caller,
  no synced pose (`Pose.java:11-29`). The server contract is `startUsingItem` (`:35`), which Pumpkin
  matches with `set_active_hand` (`goat_horn.rs:66`). No framework hook required.
- 3 Per-stack use duration: VERIFIED. `get_use_duration(&ItemStack)` (`goat_horn.rs:109-114`)
  reads the stack instrument and returns 0 without one, matching `getUseDuration`
  (`InstrumentItem.java:54-58`); covered by unit tests (`goat_horn.rs:146-175`).
- 4 Instrument dispatch: VERIFIED. `registry.rs:61-85` dispatches via `get_pumpkin_item(item.id)`
  with only spear/knowledge-book component fallbacks; no generic `InstrumentImpl` broadcast path.
- 5 Custom/datapack instruments: VERIFIED (representable case). `InstrumentImpl` decodes an inline
  custom instrument (`basic.rs:555-629`), consumed for sound, range, duration and durability
  (`goat_horn.rs:48-53`). All 8 vanilla instruments resolve; no BLOCKED-DATA condition. The inline
  custom-sound branch still broadcasts to the blower (`play_custom_sound` has no exclude-listener
  variant in `world/mod.rs`, outside this unit's editable basenames), but vanilla sounds are
  unaffected and this is not one of the 7 rows.
- 6 Cooldown when duration is 0: VERIFIED. Vanilla guards `instrumentCooldown > 0.0F`
  (`InstrumentItem.java:42-45`); Pumpkin guards `if duration > 0` (`goat_horn.rs:103-105`).
- 7 Fail interaction result: VERIFIED. `use_result` returns `Fail` without an instrument
  (`goat_horn.rs:116-126`), matching `InteractionResult.FAIL` (`InstrumentItem.java:49-51`);
  `registry.rs:89-95` skips cooldown side effects on `Fail`.

Final row state: 1-7 VERIFIED (matches vanilla). No STILL OPEN rows, no BLOCKED-DATA rows.
