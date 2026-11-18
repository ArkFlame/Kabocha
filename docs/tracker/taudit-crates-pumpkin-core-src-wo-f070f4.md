# world/scoreboard.rs — deep audit

Sources: crates/pumpkin-core/src/world/scoreboard.rs, crates/pumpkin-core/src/world/scoreboard/tests.rs.
Compared against vanilla `Scoreboard`, `ServerScoreboard`, `ScoreboardSaveData`, `PlayerTeam`/`ClientboundSetPlayerTeamPacket`, `Objective`/`ClientboundSetObjectivePacket`, `ClientboundSetScorePacket`, and Bedrock `SetDisplayObjective`/`SetScore` semantics; Paper adds no scoreboard behaviour here.

## Fixed in this audit
- Duplicate `mod tests`: the file declared both `mod tests;` (scoreboard/tests.rs) and an inline `mod tests { ... }`, which is E0428 for any `cfg(test)` build; the three inline Bedrock tests were moved into scoreboard/tests.rs.
- `ScoreboardBuilder::score` put the entity name in the outer `scores` map and the objective name in the inner map, the inverse of every reader (`get_score` and the packet senders index by objective first), so builder-built boards never returned their scores; keys swapped and pinned by `builder_scores_are_keyed_by_objective`.
- `BedrockScoreboard::add_score` used `+` on `i32` (debug-build overflow panic); now `wrapping_add`, matching the Java `int` overflow of `ScoreboardScore.add` and the JE path.

Both audit markers in the file are comments, not TODOs: line 731 (vanilla display-slot names) and line 774 (teams not persisted).

## Remaining gaps
| # | Location | Vanilla | Gap | Size |
|---|---|---|---|---|
| 1 | crates/pumpkin-core/src/world/scoreboard.rs:773-831, 835-894 | `ScoreboardSaveData` saves and loads `data.Teams` (Name, DisplayName, Prefix, Suffix, FriendlyFire, SeeFriendlyInvisibles, NameTagVisibility, CollisionRule, Color, Players) | Teams are never written to or read from `scoreboard.dat`, so every team, prefix/suffix and colour is lost on restart while objectives/scores/display slots survive | M |
| 2 | crates/pumpkin-core/src/world/scoreboard.rs:250-266, 297-305 | Bedrock only shows an objective through `SetDisplayObjective`; adding/updating an objective must not touch an existing display | Every JE `add_objective`/`update_objective` also broadcasts a Bedrock `SetDisplayObjective(sidebar)` for that objective without recording a display slot, so a Bedrock client shows an objective the server considers undisplayed (and an objective update clobbers another sidebar without the remove+redisplay+row-restore sequence `BedrockObjectivePacket` uses) | M |
| 3 | crates/pumpkin-core/src/world/scoreboard.rs:316-320, 668-673 | Bedrock has only list/sidebar/belowname | Every `sidebar.team.*` (and any future) JE display slot is sent to Bedrock as `sidebar`, so `/scoreboard objectives setdisplay sidebar.team.red x` overwrites the Bedrock sidebar even though the server keeps both slots | S |
| 4 | crates/pumpkin-core/src/world/scoreboard.rs:776-831 | `ScoreboardSaveData` persists per-objective and per-score number formats, and the objective criterion is required to survive | `criterion` is persisted but `number_format` is dropped on save and load, so a blanked/fixed number format reverts to the default after restart | S |
| 5 | crates/pumpkin-core/src/world/scoreboard.rs:569-595 | `Scoreboard.addPlayerToTeam` only accepts a holder that is not already on a team and the old team gets a `RemovePlayers` packet | The code only early-returns when the holder is already in *this* team, so a holder passed through the plugin API can end up in two teams and clients keep the stale membership (needs a vanilla check before changing) | S |
| 6 | crates/pumpkin-core/src/world/scoreboard.rs:455-473 | `ServerScoreboard.removePlayerScore` resets only scores that existed | A reset packet is broadcast even when the score does not exist; harmless on JE clients, but it announces removals that never happened | S |
| 7 | crates/pumpkin-core/src/world/scoreboard.rs:167-169 | Objective names are immutable map keys | `get_objective_mut` lets callers change `name`, desynchronising the map key from the objective, after which `update_objective` inserts a duplicate under the new name and removal targets the wrong key; needs an invariant check or a keyed update API | S |

Checked, no change: display-slot persistence names match vanilla (`list`, `sidebar`, `below_name`, `sidebar.team.*`); `Locked` defaults to true like `dummy` (read-only criteria); `remove_objective` drops its scores and display references; team packet fields (`options` bit flags, visibility/collision names, colour id) and `TeamMethod` transitions match `ClientboundSetPlayerTeamPacket`; objective add/update/remove modes and render types match; `NoTarget`/`Player`/`World` targets gate on `scoreboard_synced` so pre-spawn changes are replayed by `send_to_player`.

## Status (code-verified sweep)
CLOSED: both verification items (BedrockScoreboard wrapping_add and score map orientation) were re-read against the current world/scoreboard.rs and decompiled vanilla and are VERIFIED FIXED. See the Result sections below.


## Result

Re-read both STILL OPEN verification items against the code and decompiled vanilla (`.vendor/vanilla-decompiled/net/minecraft/`, target V_26_3).

- STILL OPEN "BedrockScoreboard wrapping_add": VERIFIED FIXED, no change. `BedrockScoreboard::add_score` (`world/scoreboard.rs:1298`) uses `current.wrapping_add(delta)` and the JE path `Scoreboard::add_score` (`:450`) uses `current_val.wrapping_add(delta)`. Vanilla `Score.value(int)` (`world/scores/Score.java:38-40`) is a plain Java `int` assignment, so wrapping on overflow is the correct emulation; a Rust debug `+` would panic. No pure unit-test seam exists: `add_score` needs a live `Player` with a Bedrock client.
- STILL OPEN "score map orientation": VERIFIED FIXED, no change. `ScoreboardBuilder::score` (`world/scoreboard.rs:1436-1440`) inserts with `.entry(obj_s).or_default().insert(entity_s, score)`, i.e. objective as the outer key, matching every reader (`get_score` `:187-189` indexes `self.scores.get(objective_name)?.get(entity_name)`). Decompiled vanilla (`world/scores/Scoreboard.java:39,231`) keys `scoresByName` by player first then objective, the inverse of Pumpkin's layout, but Pumpkin is consistently objective-first across readers and packet senders, so the builder must follow that. `world/scoreboard/tests.rs:352` (`builder_scores_are_keyed_by_objective`) already pins the orientation.

No source file changed: both items were already applied by a prior pass and this pass only re-verified them. No rustfmt run (no `.rs` file touched). No test added (orientation already pinned; no pure seam for the Bedrock overflow path).

## Result (re-verification 2026-11-18)

Re-checked both rows marked STILL OPEN against the current code and decompiled vanilla; both were already fixed by a prior pass, so no source edit was needed this pass.

- STILL OPEN "BedrockScoreboard wrapping_add": VERIFIED FIXED, no change. `BedrockScoreboard::add_score` (`world/scoreboard.rs:1354`) uses `current.wrapping_add(delta)` and the JE `Scoreboard::add_score` (`:450`) uses `current_val.wrapping_add(delta)`. Vanilla `Score.value(int)` (`.vendor/vanilla-decompiled/net/minecraft/world/scores/Score.java:38-40`) is a plain Java `int` assignment (`this.value = score`), so wrapping on overflow is the correct emulation; a Rust debug `+` would panic. No pure unit-test seam exists: `add_score` needs a live `Player` with a Bedrock client.
- STILL OPEN "score map orientation": VERIFIED FIXED, no change. `ScoreboardBuilder::score` (`world/scoreboard.rs:1492-1496`) inserts `.entry(obj_s).or_default().insert(entity_s, score)`, i.e. objective as the outer key, matching every reader (`get_score` `:187-189` indexes `self.scores.get(objective_name)?.get(entity_name)`). `world/scoreboard/tests.rs:352` (`builder_scores_are_keyed_by_objective`) already pins the orientation.

Also re-confirmed the single `mod tests;` declaration at `:30` with no inline duplicate. No source file changed, so no rustfmt run and no new test added.

## Result (deep-audit pass 2026-11-18)

Worked the "Remaining gaps" table smallest-first against decompiled vanilla in
`.vendor/vanilla-decompiled/net/minecraft/`. Only `crates/pumpkin-core/src/world/scoreboard.rs` was
edited; no generated data, WIT or cross-crate files touched.

- Row 3 (S) Bedrock team display slots: FIXED. Added `bedrock_display_slot`, which maps only
  List/Sidebar/BelowName to Bedrock slots. `set_display_objective` and `send_to_player` now send a
  JE-only packet for `sidebar.team.*` instead of a Bedrock `SetDisplayObjective(sidebar)`, so a
  team display no longer overwrites the real Bedrock sidebar. Pinned by the pure unit test
  `bedrock_slot_tests::team_display_slots_have_no_bedrock_slot` added in the same file.
- Row 4 (S) number formats dropped on save/load: FIXED. Vanilla `Objective.Packed.CODEC`
  (`Objective.java:148-159`) and `Score.Packed.MAP_CODEC` (`Score.java:68-77`) both use
  `NumberFormatTypes.CODEC.optionalFieldOf("format")`, a tagged compound (`type` = blank/styled/
  fixed, `value` component for fixed). Added `number_format_to_nbt`/`number_format_from_nbt` and
  persisted `format` for both objectives and scores. `styled` is skipped because Pumpkin's `Style`
  has no NBT codec (same reason its network writer has a TODO).
- Row 1 (M) teams not persisted: FIXED. Vanilla `ScoreboardSaveData.Packed.CODEC`
  (`ScoreboardSaveData.java:41-55`) writes a `Teams` list using `PlayerTeam.Packed.CODEC`
  (`PlayerTeam.java:271-302`). `to_saved_nbt` now emits `Teams` and `load_saved_nbt` rebuilds every
  team with Name, DisplayName, TeamColor, AllowFriendlyFire, SeeFriendlyInvisibles,
  MemberNamePrefix/Suffix, NameTagVisibility, DeathMessageVisibility, CollisionRule and Players.
- Row 6 (S) reset packet for a score that does not exist: already fixed before this pass.
  `remove_score` returns early when the score is absent, matching
  `ServerScoreboard.removePlayerScore`.
- Row 2 (M) Bedrock add/update objective broadcasts a sidebar display: LEFT OPEN. Fixing it means
  changing the shared JE add/update path and reconciling it with the `BedrockObjectivePacket`
  remove/redisplay/restore-row sequence in `BedrockScoreboard`, a cross-cutting Bedrock protocol
  change beyond a single smallest fix.
- Row 5 (S) duplicate team membership via the plugin API: LEFT OPEN. Decompiled
  `Scoreboard.addPlayerToTeam` (`:385-404`) removes the holder from the previous team, but the
  bundled `ServerScoreboard.addPlayerToTeam` (`:121-131`) does not broadcast a `RemovePlayers`
  packet for that old team, so the exact client-facing parity behaviour is ambiguous and needs a
  dedicated vanilla check.
- Row 7 (S) `get_objective_mut` desyncs the map key: LEFT OPEN. It is currently unused, but changing
  or removing this public API is a cross-crate API change outside this unit.
- Prior STILL OPEN verification items remain VERIFIED FIXED (see the two earlier Result sections).

No cargo/rustfmt run: worker tool restriction, so the batch compile is the verification gate.

## Result (gap sweep 2026-11-18)

Re-read every row of the Remaining gaps table against the current `world/scoreboard.rs` and decompiled vanilla (`.vendor/vanilla-decompiled/net/minecraft/`, target V_26_3). Smallest rows first.

- Row 1 (M, Teams persistence): ALREADY FIXED, no change. `to_saved_nbt` writes the `Teams` list (`scoreboard.rs:950-994,1007`) and `load_saved_nbt` reads it back (`:1075-1123`), covering Name, DisplayName, Prefix/Suffix, friendly-fire/invisibles flags, name-tag/collision strings, colour and Players.
- Row 3 (S, Bedrock `sidebar.team.*` slot): ALREADY FIXED, no change. `set_display_objective` maps only List/Sidebar/BelowName to Bedrock and sends Java-only for team slots (`:326-356`); `bedrock_display_slot` returns `None` for all `Team*` slots (`:835-842`). `bedrock_slot_tests` pins it.
- Row 4 (S, number_format persistence): ALREADY FIXED, no change. Criterion plus objective/score `format` are written (`:903-920,941-945`) and read (`:1033-1034,1056`) via `number_format_to_nbt`/`from_nbt`. `Styled` is intentionally omitted (no Style NBT codec).
- Row 6 (S, removePlayerScore reset): ALREADY FIXED, no change. `remove_score` returns early when the score does not exist (`:512-535`), matching vanilla `ServerScoreboard.removePlayerScore`.
- Row 5 (S, one team per holder): FIXED. Verified vanilla `Scoreboard.addPlayerToTeam` (`.vendor/.../world/scores/Scoreboard.java:385-404`) removes the holder from any previous team's player list before adding, and `ServerScoreboard.addPlayerToTeam` (`.vendor/.../server/ServerScoreboard.java:122-131`) broadcasts only the ADD packet (the Java client removes the old membership itself). `add_player_to_team` now retains-filter the holder out of every team before adding to the target; no extra RemovePlayers packet is sent, matching vanilla. Added pure unit test `team_membership_tests::joining_a_team_removes_the_holder_from_its_previous_team` (uses `NoTarget`, no live player). Note: the row's "old team gets a RemovePlayers packet" claim is not present in this decompiled vanilla, so no such packet was added.
- Row 7 (S, objective name immutability): FIXED. `get_objective_mut` was unused across the whole repo and let callers desynchronise the `objectives` map key; removed it. Immutable `get_objective`/`get_objectives` remain.
- Row 2 (M, Bedrock add/update objective clobbers display): LEFT OPEN, reason: the JE `Scoreboard` has no Bedrock state, so replacing its naive `BSetDisplayObjective(sidebar)` with the remove+redisplay+row-restore sequence would require tracking Bedrock display state across the JE/Bedrock scoreboard split (out of this file's scope). Bedrock lifecycle is already handled correctly by `BedrockScoreboard::objective_packets` (`:1398-1444`); changing the JE path without that state risks leaving Bedrock clients with stale objectives.

Files changed: `crates/pumpkin-core/src/world/scoreboard.rs` only. No generated data, WIT, or cross-crate edits. No unwrap/expect/panic, no stdout, no em dashes. rustfmt and tests not run by this worker (orchestrator compiles once).


## Result (second pass 2026-11-18)

Closed Row 2 (M, Bedrock add/update objective clobbers display), the only gap still LEFT OPEN. Re-verified all other rows, which remain fixed.

- Row 2 (M): FIXED. Scoreboard.add_objective and update_objective broadcast a Bedrock SetDisplayObjective(sidebar) on every add/update, which reveals an undisplayed objective and clobbers the real Bedrock sidebar. Vanilla ServerScoreboard.onObjectiveAdded / onObjectiveChanged (.vendor vanilla-decompiled net minecraft server ServerScoreboard.java lines 172-187) broadcast only ClientboundSetObjectivePacket, never a display packet, and send_to_player in this same file already documents and implements that rule. Changed both methods to send_je with the Java update and removed the Bedrock packets; Bedrock displays are now written only by the explicit set_display_objective and the send_to_player slot loop. Pinned by the pure unit test add_and_update_objective_do_not_write_a_bedrock_display (counts editioned vs Java-only sends via a DisplaySendTarget), added to scoreboard/tests.rs.
- Rows 1,3,4,5,6,7 and the earlier STILL OPEN wrapping and orientation items: re-read, all still fixed, no change this pass.

Files changed: world/scoreboard.rs and world/scoreboard/tests.rs. No generated data, WIT or cross-crate edits. No unwrap/expect/panic, no stdout, no em dashes. No cargo or rustfmt run (worker restriction; orchestrator compiles once). Known limitation: a displayed objective Bedrock title is not refreshed on update, matching the existing send_to_player Java-only objective handling; the reported clobbering bug is removed.


## Result (third pass 2026-11-18)

Re-read every row in the Remaining gaps table plus the two STILL OPEN verification items against the current `world/scoreboard.rs` and decompiled vanilla under `.vendor/vanilla-decompiled/net/minecraft/`. All rows are CLOSED; no source edit was needed this pass.

- Row 1 (M, Teams persistence): VERIFIED FIXED. `to_saved_nbt` writes the `Teams` list (`:986,999`) and `load_saved_nbt`/`load_teams` read it back (`:1068-1120`), covering Name, DisplayName, TeamColor, AllowFriendlyFire, SeeFriendlyInvisibles, MemberNamePrefix/Suffix, NameTagVisibility, DeathMessageVisibility, CollisionRule and Players. Field names match vanilla `ScoreboardSaveData.Packed.CODEC` (`.vendor/.../world/scores/ScoreboardSaveData.java:41-55`) and `PlayerTeam.Packed.CODEC` (`.vendor/.../world/scores/PlayerTeam.java:271-302`).
- Row 2 (M, Bedrock add/update objective clobbers display): VERIFIED FIXED. `add_objective`/`update_objective` now call `target.send_je` only (`:268,302`), never a Bedrock display packet. Vanilla `Scoreboard.onObjectiveAdded`/`onObjectiveChanged` are no-ops and `ServerScoreboard.onObjectiveChanged` broadcasts only `ClientboundSetObjectivePacket` (`.vendor/.../world/scores/Scoreboard.java:462-466`, `.vendor/.../server/ServerScoreboard.java:172-187`). Pinned by `add_and_update_objective_do_not_write_a_bedrock_display` in scoreboard/tests.rs.
- Row 3 (S, Bedrock `sidebar.team.*`): VERIFIED FIXED. `bedrock_display_slot` returns `Some` only for List/Sidebar/BelowName and `None` for all others (`:831-838`); `set_display_objective` sends Java-only for team slots (`:332-336`). Pinned by `bedrock_slot_tests::team_display_slots_have_no_bedrock_slot`.
- Row 4 (S, number formats): VERIFIED FIXED. Objective and score `format` are written/read via `number_format_to_nbt`/`number_format_from_nbt` (`:842-869,1027,1049`). Codec shape matches vanilla `NumberFormatTypes.CODEC` dispatch (`.vendor/.../network/chat/numbers/NumberFormatTypes.java:14-25`) and `FixedFormat` `value` field (`.vendor/.../network/chat/numbers/FixedFormat.java:12`); `Objective.Packed.CODEC` and `Score.Packed.MAP_CODEC` both use `optionalFieldOf("format")` (`.vendor/.../world/scores/Objective.java:157`, `Score.java:74`). `styled` omitted (no Style NBT codec).
- Row 5 (S, one team per holder): VERIFIED FIXED. `add_player_to_team` retains the holder out of every non-target team and no-ops only when already in the target (`:632-645`), matching vanilla `Scoreboard.addPlayerToTeam` (`.vendor/.../world/scores/Scoreboard.java:385-404`); `ServerScoreboard.addPlayerToTeam` broadcasts only the ADD packet (`.vendor/.../server/ServerScoreboard.java:122-131`). Pinned by `team_membership_tests`.
- Row 6 (S, reset packet for absent score): VERIFIED FIXED. `remove_score` returns early when the score is absent (`:505-512`). Vanilla `ServerScoreboard.onPlayerScoreRemoved` only fires after the score is actually removed (`.vendor/.../server/ServerScoreboard.java:88-95`).
- Row 7 (S, `get_objective_mut` desync): VERIFIED FIXED. No `get_objective_mut` exists anywhere; only immutable `get_objective`/`get_objectives` remain.
- STILL OPEN "BedrockScoreboard wrapping_add": VERIFIED FIXED. Both `BedrockScoreboard::add_score` (`:1575`) and JE `Scoreboard::add_score` (`:494`) use `wrapping_add`; vanilla `Score.value(int)` is a plain Java int assignment (`.vendor/.../world/scores/Score.java:38-40`).
- STILL OPEN "score map orientation": VERIFIED FIXED. `ScoreboardBuilder::score` inserts `.entry(obj_s).or_default().insert(entity_s, score)` (`:1715`), matching all readers; pinned by `builder_scores_are_keyed_by_objective` (scoreboard/tests.rs:352).

No source file changed, so no rustfmt run and no new test added. No unwrap/expect/panic, no stdout, no em dashes. No cargo run (worker restriction; orchestrator compiles once). Known limitation unchanged: a displayed Bedrock objective title is not refreshed on update.


## Result (final verification pass 2026-11-18)

Re-read every row in the Remaining gaps table and the two former STILL OPEN verification items against the current `crates/pumpkin-core/src/world/scoreboard.rs` and decompiled vanilla under `.vendor/vanilla-decompiled/net/minecraft/`. All rows are CLOSED; no source edit was needed this pass.

- BedrockScoreboard/JE wrapping_add: VERIFIED FIXED. Both `Scoreboard::add_score` (`:494`) and `BedrockScoreboard::add_score` (`:1575`) use `wrapping_add`. Vanilla `Score.value(int)` is a plain Java `int` assignment (`.vendor/vanilla-decompiled/net/minecraft/world/scores/Score.java:38-40`), so wrapping is the correct overflow emulation; a Rust debug `+` would panic.
- score map orientation: VERIFIED FIXED. `ScoreboardBuilder::score` inserts `.entry(obj_s).or_default().insert(entity_s, score)` (`:1715`), objective as outer key, matching every reader; pinned by `builder_scores_are_keyed_by_objective` in scoreboard/tests.rs.
- Row 1 (Teams persistence): VERIFIED FIXED. `to_saved_nbt` writes `Teams` (`:999`) and `load_saved_nbt` reads it (`:1068`). Field names match vanilla `ScoreboardSaveData.Packed.CODEC` (`.vendor/.../world/scores/ScoreboardSaveData.java:41-55`) and `PlayerTeam.Packed.CODEC` (`.vendor/.../world/scores/PlayerTeam.java:271-302`).
- Row 2 (Bedrock add/update objective display): VERIFIED FIXED. `add_objective`/`update_objective` call `target.send_je` only (`:268,302`); vanilla `ServerScoreboard.onObjectiveChanged` broadcasts only `ClientboundSetObjectivePacket` (`.vendor/.../server/ServerScoreboard.java:172-187`). Pinned by `add_and_update_objective_do_not_write_a_bedrock_display`.
- Row 3 (Bedrock `sidebar.team.*` slot): VERIFIED FIXED. `bedrock_display_slot` (`:831`) returns `Some` only for List/Sidebar/BelowName; Java-only for team slots (`:335`). Pinned by `team_display_slots_have_no_bedrock_slot`.
- Row 4 (number formats): VERIFIED FIXED. `number_format_to_nbt`/`number_format_from_nbt` (`:842,858`) persist objective and score `format`. Codec shape matches vanilla `NumberFormatTypes.CODEC` (`.vendor/.../network/chat/numbers/NumberFormatTypes.java:14-25`) and `Score.Packed.MAP_CODEC` `optionalFieldOf("format")` (`.vendor/.../world/scores/Score.java:74`). `styled` omitted (no Style NBT codec).
- Row 5 (one team per holder): VERIFIED FIXED. `add_player_to_team` (`:618`) retains the holder out of every non-target team, matching vanilla `Scoreboard.addPlayerToTeam` (`.vendor/.../world/scores/Scoreboard.java:385-404`). Pinned by `team_membership_tests`.
- Row 6 (reset packet for absent score): VERIFIED FIXED. `remove_score` (`:499`) returns early when the score is absent; vanilla `ServerScoreboard.onPlayerScoreRemoved` fires only after removal (`.vendor/.../server/ServerScoreboard.java:88-95`).
- Row 7 (`get_objective_mut` desync): VERIFIED FIXED. No `get_objective_mut` exists anywhere in the crate; only immutable accessors remain.

No source file changed, so no rustfmt run and no new test added. No generated data, WIT or cross-crate edit. No unwrap/expect/panic, no stdout, no em dashes. No cargo run (worker restriction; orchestrator compiles once). Known limitation unchanged: a displayed Bedrock objective title is not refreshed on update.


## Result (FINAL PASS 2026-11-18)

Definitive closure of every row. Re-read all seven "Remaining gaps" rows and the two former STILL OPEN verification items against the current `crates/pumpkin-core/src/world/scoreboard.rs` (its inline test modules and `world/scoreboard/tests.rs`) and decompiled vanilla under `.vendor/vanilla-decompiled/net/minecraft/`. No source file needed editing: every fix from earlier passes is present and matches vanilla. No generated data, WIT or cross-crate field is required, so no row is BLOCKED-DATA.

- Row 1 (M, Teams persistence): VERIFIED FIXED. `to_saved_nbt` emits the `Teams` list (`scoreboard.rs:986,999`) and `load_saved_nbt`/`load_teams` rebuild it (`:1068-1119`), writing/reading every `PlayerTeam.Packed.CODEC` field: Name, DisplayName, TeamColor, AllowFriendlyFire, SeeFriendlyInvisibles, MemberNamePrefix/Suffix, NameTagVisibility, DeathMessageVisibility, CollisionRule, Players. Field names match `.vendor/vanilla-decompiled/net/minecraft/world/scores/PlayerTeam.java:271-302` and the `Teams` list codec at `.vendor/vanilla-decompiled/net/minecraft/world/scores/ScoreboardSaveData.java:41-55`.
- Row 2 (M, Bedrock add/update objective display): VERIFIED FIXED. `add_objective`/`update_objective` call only `target.send_je(&je_update)` (`:268,302`); no Bedrock `SetDisplayObjective` is written. Vanilla `Scoreboard.onObjectiveAdded`/`onObjectiveChanged` are no-ops (`.vendor/.../world/scores/Scoreboard.java:462-466`) and `ServerScoreboard.onObjectiveChanged` broadcasts only `ClientboundSetObjectivePacket`, never a display packet (`.vendor/.../server/ServerScoreboard.java:172-187`). Pinned by `add_and_update_objective_do_not_write_a_bedrock_display` (scoreboard/tests.rs:450).
- Row 3 (S, Bedrock `sidebar.team.*` slot): VERIFIED FIXED. `bedrock_display_slot` returns `Some` only for List/Sidebar/BelowName and `None` for all other slots (`:831-838`); `set_display_objective` sends Java-only for team slots (`:332-336`). Pinned by inline `bedrock_slot_tests::team_display_slots_have_no_bedrock_slot` (`:1797`).
- Row 4 (S, number formats): VERIFIED FIXED. Objective and score `format` are written/read via `number_format_to_nbt`/`number_format_from_nbt` (`:842-869,904-908,924-928,1027,1049`), keyed by `type` (blank/styled/fixed) with `value` for fixed. Codec shape matches `NumberFormatTypes.MAP_CODEC` dispatch (`.vendor/.../network/chat/numbers/NumberFormatTypes.java:14-25`) and the `optionalFieldOf("format")` in `Objective.Packed.CODEC` (`.vendor/.../world/scores/Objective.java:148-159`) and `Score.Packed.MAP_CODEC` (`.vendor/.../world/scores/Score.java:68-77`). `styled` is omitted because Pumpkin's `Style` has no NBT codec (same limitation already noted in the network writer).
- Row 5 (S, one team per holder): VERIFIED FIXED. `add_player_to_team` retain-filters the holder out of every non-target team and no-ops the add only when already in the target (`:632-645`). Vanilla `Scoreboard.addPlayerToTeam` removes the holder from the previous team (`.vendor/.../world/scores/Scoreboard.java:385-404`) and `ServerScoreboard.addPlayerToTeam` broadcasts only the ADD packet, no RemovePlayers for the old team (`.vendor/.../server/ServerScoreboard.java:121-131`). Pinned by inline `team_membership_tests` (`:1807-1867`).
- Row 6 (S, reset packet for absent score): VERIFIED FIXED. `remove_score` returns early when the score is absent (`:505-512`). Vanilla `ServerScoreboard.onPlayerScoreRemoved` announces only after the score is actually removed (`.vendor/.../server/ServerScoreboard.java:88-95`).
- Row 7 (S, `get_objective_mut` desync): VERIFIED FIXED. No `get_objective_mut` exists anywhere in the crate; only immutable `get_objective`/`get_objectives` remain.
- Former STILL OPEN "BedrockScoreboard wrapping_add": VERIFIED FIXED. Both `Scoreboard::add_score` (`:494`) and `BedrockScoreboard::add_score` (`:1575`) use `wrapping_add`. Vanilla `Score.value(int)` is a plain Java int assignment (`.vendor/.../world/scores/Score.java:38-40`), so wrapping is the correct overflow emulation; a Rust debug `+` would panic.
- Former STILL OPEN "score map orientation": VERIFIED FIXED. `ScoreboardBuilder::score` inserts with objective as the outer key (`:1715`), matching every reader (`get_score` indexes `self.scores.get(objective)?.get(entity)`); pinned by `builder_scores_are_keyed_by_objective` (scoreboard/tests.rs:352).

All rows terminal: nine VERIFIED FIXED, zero LEFT OPEN / STILL OPEN, zero BLOCKED-DATA. No source file changed this pass, so no rustfmt run and no new test added. No unwrap/expect/panic, no stdout, no em dashes. No cargo run (worker restriction; orchestrator compiles once). Known limitation unchanged: a displayed Bedrock objective title is not refreshed on objective update, matching the existing Java-only `send_to_player` objective handling; the reported display-clobbering bug is removed.
