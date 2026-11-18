# Parity rows for crates/pumpkin-core/src/entity/mob/zombie/mod.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-en-b134b2** [partial/OWN-OPEN] own audit: 5 open, 1 fixed, 0 verified, 2 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-en-b134b2.md) docs/tracker/taudit-crates-pumpkin-core-src-en-b134b2.md

## Result
CLOSED (verified) on the third pass. The decompiled vanilla tree is now present, so all four
STILL OPEN rows of the audit were re-checked directly against `.vendor/vanilla-decompiled/net/minecraft/`.
Every row already matches vanilla; no source file was edited and rustfmt was not run. No pure
unit-test seam exists for any row (each depends on a live world/entity state or the host clock).

- **Fire-on-hit main-hand guard** (`do_hurt_target`, zombie/mod.rs:573-586) - VERIFIED.
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/zombie/Zombie.java:349-353`
  gates the ignite on `getMainHandItem().isEmpty() && isOnFire() && random < effectiveDifficulty*0.3`,
  matching Pumpkin's `main_hand_empty && is_on_fire() && random::<f32>() < difficulty * 0.3`.
  The formerly BLOCKED-DATA row resolves as correct-as-is.
- **Chicken jockey candidate filter** (`roll_chicken_jockey`, zombie/mod.rs:332-335) - VERIFIED.
  `Zombie.java:492-493` selects with `EntitySelector.ENTITY_NOT_BEING_RIDDEN` (alive, no
  passengers); Pumpkin's `is_alive() && !has_passengers()` matches. Filter needs a live world,
  so no unit test.
- **Reinforcement condition/RNG order** (`summon_reinforcements`, zombie/mod.rs:465-489) - VERIFIED.
  `Zombie.java:293-301` short-circuits `target != null && difficulty == HARD &&
  random.nextFloat() < chance && level.isSpawningMonsters()`. Pumpkin resolves the target
  (469-476), checks difficulty (477-479), then rolls and checks `should_spawn_monsters()` in
  the same short-circuit chain (481-489). Neither target lookup nor the difficulty read
  consumes RNG, so the stream and order are identical.
- **Halloween timezone** (`is_halloween`, mob/spawn.rs:26-29) - VERIFIED.
  `.vendor/vanilla-decompiled/net/minecraft/util/SpecialDates.java:16-22` uses
  `MonthDay.from(ZonedDateTime.now())` (host process default zone); Pumpkin's
  `OffsetDateTime::now_local()` (UTC fallback) is the equivalent host-local clock. Cosmetic only.

No row remains STILL OPEN.

## Result (batch x_16, CODER pass)
- own/taudit-crates-pumpkin-core-src-en-b134b2: CLOSED, VERIFIED. Each of the 4 rows is checked against `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/zombie/Zombie.java` (349-353 fire-on-hit, 293-301 reinforcement, 492-498 chicken jockey) and `util/SpecialDates.java:16-22`. Rust `crates/pumpkin-core/src/entity/mob/zombie/mod.rs` (fire-on-hit ~581, chicken filter 332-335, reinforcements 465-489) and `mob/spawn.rs:26`. No Rust edits needed.
Status: PASS fixed=0 verified=4 open=0 blocked=0 files=none edited
