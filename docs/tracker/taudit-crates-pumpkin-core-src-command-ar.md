# Tracker: audit / command argument_types / scoreboard_slot.rs

Reviewed: `crates/pumpkin-core/src/command/argument_types/scoreboard_slot.rs` on branch
`kab/taudit-crates-pumpkin-core-src-command-ar`.
Vanilla reference: `net.minecraft.commands.arguments.ScoreboardSlotArgumentType` (1.21.x),
`net.minecraft.world.scores.DisplaySlot` (`byName` / `getSerializedName`),
`SharedSuggestionProvider.suggest` / `matchesSubStr`. No `.vendor/` copy exists in this
worktree, so the comparison is against the 1.21.x sources from knowledge.

The four "vanilla" markers the tracker counted in this file are doc comments, not TODO/FIXME.

## Fixed in this review

| Gap | Change |
|---|---|
| `slot_name` (-sidebar +) was a linear scan over `SLOT_NAMES` with `unwrap_or("sidebar")`. Vanilla `DisplaySlot` owns its serialized name, so no variant can lack one; the fallback could only ever print the wrong slot (e.g. `objectives setdisplay` feedback after a codegen-added variant) and hid the omission. | Replaced with an exhaustive `match` (`scoreboard_slot.rs:92-120`), so a new enum variant fails to compile instead of silently displaying `sidebar`. |
| No test pinned the two directions (`parse` name table vs `slot_name`) together. | `slot_name_and_parse_agree_for_every_known_name`, `rejects_unknown_slot_name`. |

Parsing itself already matches vanilla: unquoted string, exact (case-sensitive) lookup over
the 19 `DisplaySlot` names, `argument.scoreboardDisplaySlot.invalid` with the raw input as
`%s`, suggestions over the same name set. No change needed.

## Remaining gaps

### 1. Suggestion filter splits on more separators than vanilla - size S (shared helper)

`scoreboard_slot.rs:68-71` -> `pumpkin-command/src/suggestion/suggestions.rs:218-227`
(`matches_substr`). Vanilla `SharedSuggestionProvider.matchesSubStr` only restarts the
prefix scan after `'.'`; pumpkin also restarts after `'_'`, `'/'` and `':'`. For this
argument that means typing `name` suggests `below_name`, where vanilla suggests nothing
(no `'.'` in `below_name`). Not fixable in this file alone without a prefix/vanilla-only
helper; behaviour is otherwise identical for the 16 `sidebar.team.*` names.

### 2. Parse error carries no reader context - size S

`scoreboard_slot.rs:54-56` calls `create_without_context`; vanilla uses
`ERROR_INVALID_VALUE.createWithContext(reader, string)`. `StringReader` implements
`ContextProvider` (`pumpkin-command/src/string_reader.rs:359`), so
`create_args_slice(reader, ..)` would attach the cursor. No observable wire difference
today (the cursor is never sent to the client), so left alone; the same applies to most
argument types in this directory.

### 3. Serialized names live in core, not in generated data - size S

`scoreboard_slot.rs:20-40`. `pumpkin-data`'s `ScoreboardDisplaySlot` is generated from
`assets/scoreboard_display_slot.json` (`tools/pumpkin-codegen/src/scoreboard_slot.rs`) but
carries no serialized names, so `SLOT_NAMES` is a hand-maintained duplicate. A Minecraft
bump that adds a display slot would make `parse` reject a valid vanilla name (and now fail
to compile in `slot_name`). Fix: emit the names in the codegen output and delete
`SLOT_NAMES`; needs an asset field the Extractor does not currently emit.

### 4. `examples()` differs from vanilla - size S, cosmetic

`scoreboard_slot.rs:74-80` returns `sidebar`, `list`, `below_name`; vanilla's
`ScoreboardSlotArgumentType.EXAMPLES` is `["sidebar", "foo.bar"]`. Examples are not sent
on the wire (`JavaClientArgumentType::ScoreboardSlot` carries only the type id) and nothing
local reads them, so no behaviour change follows.
