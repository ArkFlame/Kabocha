# audit: crates/pumpkin-core/src/command/argument_types/scoreboard_slot.rs

Source: `crates/pumpkin-core/src/command/argument_types/scoreboard_slot.rs`.
Vanilla reference: `net/minecraft/world/scores/ScoreboardSlot` / `ScoreboardDisplaySlot`
(`byName`, `getName`) and `net/minecraft/commands/arguments/ScoreboardSlotArgumentType`
(`parse`, `EXAMPLES`, `getScoreboardSlot`). Paper/Spigot add no overrides.

No TODO/FIXME markers exist in the file; the tracker's "vanilla markers" were the four
doc comments that cite vanilla (lines 19, 42, 92, 111 pre-change).

## Fixed in this review

- `scoreboard_slot.rs:54` parse error used `create_without_context`, so an unknown slot name
  produced `argument.scoreboardDisplaySlot.invalid` with no cursor/input. Vanilla throws
  `ERROR_INVALID_VALUE.createWithContext(reader, string)`. Now uses
  `INVALID_DISPLAY_SLOT_ERROR.create(&*reader, ...)`, matching the other parse-site argument
  types (`entity_selector/option.rs:188`, `entity.rs:137-146`). Regression test
  `invalid_slot_error_carries_reader_context` asserts cursor 5 / input "bogus".

## Re-checked as already matching vanilla (no change needed)

- `SLOT_NAMES` covers all 19 `ScoreboardDisplaySlot` variants in vanilla order and spelling
  (`list`, `sidebar`, `below_name`, then `sidebar.team.<16 colors>` in `ChatFormatting` order);
  `byName` is exact-match, no aliases, no case folding, so the linear `find_map` is equivalent.
- Quoted input (`"sidebar"`) is rejected because both `readUnquotedString` and vanilla
  `readUnquotedString` stop at the quote; empty input yields the invalid-slot error.
- `client_side_parser` maps to `minecraft:scoreboard_slot` (`commands.rs:324`), which is the
  vanilla `ArgumentTypes` id the client uses for its own completions.
- `get` and `text()` argument plumbing match `getScoreboardSlot`.

## Remaining gaps

1. `scoreboard_slot.rs:74-80` `examples()` lists `sidebar`/`list`/`below_name`. Vanilla
   `ScoreboardSlotArgumentType.EXAMPLES` is `List.of("sidebar", "foo.bar")` (`foo.bar` is a
   long-standing Mojang oddity). Only affects help text, and the vanilla value cannot be
   confirmed against decompiled sources in this worktree. Size S, low value.
2. `scoreboard_slot.rs:94-99` `slot_name` falls back to `"sidebar"` for an unmatched slot.
   The table is exhaustive today so the fallback is dead code and silently wrong if a variant
   is ever added. Suggest an exhaustive `match` (or `Option`) instead of `unwrap_or`. Size S.
3. Adjacent, out of this file: `crates/pumpkin-core/src/command/commands/scoreboard.rs:617`
   builds the `commands.scoreboard.objectives.display.set|cleared` slot argument as
   `TextComponent::text(slot_name(slot))`; vanilla passes `ScoreboardSlot.getName()`
   (also a plain string), so this matches once `slot_name` is exact. No action.
## Re-audit 2026-11-17 (single pass, no changes)
- Gap 2 above is closed: `slot_name` (scoreboard_slot.rs:97-118) is now an exhaustive `match` with
  no fallback, so no dead default remains.
- Re-read the file end to end: parse/error/suggestions/examples plumbing and the 4 tests match
  vanilla `ScoreboardSlotArgumentType.parse` + `DisplaySlot.byName`; no TODO/FIXME; no new gap.
- Gap 1 (`examples()`) stays open as cosmetic S: vanilla `EXAMPLES` is `sidebar`, `foo.bar`; examples
  never reach the wire, and `foo.bar` cannot be confirmed from decompiled sources in this worktree.
