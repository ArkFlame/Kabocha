# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-28526c.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

exact vanilla match or already fixed elsewhere in the checkout, so no STILL OPEN rows remain.

## Result
Re-verified all five rows against the vendored decompiled vanilla under
`.vendor/vanilla-decompiled/net/minecraft/`. No source change was needed: every row is an exact
vanilla match or already fixed elsewhere in the checkout, so no STILL OPEN rows remain.

- empty-collection set semantics (memory/mod.rs:255-258, 333-335 / value.rs:24,81,152,170):
  VERIFIED. Both `Brain.setMemoryInternal` overloads null any empty `Collection` then clear the
  slot (`world/entity/ai/Brain.java:214-262`, `isEmptyCollection` at 573-575), and the load path
  routes through them (`Brain.java:205-212`). Pumpkin's `MemoryValue::is_empty_collection`
  clearing matches; the tracker's suggested deletion would have broken parity, so it was kept.
- setMemoryInternal unregistered put (memory/mod.rs:252-254): VERIFIED. `getMemorySlotIfPresent`
  returns null for unregistered types (`world/entity/ai/Brain.java:157-159`) and both overloads
  guard with `if (slot != null)` (`Brain.java:216,241`); `set_with_expiry` early-returns on a
  missing slot. Match.
- negative ttl (memory/mod.rs:124-131): VERIFIED. `MemorySlot.hasExpired()` is
  `timeToLive <= 0L` and `tick()` clears on `hasExpired()`
  (`world/entity/ai/memory/MemorySlot.java:16-24,61-63`). Pumpkin's `self.ttl <= 0` matches,
  including negative values.
- dump ttl wording (memory/mod.rs:136-145): VERIFIED. `DebugBrainDump.accept` passes `timestamp`
  as the displayed ttl (`util/debug/DebugBrainDump.java:158-160`) while `MemorySlot.toString`
  prints the real ttl (`world/entity/ai/memory/MemorySlot.java:69-72`). Pumpkin deliberately
  prints the real ttl and the comment documents that vanilla quirk. No change.
- VISIBLE_ADULT_PIGLIN_COUNT / VISIBLE_ADULT_HOGLIN_COUNT (memory/types.rs:122-123): FIXED and
  VERIFIED. Declared with correct ids 79/80 and names
  (`world/entity/ai/memory/MemoryModuleType.java:116-117`). Hoglins: `HoglinSpecificSensor`
  writes both counts filtering out babies and non-piglin/hoglin
  (`world/entity/ai/sensing/HoglinSpecificSensor.java:41-57`, hoglin_specific.rs:80-115) and
  `piglins_outnumber_hoglins` reads `piglin_count > hoglin_count + 1`
  (`world/entity/monster/hoglin/HoglinAi.java:174-182`, hoglin.rs:38-62). Piglins:
  `refresh_visible_adult_counts` stores both counts, counting brutes as piglins
  (`world/entity/ai/sensing/PiglinSpecificSensor.java:73-80,107-108`, piglin.rs:540-598) and
  `hoglins_outnumber_piglins` reads `hoglin_count > piglin_count + 1`
  (`world/entity/monster/piglin/PiglinAi.java:779-783`, piglin_ai.rs:74-78). Existing tests
  `hoglin_sensor_counts_only_visible_adults`, `outnumbered_by_piglins_matches_vanilla_boundary`,
  `hoglins_outnumber_piglins_uses_both_visible_counts` and
  `retreat_broadcasts_to_visible_adult_piglins` cover the behavior; no duplicate test added.

Compile-pitfall table re-read: no source code edited this pass, so no new imports, clones,
struct initializers, macro paths or clippy-pedantic concerns. No rustfmt needed.
