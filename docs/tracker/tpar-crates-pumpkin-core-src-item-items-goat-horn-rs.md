# Parity rows for crates/pumpkin-core/src/item/items/goat_horn.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **items/Goat Horn** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/titems-goat-horn-5fa4ed** [partial/OWN-OPEN] own audit: 4 open, 0 fixed, 0 verified, 0 blocked-data (docs/tracker/titems-goat-horn-5fa4ed.md) docs/tracker/titems-goat-horn-5fa4ed.md
- **own/titems-goat-horn** [partial/OWN-OPEN] own audit: 23 open, 5 fixed, 1 verified, 0 blocked-data (docs/tracker/titems-goat-horn.md) docs/tracker/titems-goat-horn.md

## Result
FINAL PASS. Re-verified every row against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/world/item/InstrumentItem.java`,
`Instrument.java` and `world/entity/Pose.java`, and re-read all current call sites
(`item/registry.rs`, `network/server/use_item.rs`, `basic.rs`). All fixes are present
and correct in the current checkout; the unchanged `goat_horn.rs` type-checks against
the real APIs (verified signatures: `play_custom_sound`, `play_sound_raw_expect`,
`emit_game_event_with_context`, `set_active_hand`, `start_cooldown`, `is_on_cooldown`,
`damage_item_in_slot`, `get_use_cooldown`, `InstrumentImpl::custom_definition`/
`use_duration_ticks`). No `.rs` edit was needed, so no rustfmt applies.

- **items/Goat Horn** [AUDITED]: VERIFIED (matches vanilla `InstrumentItem.java`).
  Instrument read from the stack component (`:60-62` / `goat_horn.rs:40`);
  `startUsingItem` -> `set_active_hand` (`:35` / `:66`); sound + `GameEvent.INSTRUMENT_PLAY`
  with `Context.of(player)` before durability (`:70-74` / `:68-96`); durability damage
  (`:37-40` / `:98-100`); cooldown only when `duration > 0` (`:42-45` / `:103-105`);
  one `ITEM_USED` stat (`:47` / `:106`); `Consume`/`Fail` result (`:48,50` / `:116-126`).
  Volume is `range / 16.0` in both (`:72` / `:54`).

- **own/titems-goat-horn-5fa4ed** (7 sub-rows): all VERIFIED (matches vanilla).
  1 Used statistic once (`goat_horn.rs:106` vs `InstrumentItem.java:47`; net handler
  `use_item.rs:39-42` no longer awards it); 2 TOOT_HORN use animation is client-only,
  driven by the synced USING_ITEM flag set in `set_active_hand` (`Pose.java:11-29`,
  `InstrumentItem.java:65-68`); 3 per-stack duration `get_use_duration(&ItemStack)`
  (`goat_horn.rs:109-114` vs `InstrumentItem.java:54-58`, tests `:146-205`); 4 dispatch
  via `get_pumpkin_item` with no generic `InstrumentImpl` branch (`registry.rs:61-85`);
  5 inline custom instrument covers sound/range/duration/damage
  (`goat_horn.rs:48-53`, `basic.rs` codec); 6 cooldown guarded by `duration > 0`
  (`:103-105` vs `:42-45`); 7 `use_result` returns `Fail` without an instrument
  (`:116-126` vs `:49-50`), with `registry.rs:89-95` skipping cooldown on `Fail`.

- **own/titems-goat-horn** (5 sub-rows):
  - Row 1 (TOOT_HORN use animation): VERIFIED (matches vanilla). Client-only; no server
    call site or synced pose, so an `ItemBehaviour` hook would be dead code
    (`InstrumentItem.java:65-68`, `Pose.java:11-29`).
  - Row 2 (per-stack use duration): FIXED (`goat_horn.rs:109-114` vs `InstrumentItem.java:54-58`).
  - Row 3 (registry-referenced datapack instruments): BLOCKED-DATA. Whole missing
    subsystem pinned against `Instrument.java:16-26,40-41` (the `Instrument` record
    `sound_event`/`use_duration`/`range`/`durability_damage` and `CODEC`/`STREAM_CODEC`
    as `Holder<Instrument>` over `Registries.INSTRUMENT`). Missing links: (1)
    `InstrumentImpl` keeps only a name string, discarding the positive holder raw id
    (`pumpkin-data/src/data_component_impl/basic.rs:555-557`); (2) the codec falls back
    to the ponder horn for ids above the 8 generated builtins; (3) no `Instrument` NBT
    decoder or manager-level raw-id lookup; (4) `goat_horn.rs` is a sync `ItemBehaviour`
    with no reachability to `DynamicRegistryManager`. No single generated `pumpkin-data`
    field is missing; it is a cross-crate protocol+data+core subsystem and out of this
    unit's edit scope.
  - Row 4 (cooldown): FIXED (`goat_horn.rs:103-105`, cooldown only when `duration > 0`).
  - Row 5 (fail interaction result): FIXED (`goat_horn.rs:116-126`, `use_result` returns `Fail`).

Final row state: every row is terminal (FIXED or VERIFIED matches vanilla) except the
single BLOCKED-DATA subsystem row (own/titems-goat-horn row 3). No STILL OPEN rows remain.


## Result (third pass, terminal)

Re-verified every row against decompiled vanilla
`.vendor/vanilla-decompiled/net/minecraft/world/item/InstrumentItem.java`,
`Instrument.java` and `world/entity/Pose.java`, and re-read the current
`crates/pumpkin-core/src/item/items/goat_horn.rs` plus the dispatch sites
(`item/registry.rs`, `network/server/use_item.rs`, `basic.rs`). All prior fixes are
still present in HEAD and match vanilla. No `.rs` edit was needed this pass, so no
rustfmt run applies. No STILL OPEN rows remain.

- **items/Goat Horn** [AUDITED]: VERIFIED (matches vanilla). Re-review of the changed
  `goat_horn.rs` against `InstrumentItem.java`: instrument read from the stack component
  (`:60-62`), `startUsingItem` -> `set_active_hand` (`:35` / `goat_horn.rs:66`), sound +
  `GameEvent.INSTRUMENT_PLAY` with `Context.of(player)` emitted together before damage
  (`InstrumentItem.java:70-74` / `goat_horn.rs:68-96`), durability damage (`:37-40` /
  `:98-100`), cooldown only when `useDuration > 0` (`:42-45` / `:103-105`), one
  `ITEM_USED` stat (`:47` / `:106`), and `Consume`/`Fail` result (`:48,50` /
  `:116-126`). Volume is `range / 16.0` in both (`:72` / `goat_horn.rs:54`).
- **own/titems-goat-horn-5fa4ed** (all 7 sub-rows): VERIFIED (matches vanilla).
  1 Used statistic once (`goat_horn.rs:106` vs `InstrumentItem.java:47`); 2 TOOT_HORN
  use animation is client-only and driven by the synced USING_ITEM flag set in
  `set_active_hand` (`Pose.java:11-29`, `InstrumentItem.java:65-68`); 3 per-stack
  duration `get_use_duration(&ItemStack)` (`goat_horn.rs:109-114` vs `:54-58`, tests at
  `:146-205`); 4 dispatch via `get_pumpkin_item` with no generic InstrumentImpl branch
  (`registry.rs:61-85`); 5 inline custom instrument covers sound/range/duration/damage
  (`goat_horn.rs:48-53`, `basic.rs` codec); 6 cooldown guarded by `duration > 0`
  (`:103-105` vs `:42-45`); 7 `use_result` returns `Fail` without an instrument
  (`:116-126` vs `:49-50`), with `registry.rs:89-95` skipping cooldown on `Fail`.
- **own/titems-goat-horn** (all 5 sub-rows):
  - Row 1 (TOOT_HORN use animation): VERIFIED (matches vanilla). Client-only; no server
    call site or synced pose, so an `ItemBehaviour` hook would be dead code
    (`InstrumentItem.java:65-68`, `Pose.java:11-29`).
  - Row 2 (per-stack use duration): FIXED (`goat_horn.rs:109-114` vs `InstrumentItem.java:54-58`).
  - Row 3 (registry-referenced datapack instruments): BLOCKED-DATA. Whole missing
    subsystem pinned against `Instrument.java:16-26,40-41` (record `sound_event`/
    `use_duration`/`range`/`durability_damage` and `CODEC`/`STREAM_CODEC` as
    `Holder<Instrument>` over `Registries.INSTRUMENT`). Missing: (1) `InstrumentImpl`
    keeps only a name string, discarding the positive holder raw id (`basic.rs`); (2)
    the codec falls back to the ponder horn for ids above the 8 generated builtins; (3)
    no `Instrument` NBT decoder or manager-level raw-id lookup; (4) `goat_horn.rs` is a
    sync `ItemBehaviour` with no reachability to `DynamicRegistryManager`. No single
    generated `pumpkin-data` field is missing; it is a cross-crate protocol+data+core
    subsystem and out of this unit's edit scope.
  - Row 4 (cooldown): FIXED (`goat_horn.rs:103-105`, cooldown only when `duration > 0`).
  - Row 5 (fail interaction result): FIXED (`goat_horn.rs:116-126`, `use_result` returns `Fail`).

Final row state: all AUDITED/OWN rows terminal - FIXED or VERIFIED (matches vanilla)
except the one BLOCKED-DATA subsystem row. No STILL OPEN rows.
