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
