# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-entity-mob-zombie-mod-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

STILL OPEN rows of the audit were re-checked directly against `.vendor/vanilla-decompiled/net/minecraft/`.
No row remains STILL OPEN.

## Result
No open rows remain. The audit's STILL OPEN rows were re-checked against `.vendor/vanilla-decompiled/net/minecraft/`.

## Result (batch x_16, CODER pass)
- Zero STILL OPEN rows in this file. Checked the referenced audit `taudit-crates-pumpkin-core-src-en-b134b2.md`: its four rows (fire-on-hit guard, chicken filter, reinforcement order, Halloween zone) are VERIFIED against vanilla `Zombie.java:293-301,349-353,492-498`, `EntitySelector.java:12`, `SpecialDates.java:16-22`. Rust: `zombie/mod.rs:334` (filter `is_alive && !has_passengers && !has_vehicle` equals `ENTITY_NOT_BEING_RIDDEN`), `mob/spawn.rs:26`.
Status: PASS fixed=0 verified=4 open=0 blocked=0 files=none edited

## Result (CODER pass, no cargo)
- Zombie `zombie/mod.rs:334` reinforcement/rider filter (`is_alive && !has_passengers && !has_vehicle`) spot-checked; the four audit rows remain VERIFIED. No source edit.
- Status: PASS fixed=0 verified=4 open=0 blocked=0.
