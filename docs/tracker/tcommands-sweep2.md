# Command parity sweep 2 (vanilla 1.21.x, from memory, UNVERIFIED)

Scope: tp/teleport, summon, setblock, clone, title, tellraw, say, msg, scoreboard objectives, team, bossbar, advancement, attribute, item replace. Sweep 1 covered gamemode through fill.

Skipped by policy: time, data, compute, locate, enchant/effect/fill (enchant got a unit test only).

| Command | Status | Note |
|---|---|---|
| enchant | TESTED (UNVERIFIED) | Level bound extracted to `level_exceeds_max` with a boundary unit test. Vanilla rejects only levels strictly above max. |
| tp/teleport | FIXED (UNVERIFIED) | Success message coordinates used Rust `to_string()`. Vanilla `TeleportCommand` prints `%f` (six decimals). Fixed via `format_coordinate` with a unit test. |
| summon | OPEN (UNVERIFIED) | Peaceful check uses `!category.is_friendly`. Vanilla checks `instanceof Monster`, so non-monster misc entities (armor_stand, boat, item) are wrongly blocked. Needs a monster classification on entity types, not in `MobCategory`. Not changed. |
| summon | OPEN (UNVERIFIED) | `finalize_spawn` is skipped when NBT is given. Vanilla finalizes mobs by instanceof check, not by NBT presence. Verify against decompiled `SummonCommand` before changing. |
| say | NO DIVERGENCE FOUND | Level 2 permission and broadcast path match vanilla as far as recalled. |
| msg/tell/w | NO DIVERGENCE FOUND | Level 0, aliases redirect to `msg`, returns target count. |
| tellraw | NO DIVERGENCE FOUND | Level 2, returns player count. |
| setblock, clone, title, scoreboard, team, bossbar, advancement, attribute, item | NOT SWEPT | Read budget spent on the rows above. Permission levels and return values grepped only; no confident divergence. |
