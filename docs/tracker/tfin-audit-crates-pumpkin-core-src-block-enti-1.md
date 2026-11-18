# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-block-enti.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: sculk_catalyst.rs spreader.rs

| 5 | `entities/sculk_catalyst.rs:62` | `level.getRandom()` | Reseeds a Xoroshiro from OS entropy every tick, so spreading is not reproducible from the world seed. Same pattern in `fire`, `stem`, `jigsaw_block`, `grass_block`; not catalyst-specific, fix house-wide. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 6 | `entities/sculk_catalyst.rs:132-172` | Souls arrive through the `GameEvent.ENTITY_DIE` listener (BY_DISTANCE, radius 8) | Manual dispatch from `LivingEntity::on_death` only, so any entity removed without that path (despawn/`discard`, direct `remove`) never feeds a catalyst. Equivalent for normal deaths; revisit when a game-event system lands. | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 7 | `pumpkin-world/.../sculk/spreader.rs:81-83` | `SculkSpreader.createLevelSpreader()` constants | Level spreader uses 10/4/10/5 (growth spawn cost, no-growth radius, charge decay, additional decay); not verifiable without a decompile, and shares the file's charge-merge logic. | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
- Row 1 (bloom particles): STILL OPEN. Client-only cosmetic; needs a level-event/particle packet the server does not expose, outside this unit's three files.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
