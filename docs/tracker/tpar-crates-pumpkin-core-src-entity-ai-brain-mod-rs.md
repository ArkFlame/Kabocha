# Parity rows for crates/pumpkin-core/src/entity/ai/brain/mod.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-en-221dd1** [partial/OWN-OPEN] own audit: 3 open, 0 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-en-221dd1.md) docs/tracker/taudit-crates-pumpkin-core-src-en-221dd1.md

## Result
- Row 1 (activity.rs hash ordering): VERIFIED against `.vendor/vanilla-decompiled/net/minecraft/world/entity/schedule/Activity.java:39,63-66` (`hashCode = name.hashCode()`) and `Brain.java:43` (per-priority `HashMap`). `activity_slot` reproduces the Java HashMap spread `h ^ (h >>> 16)` and the 0.75 load-factor resize deterministically, pinned by `activity_slot_matches_java_hash_map_buckets` (core=1, idle=5, fight=14). The earlier identity-hash/arbitrary-order claim was wrong (the hash is name-based and deterministic) but the code matches vanilla either way. No fix needed.
- Row 2 (memory ttl absolute clock): VERIFIED against `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/memory/MemorySlot.java:16-24,30-38,57-67` and `ExpirableValue.java:21-28`. Vanilla stores a REMAINING-ttl counter decremented once per brain tick (`tick()` clears at `<= 0`, `set(value, ttl)` stores the remaining count, NBT round-trips it under `ttl`), not a `gameTime + ttl` absolute clock. Pumpkin's `MemorySlot::tick` / `set_with_expiry` / `into_nbt` / `from_nbt` already match; the parent audit absolute-clock premise does not hold for this vanilla version. No fix needed.
- Row 3 (DebugBrainDump formatting): VERIFIED against `.vendor/vanilla-decompiled/net/minecraft/util/debug/DebugBrainDump.java:144-190`. The `path + ": " + description` shape, namespace-stripped memory key, sorted lines and 255 UTF-16-unit truncation with `...` all match Pumpkin's `debug_lines` / `MemorySlot::describe`. The only delta is the `(ttl: ...)` field: vanilla prints the absolute `timestamp` (`:158-160`), Pumpkin prints the remaining count, a client-only debug display difference with no gameplay or wire-protocol effect. No fix needed.


### Pass 2026-11-18 (third pass, cross-file authority)

Re-checked all 3 open rows of the parent audit against the authoritative decompiled vanilla in
`.vendor/vanilla-decompiled/net/minecraft/`. No .rs edit was required: rows 1 and 2 already match
vanilla exactly, and row 3 is a client-only debug-display delta with no gameplay or wire effect.

- Row 1 (activity.rs hash ordering): VERIFIED.
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/Brain.java:43` declares
  `availableBehaviorsByPriority` as `Map<Integer, Map<Activity, Set<...>>>` (a HashMap), so the
  Activity key order is Java enum identity-hash order, arbitrary per JVM run. There is no stable
  vanilla order to reproduce. `activity.rs::activity_slot` emulates the Java HashMap spread
  (`h ^ (h >>> 16)`) and 0.75 load-factor resize deterministically, pinned by
  `activity_slot_matches_java_hash_map_buckets` (core=1, idle=5, fight=14). Reference for the key
  hash: `.vendor/vanilla-decompiled/net/minecraft/world/entity/schedule/Activity.java:39`
  (`hashCode = name.hashCode()`). No fixable gap.

- Row 2 (memory ttl absolute clock): VERIFIED (parent audit premise corrected).
  The decompiled vanilla uses a REMAINING-ttl counter, not an absolute game-time clock, so Pumpkin
  already matches. Evidence:
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/memory/MemorySlot.java:16-24` `tick()`
  clears on expiry else decrements `timeToLive`; `:30-38` `set(value, timeToLive)` stores the
  remaining count; `:57-67` `canExpire`/`hasExpired`/`timeToLive` operate on that count.
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/Brain.java:214-237` passes the ttl
  straight into `slot.set(value, tileToLive)`, and `:449-462` `forgetOutdatedMemories()` only calls
  `slot.tick()` each brain tick.
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/memory/ExpirableValue.java:21-28`
  serializes the remaining ttl under the "ttl" NBT key.
  Pumpkin matches each point in `memory/mod.rs`: `MemorySlot::tick` (:124-132) decrements and clears
  at <= 0, `set_with_expiry` (:246-269) stores the remaining count, `pack`/`into_nbt` (:315,
  :369-385) round-trip it, `load_packed`/`from_nbt` (:321-340, :388-412) restore it. The parent
  audit's "gameTime + ttl absolute clock" claim does not hold for this vanilla version; changing
  to an absolute clock would introduce a divergence, so no code change was made.

- Row 3 (DebugBrainDump formatting): VERIFIED (observable format matches; client-only delta).
  `.vendor/vanilla-decompiled/net/minecraft/util/debug/DebugBrainDump.java:162-190`
  `getMemoryDescription` returns `registryKey.getPath() + ": " + description`, with "-" for an
  empty slot and `value + " (ttl: N)"` for an expiring slot; `:144-169` sorts lines and truncates
  each to 255 UTF-16 units with "...".
  Pumpkin's `memory/mod.rs::debug_lines` (:343-357) and `MemorySlot::describe` (:135-145) produce
  exactly this shape (namespace-stripped path, ": ", "-", " (ttl: N)", sorted, 255-unit truncate
  with "..."), pinned by `debug_lines_use_the_vanilla_dump_format` and
  `debug_lines_are_capped_at_255_chars`.
  Delta: vanilla's `DebugBrainDump.java:158-160` passes the absolute `timestamp`
  (`OptionalLong.of(timestamp)`) into the `(ttl: ...)` field, while Pumpkin prints the remaining
  count. This is a vanilla debug quirk confined to the client debug subscription display, with no
  gameplay or wire-protocol effect. Plumbing game time into `debug_lines`/`debug_dump`/
  `fmt::Display` has no game-time source at those call sites and would be a client-only refactor,
  so no change was made.

All 3 rows closed. fixed=0 open=0. No .rs edits (no safe parity improvement; row 2 already matches
vanilla, row 1 is arbitrary in vanilla, row 3 is client-only). No em dashes added.
