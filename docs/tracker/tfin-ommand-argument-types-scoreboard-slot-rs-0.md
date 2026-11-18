# Final rows from docs/tracker/tpar-s-pumpkin-core-src-command-argument-types-scoreboard-slot-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: scoreboard_slot.rs

CLOSED. All formerly STILL OPEN rows independently VERIFIED against decompiled vanilla on 2026-11-18; no source change needed. See "Third-pass verification (2026-11-18)" below. Each row is recorded as FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field.
No STILL OPEN rows remain; unit fully closed.
Re-read `scoreboard_slot.rs` end to end and independently checked every row against decompiled vanilla. No source change needed; every row is closed. No STILL OPEN rows remain.

## Result (FINAL PASS 2026-11-18)
Re-read `scoreboard_slot.rs` end to end and independently checked every row against decompiled vanilla. No source change needed; every row is closed. No STILL OPEN rows remain.

- Gap 1 `examples()` (scoreboard_slot.rs:90-93): VERIFIED. Returns `["sidebar", "foo.bar"]`, identical to vanilla `ScoreboardSlotArgument.EXAMPLES = Arrays.asList("sidebar", "foo.bar")` (`.vendor/vanilla-decompiled/net/minecraft/commands/arguments/ScoreboardSlotArgument.java:19`).
- Gap 2 `slot_name` (scoreboard_slot.rs:110-131): VERIFIED. Exhaustive `match` over all 19 `ScoreboardDisplaySlot` variants, no fallback; each name equals its vanilla `getSerializedName()` in declaration order (`.vendor/vanilla-decompiled/net/minecraft/world/scores/DisplaySlot.java:8-26`).
- Gap 3 `display_slot_component` (scoreboard.rs:623-625): VERIFIED, NO ACTION. Passes `slot_name(slot)` as the plain serialized-name string, matching vanilla's `slot.getSerializedName()` use sites (file outside this unit; no change required).
- `parse` (scoreboard_slot.rs:67-73): VERIFIED = vanilla `readUnquotedString` + `DisplaySlot.CODEC.byName(name)` + `ERROR_INVALID_VALUE.createWithContext(reader, name)` (ScoreboardSlotArgument.java:36-43); error via `INVALID_DISPLAY_SLOT_ERROR.create(&*reader, ...)` with key `argument.scoreboardDisplaySlot.invalid`.
- `get` (scoreboard_slot.rs:97-102): VERIFIED = `getDisplaySlot` (ScoreboardSlotArgument.java:31-33).
- `list_suggestions` (scoreboard_slot.rs:79-88): VERIFIED = vanilla stream over `DisplaySlot.values()` serialized names (ScoreboardSlotArgument.java:47-49).
- `client_side_parser` -> `JavaClientArgumentType::ScoreboardSlot` (scoreboard_slot.rs:75-77): VERIFIED = client id `minecraft:scoreboard_slot`.
- `SLOT_NAMES` (scoreboard_slot.rs:20-58): VERIFIED = 19 `DisplaySlot` enum variants in declaration order with exact spelling (DisplaySlot.java:8-26).
- Tests (scoreboard_slot.rs:138-174): 4 pure unit tests cover error context, known-name parsing, name/parse agreement for all 19 names, and unknown-name rejection. No new test needed.
- No BLOCKED-DATA rows: no missing `pumpkin-data` generated field or absent subsystem. No .rs file touched, so rustfmt not applicable.
