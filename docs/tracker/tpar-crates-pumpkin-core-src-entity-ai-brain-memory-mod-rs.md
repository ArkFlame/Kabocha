# Parity rows for crates/pumpkin-core/src/entity/ai/brain/memory/mod.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-en-28526c** [partial/OWN-OPEN] own audit: 5 open, 1 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-en-28526c.md) docs/tracker/taudit-crates-pumpkin-core-src-en-28526c.md

## Result

All five rows from the referenced own-audit are closed. Each was re-checked directly against
the vendored decompiled vanilla under `.vendor/vanilla-decompiled/net/minecraft/`. No source
edit was needed: every row is an exact vanilla match or was already fixed in the checkout.
Compile-pitfall table re-read: no source code edited, so no new imports, clones, struct
initializers, macro paths or clippy-pedantic concerns. No new unit test: there is no genuine
behaviour gap to test, and the piglin/hoglin count behaviour is already covered by the existing
tests `hoglin_sensor_counts_only_visible_adults`, `hoglins_outnumber_piglins_uses_both_visible_counts`
and `retreat_broadcasts_to_visible_adult_piglins`.

- empty-collection set semantics (memory/value.rs:23-26, 81-83, 152-154, 170-172; mod.rs:255-258,
  333-335): VERIFIED. `world/entity/ai/Brain.java:214-262` nulls any empty `Collection` then calls
  `slot.clear()` in both `setMemoryInternal` overloads (`isEmptyCollection` at 573-575), and the
  load path `setMemoryInternal(MemoryMap.Value)` routes through them (Brain.java:205-212). Pumpkin's
  `MemoryValue::is_empty_collection` clearing and its use in `set_with_expiry` / `load_packed` match.
  The original suggested fix (delete the special case) would have broken parity; kept as-is.
- setMemoryInternal unregistered put (memory/mod.rs:252-254): VERIFIED. `getMemorySlotIfPresent`
  returns null for an unregistered type (`world/entity/ai/Brain.java:157-159`) and both overloads
  guard with `if (slot != null)` (Brain.java:215-216, 240-241). Pumpkin's `set_with_expiry`
  early-returns on a missing slot, matching vanilla.
- negative ttl (memory/mod.rs:124-131): VERIFIED. `MemorySlot.hasExpired()` is `timeToLive <= 0L`
  and `tick()` clears on `hasExpired()`
  (`world/entity/ai/memory/MemorySlot.java:16-24,61-63`). Pumpkin's `self.ttl <= 0` matches,
  including negative values.
- dump ttl wording (memory/mod.rs:136-145): VERIFIED. The comment is accurate for the newer dump:
  `util/debug/DebugBrainDump.java:158-160` passes `timestamp` (game time) as the displayed ttl while
  `MemorySlot.toString` prints the real ttl (`world/entity/ai/memory/MemorySlot.java:69-72`). Pumpkin
  deliberately prints the real ttl and the comment documents the vanilla quirk. No behaviour change.
- VISIBLE_ADULT_PIGLIN_COUNT / VISIBLE_ADULT_HOGLIN_COUNT (memory/types.rs:122-123): FIXED and
  VERIFIED. Declared as ids 79/80 with names `visible_adult_piglin_count` /
  `visible_adult_hoglin_count` (`world/entity/ai/memory/MemoryModuleType.java:116-117`). Hoglins:
  `entity/ai/brain/sensing/hoglin_specific.rs:80-114` writes both counts filtering babies and
  non-piglin/hoglin (`world/entity/ai/sensing/HoglinSpecificSensor.java:41-57`) and
  `entity/mob/hoglin.rs:36-62` reads them with `piglin_count > hoglin_count + 1`
  (`world/entity/monster/hoglin/HoglinAi.java:174-182`). Piglins:
  `entity/mob/piglin.rs:538-601` counts visible adults including brutes as piglins
  (`world/entity/ai/sensing/PiglinSpecificSensor.java:73-80,107-108`) and
  `entity/mob/piglin_ai.rs:71-78` reads them with `hoglin_count > piglin_count + 1`
  (`world/entity/monster/piglin/PiglinAi.java:779-783`). Exact match.

## Result (third pass, independent re-verification)

Re-checked every row against `.vendor/vanilla-decompiled/net/minecraft/`. Final statuses:
empty-collection set semantics VERIFIED (Brain.java:214-262); setMemoryInternal unregistered put
VERIFIED (Brain.java:157-159,215-216,240-241); negative ttl VERIFIED
(world/entity/ai/memory/MemorySlot.java:16-24,61-63); dump ttl wording VERIFIED
(util/debug/DebugBrainDump.java:158-160); VISIBLE_ADULT_PIGLIN/HOGLIN counts FIXED and VERIFIED
(world/entity/ai/sensing/HoglinSpecificSensor.java:41-57,
world/entity/ai/sensing/PiglinSpecificSensor.java:73-80,107-108,
world/entity/monster/hoglin/HoglinAi.java:174-182,
world/entity/monster/piglin/PiglinAi.java:779-783). No STILL OPEN rows remain.
