# Parity rows for crates/pumpkin-core/src/command/argument_types/scoreboard_slot.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-co-589cf4** [partial/OWN-OPEN] own audit: 5 open, 1 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-co-589cf4.md) docs/tracker/taudit-crates-pumpkin-core-src-co-589cf4.md

## Result
CLOSED. All formerly STILL OPEN rows independently VERIFIED against decompiled vanilla on 2026-11-18; no source change needed. See "Third-pass verification (2026-11-18)" below. Each row is recorded as FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field.


## Third-pass verification (2026-11-18)
Independent re-read of `crates/pumpkin-core/src/command/argument_types/scoreboard_slot.rs` against `.vendor/vanilla-decompiled/net/minecraft/commands/arguments/ScoreboardSlotArgument.java` and `.vendor/vanilla-decompiled/net/minecraft/world/scores/DisplaySlot.java`. No source change needed; no `.rs` file touched (rustfmt not applicable).

- `examples()` (scoreboard_slot.rs:90-93): VERIFIED. Returns `["sidebar", "foo.bar"]`, identical to vanilla `ScoreboardSlotArgument.EXAMPLES = Arrays.asList("sidebar", "foo.bar")` (ScoreboardSlotArgument.java:19).
- `slot_name` (scoreboard_slot.rs:110-131): VERIFIED. Exhaustive `match` over all 19 `ScoreboardDisplaySlot` variants, no fallback; every serialized name equals its vanilla `DisplaySlot` name (DisplaySlot.java:8-26). `SLOT_NAMES` (20-58) carries the same 19 names in enum declaration order.
- `parse` (scoreboard_slot.rs:67-73): VERIFIED. `read_unquoted_string()` + exact name lookup + `INVALID_DISPLAY_SLOT_ERROR.create(&*reader, TextComponent::text(name))` (translation key `argument.scoreboardDisplaySlot.invalid`, generated/translation.rs:1048-1049) matches vanilla `readUnquotedString` + `DisplaySlot.CODEC.byName(name)` + `ERROR_INVALID_VALUE.createWithContext(reader, name)` (ScoreboardSlotArgument.java:20-22, 36-43).
- `get` (scoreboard_slot.rs:97-102): VERIFIED = vanilla `getDisplaySlot` (ScoreboardSlotArgument.java:31-33).
- `list_suggestions` (scoreboard_slot.rs:79-88): VERIFIED. `filter_and_suggest_one` mirrors vanilla `SharedSuggestionProvider.matchesSubStr` (pumpkin-command suggestion/suggestions.rs:160-162); suggests all 19 serialized display-slot names, matching ScoreboardSlotArgument.java:47-49.
- `client_side_parser` -> `JavaClientArgumentType::ScoreboardSlot` (scoreboard_slot.rs:75-77): VERIFIED, vanilla client id `minecraft:scoreboard_slot` (carried from prior passes).
- Adjacent scoreboard.rs use sites (gap 3): VERIFIED, NO ACTION. `display_slot_component` (scoreboard.rs:623-624) passes `slot_name(slot)` as a plain serialized-name `TextComponent`, matching vanilla's `slot.getSerializedName()` string argument; use sites 696, 716. File is outside this unit; no change required.
- Tests (scoreboard_slot.rs:138-174): 4 pure unit tests cover error context, known-name parsing, name/parse agreement for all 19 names, and unknown-name rejection. No new test needed.

No STILL OPEN rows remain; unit fully closed.
