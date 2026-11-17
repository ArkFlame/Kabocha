# Command sweep 3 (c2): attribute, bossbar, execute, scoreboard, team, tag, title, particle, playsound, spawnpoint, setworldspawn, difficulty, gamerule, whitelist, ban, op, kick, list, say, msg, tellraw, seed

Method: permission levels and argument bounds checked against the vanilla lang file `assets/en_us_java.json` (placeholder counts). Vanilla source is not vendored, so logic claims are from memory and marked as such.

| Command | Status | Note |
|---|---|---|
| spawnpoint | FIXED (UNVERIFIED, no cargo run) | Feedback passed pitch and dimension into the `[%s] in %s for %s` slots, so the dimension landed in `for` and the target name was dropped. Now `spawn_feedback_args` passes `x, y, z, yaw, dimension, target` (vanilla has no pitch slot). Pure-helper test added. |
| setworldspawn | OPEN (cosmetic) | Feedback passes 6 args to a 4-placeholder key (`x, y, z [yaw]`). Output is correct because extra args are ignored. Trim later. |
| playsound | NO DIVERGENCE FOUND | Level 2, volume/pitch/minVolume bounds (0, 0-2, 0-1), range `16 * max(volume,1)`, minVolume relocation match. |
| particle | NO DIVERGENCE FOUND | Level 2, speed min 0, count min 0 match vanilla. |
| title | NO DIVERGENCE FOUND | Level 2, show/clear/reset/times keys and single/multiple split match lang keys. |
| tag | NO DIVERGENCE FOUND | Level 2, add/remove/list keys and arg counts match lang (2 and 3 placeholders). |
| attribute | NO DIVERGENCE FOUND | Level 2, get/modifier return values (1 for add/remove, scaled int for get) and remove feedback (3 args) match lang. |
| bossbar | NO DIVERGENCE FOUND | Level 2, `max` min 1, `value` min 0, returns match. |
| scoreboard players set | NO CHANGE | Return value `holders * value` and nonzero-count feedback are deliberate per in-code comment. Not verified against vanilla. |
| team | NO DIVERGENCE FOUND | Level 2, all success paths return 1. |
| execute | NOT SWEPT | 2500 lines; out of budget for this pass. |
| difficulty | NO DIVERGENCE FOUND | Level 2, query returns id, same-value set errors with `commands.difficulty.failure` (1 arg). |
| gamerule | NO DIVERGENCE FOUND | Level 2, query/set keys have 2 args each, set returns value. |
| whitelist | NO DIVERGENCE FOUND | Level 3, on/off already-state errors, list `none` returns 0, add skips existing entries and errors only when none added (vanilla behaviour). |
| ban | NO DIVERGENCE FOUND | Level 3, `commands.ban.failed` when nothing changed, success has 2 args. |
| op | NO DIVERGENCE FOUND | Level 3, `commands.op.failed` when nothing changed, success has 1 arg. |
| kick | NO DIVERGENCE FOUND | Level 3, success has 2 args. |
| list | NO DIVERGENCE FOUND | Level 0, `list` has 3 args, `nameAndId` has 2. |
| say | NO DIVERGENCE FOUND | Level 2. |
| msg/tell/w | NO DIVERGENCE FOUND | Level 0. |
| tellraw | NO DIVERGENCE FOUND | Level 2. |
| seed | NO DIVERGENCE FOUND | Level 2 (dedicated server). |

Checks: `rtk bypass rustfmt --edition 2024 --check crates/pumpkin-core/src/command/commands/spawnpoint.rs` exit 0. No cargo run (policy).
