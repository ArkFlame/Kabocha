# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-b134b2.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: (none named; broadened edit allowed, no edits required)

## Result (final pass)

All rows for this unit are closed. Verified against decompiled vanilla in `.vendor/vanilla-decompiled/`; no `.rs` files edited, so rustfmt not run. No pure unit-test seam exists (wall-clock date and live world/entity state).

- **Fire-on-hit main-hand guard** (`crates/pumpkin-core/src/entity/mob/zombie/mod.rs:581-584`) - VERIFIED, correct as-is. `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/zombie/Zombie.java:349-353` requires `getMainHandItem().isEmpty() && isOnFire() && random.nextFloat() < difficulty * 0.3F`.
- **Chicken jockey candidate filter** (`crates/pumpkin-core/src/entity/mob/zombie/mod.rs:332-336`) - VERIFIED, matches vanilla. `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/zombie/Zombie.java:492-498` uses `EntitySelector.ENTITY_NOT_BEING_RIDDEN` and random candidate selection; `.vendor/vanilla-decompiled/net/minecraft/world/entity/EntitySelector.java:12` is `isAlive() && !isVehicle() && !isPassenger()`.
- **Reinforcement condition/RNG order** (`crates/pumpkin-core/src/entity/mob/zombie/mod.rs:465-489`) - VERIFIED, correct as-is. `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/zombie/Zombie.java:293-301` short-circuits target, `difficulty == HARD`, RNG roll, and `isSpawningMonsters()` in that order; Pumpkin matches the RNG/short-circuit behaviour (target/difficulty do not consume RNG).
- **Halloween timezone** (`crates/pumpkin-core/src/entity/mob/spawn.rs:26-29`) - VERIFIED, correct as-is. `.vendor/vanilla-decompiled/net/minecraft/util/SpecialDates.java:16-22` uses `ZonedDateTime.now()` (host default timezone); Pumpkin uses `OffsetDateTime::now_local()` with UTC fallback. Cosmetic only.
