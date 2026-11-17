# Command parity sweep 2 (vanilla 1.21.x, from memory, UNVERIFIED)

Scope: tp/teleport, summon, setblock, clone, title, tellraw, say, msg, scoreboard objectives, team, bossbar, advancement, attribute, item replace. Sweep 1 covered gamemode through fill.

Skipped by policy: time, data, compute, locate, enchant/effect/fill (enchant got a unit test only).

| Command | Status | Note |
|---|---|---|
| enchant | TESTED (UNVERIFIED) | Level bound extracted to `level_exceeds_max` with a boundary unit test. Vanilla rejects only levels strictly above max. |
| tp/teleport | FIXED (UNVERIFIED) | Success message coordinates used Rust `to_string()`. Vanilla `TeleportCommand` prints `%f` (six decimals). Fixed via `format_coordinate` with a unit test. |
| summon | FIXED (UNVERIFIED) | Peaceful check used `!category.is_friendly`, blocking misc entities. Now `is_blocked_on_peaceful` rejects only `MobCategory::MONSTER` (shared with `/execute summon`). Error key is `commands.summon.failed` per spec. Zombie-horse (MONSTER category, not `instanceof Monster`) still blocked; needs an entity-level check. Unit test added. |
| summon | OPEN (UNVERIFIED) | `finalize_spawn` is skipped when NBT is given. Vanilla finalizes mobs by instanceof check, not by NBT presence. Verify against decompiled `SummonCommand` before changing. |
| say | NO DIVERGENCE FOUND | Level 2 permission and broadcast path match vanilla as far as recalled. |
| msg/tell/w | NO DIVERGENCE FOUND | Level 0, aliases redirect to `msg`, returns target count. |
| tellraw | NO DIVERGENCE FOUND | Level 2, returns player count. |
| clone | FIXED (UNVERIFIED) | Too-big cap read `maxBlockModifications`. Vanilla `CloneCommands.MAX_CLONE_AREA` is a fixed 32768. Now `exceeds_clone_area` with a boundary test. Overlap check already matches vanilla (skipped for force/move). |
| setblock | NO DIVERGENCE FOUND | Keep fails on non-air, same-state result fails, destroy/replace/strict flags present. |
| title, scoreboard, team, bossbar, advancement, attribute, item | NOT SWEPT | Out of this pass's budget. |

## Sweep 2 pass (scoreboard/team/title/bossbar/advancement/attribute/item)

| Command | Status | Note |
|---|---|---|
| scoreboard objectives add | FIXED (UNVERIFIED) | Name over 16 UTF-16 units now fails with `commands.scoreboard.objectives.add.tooLong` before the duplicate check. Unit test `name_limit_counts_utf16_units`. |
| team add | FIXED (UNVERIFIED) | Same 16-unit limit, key `commands.scoreboard.teams.add.tooLong`, shares `name_exceeds_limit` from scoreboard.rs. |
| scoreboard objectives add/display name length | OPEN (UNVERIFIED) | Vanilla `displayTooLong` cap not enforced; exact limit not recalled reliably. |
| title/subtitle/actionbar/times/clear/reset | NO DIVERGENCE FOUND | Keys `commands.title.show.{mode}.single/multiple`, `cleared`, `reset`, `times` match vanilla. |
| bossbar | NO DIVERGENCE FOUND | `max` min 1, value min 0, colour/style literals, `commands.bossbar.set.*.unchanged` keys present. |
| team modify friendlyFire/seeFriendlyInvisibles/nametagVisibility/collisionRule | NO DIVERGENCE FOUND | Literals and `commands.team.option.*.alreadyEnabled/alreadyDisabled/unchanged` error paths match vanilla as recalled (UNVERIFIED). |
| team modify deathMessageVisibility | OPEN (UNVERIFIED) | Handler sends success feedback but never stores the value: `Team` has no death-message field. Needs a model and packet change, not a one-line fix. |
| team modify color | OPEN (UNVERIFIED) | Default colour is `White` here; vanilla `PlayerTeam` starts at `RESET`, so `color white` on a fresh team errors here but succeeds in vanilla. Representation change, not fixed. |
| team modify displayName/prefix/suffix length | NO DIVERGENCE FOUND | No length cap in vanilla modify paths as recalled; existing behaviour kept. |
| scoreboard players operation `=,+=,-=,*=,/=,%=,<,>,><` | NO DIVERGENCE FOUND | Zero divisor errors with `arguments.operation.div0` instead of skipping. Kept: vanilla `OperationArgument` raises that error as recalled. Swap and min/max match. |
| scoreboard players operation | NO DIVERGENCE FOUND | Missing target and source scores initialise to 0; result is the sum of final target scores. `players reset` not re-checked this pass. |
| scoreboard objectives modify displayname/rendertype | NO DIVERGENCE FOUND | Feedback always sent; unchanged-display-name silent-path not recalled reliably, kept. |
| advancement grant/revoke only/from/through/until/everything | NO DIVERGENCE FOUND | Mode parent/child traversal, grant skips done advancements, revoke skips no-progress, one/many success and failure keys match vanilla as recalled. |
| attribute get/base get/base set/base reset/modifier add/remove/value get | NO DIVERGENCE FOUND | Error keys `commands.attribute.failed.{entity,no_attribute,no_modifier,modifier_already_present}` and success keys match; scale truncates to int as vanilla. |
| item replace slot names | NO DIVERGENCE FOUND | Names come from generated `pumpkin_data::slot_ranges`. Entity mapping covers `container/hotbar/weapon/armor/enderchest`, `armor.body`=105 and `armor.saddle`=106. |
| item replace `villager.N` / `horse.*` | OPEN (UNVERIFIED) | Slot ids 300+ and 500+ have no entity container mapping, so they fail as no-such-slot. Needs villager and horse inventories. |
