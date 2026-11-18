# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-item-items-goat-horn-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

single BLOCKED-DATA subsystem row (own/titems-goat-horn row 3). No unresolved rows remain.
rustfmt run applies. No unresolved rows remain.
except the one BLOCKED-DATA subsystem row. No unresolved rows.

## Result (final)

All rows closed against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/world/item/InstrumentItem.java`,
`.vendor/vanilla-decompiled/net/minecraft/world/item/Instrument.java` and
`.vendor/vanilla-decompiled/net/minecraft/world/entity/Pose.java`. The current
`crates/pumpkin-core/src/item/items/goat_horn.rs` already matches vanilla for
every in-scope row; no Rust edit was needed this pass, so no rustfmt applies.
The dispatch sites (`item/registry.rs`, `net/java/play/use_item.rs`) and the
component codec (`pumpkin-data/src/data_component_impl/basic.rs`) were re-read
to confirm.

- **items/Goat Horn** (AUDITED): VERIFIED (matches vanilla
  `world/item/InstrumentItem.java`). Instrument read from the stack component
  (`:60-62` / `goat_horn.rs:40`); `startUsingItem` -> `set_active_hand`
  (`:35` / `:66`); sound + `GameEvent.INSTRUMENT_PLAY` with
  `Context.of(player)` emitted together before damage (`:70-74` / `:68-96`);
  durability damage (`:37-40` / `:98-100`); cooldown only when useDuration > 0
  (`:42-45` / `:103-105`); one `ITEM_USED` stat (`:47` / `:106`);
  `Consume`/`Fail` result (`:48,50` / `:116-126`). Volume is `range / 16.0`
  in both (`:72` / `:54`).

- **own/titems-goat-horn-5fa4ed** (7 sub-rows): all VERIFIED (matches vanilla).
  1. Used statistic awarded once (`goat_horn.rs:106` vs `InstrumentItem.java:47`);
     the network handler no longer awards it (`net/java/play/use_item.rs:39-42`).
  2. TOOT_HORN use animation is client-only, driven by the synced USING_ITEM flag
     set in `set_active_hand` (`Pose.java:11-29`, `InstrumentItem.java:65-68`).
  3. Per-stack duration `get_use_duration(&ItemStack)` (`goat_horn.rs:109-114` vs
     `InstrumentItem.java:54-58`; tests `:146-205`).
  4. Dispatch via `get_pumpkin_item` with no generic `InstrumentImpl` branch
     (`registry.rs:61-85`).
  5. Inline custom instrument covers sound/range/duration/damage
     (`goat_horn.rs:48-53`, `basic.rs` codec).
  6. Cooldown guarded by `duration > 0` (`:103-105` vs `:42-45`).
  7. `use_result` returns `Fail` without an instrument (`:116-126` vs
     `:49-50`), with `registry.rs:89-95` skipping cooldown on `Fail`.

- **own/titems-goat-horn** (5 sub-rows):
  - Row 1 (TOOT_HORN use animation): VERIFIED (matches vanilla). Client-only; no
    server call site or synced pose, so an `ItemBehaviour` hook would be dead
    code (`InstrumentItem.java:65-68`, `Pose.java:11-29`).
  - Row 2 (per-stack use duration): FIXED (`goat_horn.rs:109-114` vs
    `InstrumentItem.java:54-58`).
  - Row 3 (registry-referenced datapack instruments): BLOCKED-DATA. Missing
    subsystem: resolution of `Holder<Instrument>` over `Registries.INSTRUMENT`
    (`Instrument.java:16-26,40-41`, its `CODEC`/`STREAM_CODEC`). No single
    generated `pumpkin-data` field is missing. `InstrumentImpl`
    (`pumpkin-data/src/data_component_impl/basic.rs:555-557`) keeps only a name
    string and discards the positive holder raw id, so registry-referenced
    datapack instruments cannot be resolved from a sync `ItemBehaviour` (no
    `DynamicRegistryManager` reach). Cross-crate protocol+data+core subsystem,
    out of scope.
  - Row 4 (cooldown): FIXED (`goat_horn.rs:103-105`, cooldown only when duration > 0).
  - Row 5 (fail interaction result): FIXED (`goat_horn.rs:116-126`, `use_result`
    returns `Fail`).

Final row state: every row is terminal (FIXED or VERIFIED matches vanilla) except
the single BLOCKED-DATA subsystem row (own/titems-goat-horn row 3). No unresolved rows remain.
