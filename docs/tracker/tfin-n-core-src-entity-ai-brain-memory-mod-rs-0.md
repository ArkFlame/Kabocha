# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-entity-ai-brain-memory-mod-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

world/entity/monster/piglin/PiglinAi.java:779-783). No STILL OPEN rows remain.

## Result
All five rows independently re-verified against the vendored decompiled vanilla under
`.vendor/vanilla-decompiled/net/minecraft/`. No source edit was needed: every row is already an
exact vanilla match in the checkout. Compile-pitfall table re-read: no source code edited, so no
new imports, clones, struct initializers, macro paths or clippy-pedantic concerns, and no rustfmt
needed. No new unit test: there is no behaviour gap, and the piglin/hoglin count behaviour is
already covered by `hoglin_sensor_counts_only_visible_adults`,
`hoglins_outnumber_piglins_uses_both_visible_counts`, `retreat_broadcasts_to_visible_adult_piglins`
and `outnumbered_by_piglins_matches_vanilla_boundary`.

- empty-collection set semantics (memory/value.rs:23-26,81-83,152-154,170-172; mod.rs:255-258,333-335): VERIFIED.
  `world/entity/ai/Brain.java:573-575` `isEmptyCollection` is true for any empty `Collection`;
  both `setMemoryInternal` overloads null it out then call `slot.clear()` (Brain.java:214-262), and
  the load path routes through them (Brain.java:205-212). Pumpkin's `MemoryValue::is_empty_collection`
  clearing in `set_with_expiry` / `load_packed` matches. The original "delete the special case"
  suggestion would have broken parity, so it was kept as-is.
- setMemoryInternal unregistered put (memory/mod.rs:252-254): VERIFIED. `getMemorySlotIfPresent`
  returns null for an unregistered type (Brain.java:157-159) and both overloads guard `slot != null`
  (Brain.java:215-216,240-241). Pumpkin's `set_with_expiry` early-returns on a missing slot, matching.
- negative ttl (memory/mod.rs:124-131): VERIFIED. `MemorySlot.hasExpired()` is `timeToLive <= 0L`
  and `tick()` clears on it (`world/entity/ai/memory/MemorySlot.java:16-24,61-63`). Pumpkin's
  `self.ttl <= 0` matches, including negative values.
- dump ttl wording (memory/mod.rs:136-145): VERIFIED. `util/debug/DebugBrainDump.java:158-160`
  passes `timestamp` (game time) as the displayed ttl while `MemorySlot.toString` prints the real
  ttl (`world/entity/ai/memory/MemorySlot.java:69-72`). Pumpkin deliberately prints the real ttl
  and the comment documents the vanilla quirk. No behaviour change.
- VISIBLE_ADULT_PIGLIN_COUNT / VISIBLE_ADULT_HOGLIN_COUNT (memory/types.rs:122-123): FIXED and VERIFIED.
  Declared as ids 79/80 with names `visible_adult_piglin_count` / `visible_adult_hoglin_count`
  (`world/entity/ai/memory/MemoryModuleType.java:116-117`). Hoglins: the sensor
  `entity/ai/brain/sensing/hoglin_specific.rs:80-114` writes both counts filtering babies and
  non-piglin/hoglin (`world/entity/ai/sensing/HoglinSpecificSensor.java:41-57`) and
  `entity/mob/hoglin.rs:38-61` reads with `piglin_count > hoglin_count + 1`
  (`world/entity/monster/hoglin/HoglinAi.java:174-182`). Piglins:
  `entity/mob/piglin.rs:539-603` counts visible adults including brutes as piglins
  (`world/entity/ai/sensing/PiglinSpecificSensor.java:73-80,107-108`) and
  `entity/mob/piglin_ai.rs:74-78` reads with `hoglin_count > piglin_count + 1`
  (`world/entity/monster/piglin/PiglinAi.java:779-783`). Exact match; the counts are held on the
  piglin entity rather than brain memory because piglins use goal AI, a documented but
  behaviourally identical divergence. No STILL OPEN rows remain.
