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
