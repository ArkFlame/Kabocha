**FIXED** `1357273b3` (fix(entity): stop camels and camel husks blocking sleep). VERIFIED against code: entity/passive/camel.rs:466-468 (is_preventing_player_rest false); bed.rs NO_SLEEP_IDS has no camel ids.

# entities / Camel Husk — re-review audit (2026-11-17)

Tracker row flagged RE-REVIEW-UNTRACKED ("the reviewed source has changed"). Second pass, after the
2026-11-16 audit. Reviewed: `crates/pumpkin-core/src/entity/passive/camel.rs`,
`crates/pumpkin-core/src/entity/passive/camel/ai.rs`, `crates/pumpkin-core/src/entity/mob/zombie/husk.rs`,
`crates/pumpkin-core/src/entity/type.rs`, `crates/pumpkin-core/src/entity/passive/abstract_horse.rs`, and
the generated data (`crates/pumpkin-data/src/generated/entity_type.rs`,
`crates/pumpkin-data/src/generated/tag.rs`). No decompiled vanilla tree (`.vendor/`), so vanilla-side
claims remain cross-checked against generated data only. No cargo/tests run (worker rule).

## Source state since the last audit

`git diff bdc31379f..HEAD` over the reviewed files touches only `entity/type.rs` (falling-block initial
state, `type.rs:275-280`) — unrelated to the camel husk. `camel.rs`, `camel/ai.rs`, `husk.rs` have no
commits after `095f5c12c fix(entity): stop camel husks falling in love`. The "source changed" signal was
`4304228f5 feat(entity): bring zombie family to parity`, already covered by the previous audit.

## Fixed in this pass

None. The only real gap found by the previous pass (`095f5c12c`: camel husks fell in love and emitted
hearts although `can_mate` always refuses) is present in the tree: `camel.rs:382-386` gates on
`can_fall_in_love()`, and `camel.rs:422-424` returns `!is_husk() && !is_in_love()`.

## Verified from generated data this pass

- `CAMEL_HUSK` (`entity_type.rs:1821-1928`): 32 max health, 0.09 movement speed, 16 follow range,
  10 tempt range, 1.5 step height, 0.5 fall-damage multiplier, 2 XP, `limit_per_chunk` 6,
  `category MONSTER` with `is_enemy: false`, dimensions 1.7 x 2.375 with a Sitting pose 0.945 high.
  Rust reads all of these from data, no hardcoded duplicates.
- Sounds are fully data-wired (`entity_type.rs:1854-1871`): hurt/death/ambient/eat/sit/stand/dash/
  dash-ready/saddle/swim/fall all resolve to `EntityCamelHusk*`. `camel.rs:140-144` + the
  `abstract_horse_mob_hooks` ambient hook (`abstract_horse.rs:1362-1370`) use `entity_type.ambient_sound`.
- Undead family traits are tags, not code: `camel_husk` is in `#minecraft:undead` (`tag.rs:20706`),
  `#minecraft:zombies` (`tag.rs:20750`), `#minecraft:sensitive_to_smite` (`tag.rs:20670`),
  `#minecraft:inverted_healing_and_harm` (`tag.rs:20556`), `#minecraft:ignores_poison_and_regen`
  (`tag.rs:20502`), `#minecraft:wither_friends` (`tag.rs:20731`); it is absent from
  `#minecraft:burn_in_daylight`, so no sun burn. Correct for a husk.
- Riding allowed by data: `#minecraft:can_equip_saddle` (`tag.rs:20352`),
  `#minecraft:can_float_while_ridden` (`tag.rs:20367`).
- Food: `#minecraft:camel_husk_food` is `rabbit_foot` (`tag.rs:24146`), used at `camel.rs:414-420`.
- Spawn rules: `type.rs:507-517` routes `CAMEL_HUSK` through `check_surface_monsters_spawn_rules`, which
  matches the `MONSTER` category + `SpawnLocation::OnGround` / `MotionBlockingNoLeaves` data.

## Assumptions revised this pass

- **Persistence on hand interaction is probably correct, not a gap.** `camel.rs:468-480` sets
  `persistence_required` and returns `None`, i.e. sets it then defers to the shared interaction. That is
  exactly the shape of a vanilla `CamelHusk.mobInteract` override (`setPersistenceRequired()` then
  `super.mobInteract(...)`), and no other mob in `crates/pumpkin-core/src/entity` uses the hook, so it was
  added deliberately rather than copy-pasted. Downgraded from "gap" to "confirm in `.vendor/`".
- The default `is_preventing_player_rest` is `entity/mob/mod.rs:2491` (the previous audit cited 2485).
  Still a family-wide base-default issue, not this row.

## Remaining gaps

1. **Numeric constants unconfirmed** — `HUSK_STEP_VOLUME = 0.4` (`camel.rs:48`, used `camel.rs:453-455`)
   and the `4.0` husk charge modifier (`camel.rs:434-440`, camel 1.0). Plausible (vanilla camel step
   volume is also 0.4 and camel husks charge faster), but not proven without `CamelHusk.java`.
   Suggested fix: drop decompiled sources into `.vendor/` and diff. Size M.
2. **Jockey formation unconfirmed** — `mob/zombie/husk.rs:63-88`: on a natural spawn with group data and
   empty space, 10% of the time it spawns a camel husk, a `PARCHED` riding it, and keeps the spawning
   husk as a second passenger (`set_pending_vehicle`); the iron spear is given to the husk, not the
   parched. Vanilla news text describes "a Parched riding a Camel Husk holding an iron spear", which
   suggests the spear belongs to the rider and that the spawning husk may be replaced rather than kept.
   Whether vanilla keeps two riders is unverified. Suggested fix: compare `Husk.finalizeSpawn` in
   1.21.10+ and add a `pumpkin-gametest` asserting rider count and spear holder. Size M.
3. **`is_preventing_player_rest` inherits `true`** — default `entity/mob/mod.rs:2491`; a camel husk
   (an `AbstractHorse`, so an `Animal`) blocks nearby beds here, whereas vanilla `Animal` returns false.
   Family-wide default; fix belongs to the `Mob`/`Animal` defaults. Size S, out of scope for this row.
4. **`CamelHusk.canFallInLove` / persistence / `isMobControlled` need `.vendor/` confirmation** — the
   current readings (false / persist-on-interact / first passenger is a mob) are the only self-consistent
   ones given `can_mate == false` and the jockey path, but each rests on inference. Size S.

## Superseded file

`docs/tracker/tentities-camel-husk.md` is the earlier audit of the same row (pre-`095f5c12c`); its
"falling in love" gap is fixed and its line numbers have drifted. This file supersedes it.

## Pass 3 (2026-11-17, HEAD 3a30dba8b, master 158272eac)

The "source changed" signal is stale again. `git log` on `camel.rs`, `camel/ai.rs` and `husk.rs`
still ends at `095f5c12c`, and `git log HEAD..master -- <those files> abstract_horse.rs mob/mod.rs
animal.rs` is empty: no upstream commit touched the reviewed source between this branch base and
current master. Re-read the whole pass-2 diff; nothing below is invalidated.

### Fixed in this pass

- **Camel and camel husk blocked nearby beds.** `Mob::is_preventing_player_rest` defaults to
  `true` (`entity/mob/mod.rs:2512`), but vanilla `Animal.isPreventingPlayerRest` returns `false`,
  and `CamelHusk` is a `Camel` (an `Animal`). Added the override in `camel.rs:465-470`, which
  covers both the camel and the camel husk because they share `CamelEntity`. Pass-2 gap 3 is now
  fixed for this row; the same default still affects every other `Animal` in the tree (cow, sheep,
  ...). The family-wide fix belongs in the `Mob` default (`as_animal().is_none()` plus explicit
  `Villager` handling), noted here but out of this row's scope. Size S.

### Confirmed, not gaps

- **Persistence on hand interaction** (`camel.rs:474-486`) is the same deliberately used pattern as
  `ZombieHorseEntity::mob_interact_with_hand` (`passive/zombie_horse.rs:168-180`), so it is not
  husk-only copy-paste. Pass-2 gap 4's persistence question is closed; `canFallInLove` and
  `isMobControlled` readings still come from inference.

### Remaining gaps (unchanged, line numbers shifted +6 after camel.rs:461)

1. **Numeric constants unconfirmed** - `HUSK_STEP_VOLUME = 0.4` (`camel.rs:48`, used
   `camel.rs:459-461`) and the `4.0` husk charge modifier (`camel.rs:434-440`, camel 1.0).
   Plausible but unproven without `CamelHusk.java`. Fix: drop decompiled sources into `.vendor/`
   and diff. Size M.
2. **Jockey formation unconfirmed** - `mob/zombie/husk.rs:63-88` gives the spear to the spawning
   husk, then mounts a parched on the new camel husk and the husk behind it (three entities). The
   analogous zombie-horse jockey (`passive/zombie_horse.rs:86-100`) spawns exactly one rider and
   gives that rider the spear, which suggests the husk path should keep two entities and the spear
   on the parched, but vanilla `Husk.finalizeSpawn` is not available to confirm whether the
   spawning husk is discarded. Fix: compare with vanilla 1.21.10+ and add a `pumpkin-gametest`
   asserting rider count and spear holder. Size M.
3. **`is_preventing_player_rest` family default** - still `true` at `entity/mob/mod.rs:2512`; only
   `zombified_piglin.rs:180` overrides. Row scope fixed above; other Animals and villagers still
   block sleep. Size S, belongs to `Mob`/`Animal` defaults.
4. **`canFallInLove` / `isMobControlled` need `.vendor/` confirmation** - the current readings
   (false / first passenger is a mob) are the self-consistent ones given `can_mate == false` and
   the jockey path. Size S.
