# audit / crates/pumpkin-core/src/entity/ai/brain/memory/mod.rs

Reviewed the whole file (603 lines) plus `memory/value.rs`, `memory/types.rs` and the
`brain/mod.rs` wrappers. No decompiled vanilla source is vendored in this worktree, so
vanilla statements below are from `net.minecraft.world.entity.ai.Brain` /
`Brain.MemorySlot` (1.21.x) and are marked `confirm` where a decompiled check is needed.

## Markers

The tracker's "3 TODO/FIXME/vanilla markers" are false positives: `mod.rs:154`, `:533`,
`:562` are a comment and two test names containing the word "vanilla". The only real
marker in this module tree is `memory/types.rs:123` (TODO: nothing fills
`VISIBLE_ADULT_PIGLIN_COUNT` / `VISIBLE_ADULT_HOGLIN_COUNT`).

## Resolved in this audit

- `PackedMemories::into_nbt` (`memory/mod.rs:357-372`) always wrote `memories: {}` even
  when nothing was encoded. Vanilla `Brain.serializeToNbt` only puts `memories` when the
  encoded compound is non-empty (`if (!compoundTag2.isEmpty())`), and `Brain.loadMemories`
  treats a missing `memories` as empty, so the empty compound was pure save-format drift.
  Fixed in commit `d88b3d53d` (tests `empty_brain_omits_the_memories_compound`,
  `a_default_brain_is_brain_dead_and_idle`). Note: `mob/mod.rs:880-884` still always
  writes the outer `Brain` tag for every mob; vanilla only writes it from brain-owning
  mobs, but `Brain: {}` is what vanilla's own callers produce when nothing was encoded,
  so only the outer per-mob condition remains unverified.

## Verified equivalent to vanilla

- `MemorySlot::tick` (`:124`) clears at `ttl == 0` after decrementing, exactly the
  countdown `Brain.MemorySlot.tick` performs; `can_expire` = `ttl != Long.MAX_VALUE`;
  `clear` resets both value and ttl (`:119`).
- `check` / `has_value` (`:188`, `:200`) implement `Brain.checkMemory`: unregistered ->
  false for every status; registered with/without value matches VALUE_ABSENT/PRESENT.
- `set` / `set_with_expiry` / `set_optional` (`:223`-`:257`) match `Brain.setMemory`,
  `setMemoryWithExpiry` (null value erases, ttl reset on every write).
- `erase`, `clear_all`, `is_value` match `eraseMemory`, `clearMemories`,
  `isMemoryValue`.
- Tick ordering (`brain/mod.rs:330`): memories age -> sensors -> start behaviors -> tick
  behaviors, same as `Brain.tick` (`forgetOutdatedMemories` first).
- `pack` / `load_packed` (`:287`, `:309`) skip non-serializable types and empty values,
  matching `Brain.serializeToNbt` / `loadMemories` (only the empty compound container was off).
- `debug_lines` sorting + 255-char per-line truncation mirrors
  `StringUtil.truncateStringIfNecessary(.., 255, true)`.

## Remaining gaps

| file:line | vanilla | suggested fix | size |
|---|---|---|---|
| memory/mod.rs:243, :321 (+ value.rs:23-26) | `Brain.setMemory` stores an empty list/set as a value, so `hasMemoryValue` stays true; Pumpkin clears the slot instead (`MemoryValue::is_empty_collection`) | confirm against decompiled `Brain.setMemory` 1.21.x, then delete the empty-collection special case and its tests | M |
| memory/mod.rs:240-242 | `Brain.setMemoryInternal` does a plain map put, so a write to a type missing from the registered list still lands and later reads REGISTERED; Pumpkin drops it | keep (all brain types are registered in `BrainProvider`), or grow the slot on write if a wiring bug is ever suspected | S |
| memory/mod.rs:155-163 | `String.length()` is UTF-16 units; Pumpkin counts Unicode scalar values, so astral characters truncate later than vanilla | count UTF-16 units | S |
| memory/mod.rs:124-131 | vanilla tests `timeToLive == 0`; a negative ttl would never expire there, Pumpkin clears at `<= 0` | no vanilla caller passes a negative ttl; leave, or mirror `== 0` exactly | S |
| memory/mod.rs:136-141 | comment claims the vanilla dump prints the game time in place of the ttl while the code prints the ttl | confirm `Brain.MemorySlot.toString` / 1.21.2 `DebugBrainDump`; if the dump shows an expiry game time, pass it in | S-M |
| memory/types.rs:123 | `PiglinAi`/`HoglinAi` fill `VISIBLE_ADULT_PIGLIN_COUNT` and `VISIBLE_ADULT_HOGLIN_COUNT` from the visible-adult sensor and use them to decide when to retreat | populate both counts in the piglin/hoglin sensor pass | M |

## Gap status

- FIXED: UTF-16 truncation of memory dump text, entity/ai/brain/memory/mod.rs:155-170, test `truncate_counts_utf16_units`.
- STILL OPEN: empty-collection set semantics (memory/mod.rs:243,321), setMemoryInternal unregistered put (memory/mod.rs:240-242), negative ttl (memory/mod.rs:124-131, no vanilla caller: leave by design), dump ttl wording (memory/mod.rs:136-141), VISIBLE_ADULT_PIGLIN/HOGLIN counts unpopulated (memory/types.rs:118-123).

## Result

Re-checked every STILL OPEN row against the vendored decompiled vanilla under
`.vendor/vanilla-decompiled/net/minecraft/`. No source change was needed in this unit's files
(`types.rs`, `value.rs`): the original gaps were guesses made before decompiled source was
available and most do not hold against 1.21.x vanilla.

- empty-collection set semantics (mod.rs:243,321 / value.rs:23-26): NOT A GAP. Both
  `Brain.setMemoryInternal` overloads do `if (isEmptyCollection(value)) value = null;` then
  `slot.clear()` (Brain.java:214-262), and the load path `setMemoryInternal(MemoryMap.Value)`
  routes through those same overloads (Brain.java:205-212). Pumpkin's
  `MemoryValue::is_empty_collection` special case matches vanilla exactly. The old suggested
  fix (delete the special case) was wrong and would have broken parity; kept as-is.
- setMemoryInternal unregistered put (mod.rs:240-242): NOT A GAP. Vanilla
  `getMemorySlotIfPresent` returns null for an unregistered type (Brain.java:157-159) and
  both `setMemoryInternal` overloads return early when the slot is null (Brain.java:215-216,
  240-241). Pumpkin drops writes to unregistered slots, matching vanilla. Also in mod.rs,
  outside this unit.
- negative ttl (mod.rs:124-131): NOT A GAP. Vanilla `MemorySlot.hasExpired()` is
  `timeToLive <= 0L` (MemorySlot.java:61-63) and `tick()` clears when `hasExpired()`
  (MemorySlot.java:16-24). Pumpkin's `self.ttl <= 0` matches, including negative values.
  Also in mod.rs, outside this unit.
- dump ttl wording (mod.rs:136-141): the stale comment is actually accurate for the newer
  dump. `DebugBrainDump.accept(type, value, timeToLive)` passes `timestamp` as the displayed
  ttl (DebugBrainDump.java:158-160), while `MemorySlot.toString` prints the real ttl
  (MemorySlot.java:69-72). Pumpkin deliberately prints the real ttl; behaviour is unchanged.
  Comment and code are in mod.rs, outside this unit, so left open.
- VISIBLE_ADULT_PIGLIN/HOGLIN counts (types.rs:122-123): the two memory types are declared
  with the correct ids and names (confirmed against MemoryModuleType.java:116-117). Vanilla
  fills them in `PiglinSpecificSensor` (lines 107-108) and `HoglinSpecificSensor` (lines
  56-57), and reads them in PiglinAi.java:780-781 / HoglinAi.java:179-180 for retreat.
  Populating them requires editing the sensor tick files, outside this unit's allowed files
  (types.rs, value.rs), so left open.

No unit test added: value.rs has no genuine gap to test, and mod.rs already covers
empty-collection clearing with `setting_an_empty_collection_clears_the_slot`.

## Result (independent re-verification)

Re-checked all five STILL OPEN rows against
`.vendor/vanilla-decompiled/net/minecraft/` and the current `value.rs` / `types.rs`.
No source change was required in this unit's files; the earlier Result above is confirmed.

- empty-collection set semantics (value.rs:23-26, 81-83, 152-154, 170-172): NOT A GAP.
  `Brain.setMemoryInternal` nulls out any empty `Collection` then calls `slot.clear()` in
  both overloads (Brain.java:217-219, 242-244), so Pumpkin's `MemoryValue::is_empty_collection`
  clearing matches vanilla. No test added: asserting `is_empty()` on an empty `Vec` is a
  formula-mirroring test the repo rules forbid.
- setMemoryInternal unregistered put (mod.rs:240-242): OUT OF UNIT. Vanilla
  `getMemorySlotIfPresent` returns null and both overloads return early (Brain.java:157-159,
  215-216, 240-241), so Pumpkin's drop matches.
- negative ttl (mod.rs:124-131): OUT OF UNIT. Vanilla `MemorySlot.hasExpired()` is `<= 0`.
- dump ttl wording (mod.rs:136-141): OUT OF UNIT (comment and code both in mod.rs).
- VISIBLE_ADULT_PIGLIN_COUNT / VISIBLE_ADULT_HOGLIN_COUNT (types.rs:122-123): declared
  correctly (ids 79/80, names match MemoryModuleType.java:116-117). Vanilla populates them
  only in `PiglinSpecificSensor` (107-108) and `HoglinSpecificSensor` (56-57); those sensor
  files are outside this unit.


## Result (final independent re-verification)

Re-checked every STILL OPEN row against the vendored decompiled vanilla and the current
`value.rs` / `types.rs`. No source change was required in this unit's files; the two files
already match 1.21.x vanilla exactly.

- empty-collection set semantics (value.rs:23-26, 81-83, 152-154, 170-172): NOT A GAP.
  `Brain.setMemoryInternal` nulls out any empty `Collection` then calls `slot.clear()` in both
  overloads (Brain.java:214-237, 239-262; `isEmptyCollection` at 573-575), and the load path
  `setMemoryInternal(MemoryMap.Value)` routes through those same overloads (Brain.java:205-212).
  Pumpkin's `MemoryValue::is_empty_collection` clearing matches vanilla. The tracker's suggested
  fix (delete the special case) would have broken parity; kept as-is.
- setMemoryInternal unregistered put (mod.rs:240-242): OUT OF UNIT. Vanilla
  `getMemorySlotIfPresent` returns null for an unregistered type and both overloads return early
  (Brain.java:215-216, 240-241), so Pumpkin's drop matches.
- negative ttl (mod.rs:124-131): OUT OF UNIT. Vanilla `MemorySlot.hasExpired()` is `<= 0L`.
- dump ttl wording (mod.rs:136-141): OUT OF UNIT (comment and code both in mod.rs).
- VISIBLE_ADULT_PIGLIN_COUNT / VISIBLE_ADULT_HOGLIN_COUNT (types.rs:122-123): declared correctly.
  Vanilla `MemoryModuleType` registers them as ids 79/80 with names
  `visible_adult_piglin_count` / `visible_adult_hoglin_count` (MemoryModuleType.java:116-117;
  counted 79/80, total 115, matching `MEMORY_TYPE_COUNT` and Pumpkin's ids/names). Vanilla fills
  them only in `PiglinSpecificSensor` / `HoglinSpecificSensor`; those sensor files are outside
  this unit's allowed files (types.rs, value.rs), so left open.

No pure unit test added: neither file has a genuine gap to test (asserting `is_empty()` on an
empty `Vec` is formula-mirroring, forbidden by repo rules), and `mod.rs` already covers
empty-collection clearing. Compile-pitfall table re-read: no code was edited, so no new imports,
clones, initializers or clippy-pedantic concerns.


## Result (second pass, cross-file scope unlocked)

All five STILL OPEN rows re-verified against vendored vanilla under
`.vendor/vanilla-decompiled/net/minecraft/`. No source change was needed: every row is either an
exact vanilla match or already implemented elsewhere in the checkout.

- empty-collection set semantics (mod.rs:243,321 / value.rs:23-26,81-83,152-154,170-172): NOT A
  GAP. `Brain.setMemoryInternal(nullable)` nulls any empty `Collection` then clears the slot
  (Brain.java:239-262), the expiring overload does the same (Brain.java:214-237), and
  `isEmptyCollection` is at Brain.java:573-575. `MemoryStore::set_with_expiry` / `load_packed`
  call `is_empty_collection` and `slot.clear()` (mod.rs:255-258, 333-335). Exact match.
- setMemoryInternal unregistered put (mod.rs:240-242): NOT A GAP. `getMemorySlotIfPresent` returns
  null for an unregistered type (Brain.java:157-159) and both overloads guard with
  `if (slot != null)` (Brain.java:215-216, 240-241). `set_with_expiry` early-returns on a missing
  slot (mod.rs:252-254). Match.
- negative ttl (mod.rs:124-131): NOT A GAP. `MemorySlot.hasExpired()` is `timeToLive <= 0L`
  (MemorySlot.java:61-63) and `tick()` clears on `hasExpired()` (MemorySlot.java:16-24). Pumpkin's
  `self.ttl <= 0` matches, including negative values.
- dump ttl wording (mod.rs:136-141): NOT A GAP. `DebugBrainDump.accept(type, value, timeToLive)`
  passes `timestamp` as the displayed ttl (DebugBrainDump.java:158-160) while `MemorySlot.toString`
  prints the real ttl (MemorySlot.java:69-72). Pumpkin deliberately prints the real ttl and the
  comment documents that vanilla quirk. No change.
- VISIBLE_ADULT_PIGLIN_COUNT / VISIBLE_ADULT_HOGLIN_COUNT (types.rs:122-123): now fully populated
  AND consumed, matching vanilla. For hoglin brains, `HoglinSpecificSensor` writes both counts
  (hoglin_specific.rs:111-114) exactly as vanilla HoglinSpecificSensor.java:56-57 (adult piglins
  counted, first stored as nearest; adult-hoglin list size), using the same `!isBaby &&
  (Piglin || Hoglin)` filter and registering the sensor in hoglin.rs:279-280. Piglins use goal AI:
  `refresh_visible_adult_counts` (piglin.rs:532-580) stores both counts as entity fields and
  `hoglins_outnumber_piglins` (piglin_ai.rs:74-78) reads them, matching PiglinAi.java:779-783
  (brutes counted as piglins at piglin.rs:553-554, as in PiglinSpecificSensor.java:73-80).
  Hoglins read the brain counts in `piglins_outnumber_hoglins` (hoglin.rs:38-62), matching
  HoglinAi.java:179-181 (`piglinCount > hoglinCount + 1`). The existing test
  `hoglin_sensor_counts_only_visible_adults` covers the sensor; no duplicate unit test added.

Compile-pitfall table re-read: no source code edited, so no new imports, clones, struct
initializers, macro paths or clippy-pedantic concerns.
