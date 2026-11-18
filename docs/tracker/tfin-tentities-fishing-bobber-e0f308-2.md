# Final rows from docs/tracker/tentities-fishing-bobber-e0f308.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: fishing_bobber.rs fishing_rod.rs type.rs

- `entity/type.rs:143` (bobber arm): STILL OPEN - needs a `from_loaded` constructor and owner-NBT
- `fishing_bobber.rs` (Lured event state): STILL OPEN - needs an additive `PlayerFishState::Lured`
- `item/items/fishing_rod.rs` (Fishing event on cast): STILL OPEN - the event is already fired and
Re-verified each STILL OPEN row against the decompiled FishingHook.java and FishingRodItem.java

## Result (FINAL PASS)

All three remaining rows are VERIFIED FIXED against decompiled vanilla; no source change was
needed this pass. No generated `pumpkin-data` field or missing subsystem is involved, so no row
is BLOCKED-DATA.

- Row 1 `entity/type.rs` bobber arm: VERIFIED FIXED. `from_type` has a `FISHING_BOBBER` arm that
  calls `FishingBobberEntity::from_loaded(entity)` (type.rs:269-271). The constructor
  (fishing_bobber.rs:148-172) defines both tracked values and leaves the owner as
  `ProjectileOwner::none()`, so an ownerless hook self-discards on its first tick. Matches
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/FishingHook.java:167-172`
  (`FishingHook.tick`: `owner == null` -> `discard()`). Vanilla always deserialises
  `EntityType.FISHING_BOBBER` to `FishingHook`, so the plain-`Entity` fallback is gone.
- Row 2 `fishing_bobber.rs` Lured event: VERIFIED FIXED. `PlayerFishState::Lured`
  (plugin/api/events/player/fish.rs:34) is fired at fishing_bobber.rs:791-801 when `wait_countdown`
  reaches 0, rolling `hook_countdown` to 20..=80 and zeroing it on cancel. Matches
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/FishingHook.java:415-427`
  (`State.LURED` fired, `timeUntilHooked = 0` on cancel). Enum is additive in native and both WIT
  hosts (v0_1, v0_2); a `Lured` round-trip unit test exists in each host's `events/player.rs`.
- Row 3 `item/items/fishing_rod.rs` Fishing event on cast: VERIFIED FIXED. The cancellable
  `PlayerFishEvent` with `PlayerFishState::Fishing` and the real hook UUID is fired before the
  throw sound and bails on cancel, clearing `player.fishing_bobber` (fishing_rod.rs:89-108).
  Matches `.vendor/vanilla-decompiled/net/minecraft/world/item/FishingRodItem.java:43-70`
  (build hook, fire `State.FISHING`, `isCancelled` -> `player.fishing = null` + PASS, then throw
  sound / spawn / single ITEM_USED award).

No tests added: all three are event/ordering paths needing a live `World`/`Player` harness, and
the only pure seam (WIT `Lured` conversion) already has round-trip tests in both host crates.
