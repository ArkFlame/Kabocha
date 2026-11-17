# entities / Camel Husk — re-review audit (2026-11-16)

Source re-reviewed (changed since last assessment):
`crates/pumpkin-core/src/entity/passive/camel.rs`, `crates/pumpkin-core/src/entity/passive/camel/ai.rs`,
`crates/pumpkin-core/src/entity/mob/zombie/husk.rs` (jockey spawn), `crates/pumpkin-core/src/entity/type.rs`.
No decompiled vanilla tree (`.vendor/`) or network access in this worktree, so numeric constants could only be
cross-checked against `pumpkin-data` / generated tags, not against `CamelHusk.java` line by line. No cargo run (worker rule).

## Verified correct (no change needed)

- Husk flag derives from `EntityType::CAMEL_HUSK` (`camel.rs:66`), not from NBT, so it survives save/load.
- Undead traits are data-driven and correct: `camel_husk` is in `#minecraft:undead` and `#minecraft:zombies`
  (`crates/pumpkin-data/src/generated/tag.rs:20697`), giving Smite / inverted healing; it is absent from
  `#minecraft:burn_in_daylight`, so `sun_burn::tick` (`entity/mob/sun_burn.rs:117`) does not burn it. Matches
  vanilla husk behaviour without a per-mob override.
- Not a vanilla `Enemy`: `target_match.rs:20-34` excludes `CAMEL_HUSK` from `TargetMatch::Enemy`, matching vanilla's
  non-hostile undead mount (no attack goals in the brain is therefore correct).
- No breeding / no babies: `can_mate` refuses for husk and only accepts `CAMEL` (`camel.rs:422`), `can_be_a_baby` is
  false for husk (`camel.rs:407`), `is_food` uses `#minecraft:camel_husk_food` (`camel.rs:416`).
- Despawn: `remove_when_far_away` returns `is_husk()` (`camel.rs:461`), monster limit 6/chunk from data.
- Saddling/riding is allowed by vanilla data (`camel_husk` is in `#minecraft:can_equip_saddle`), so the current
  `mob_interact` ride path (`camel.rs:605+`) is not a gap.
- Pose persistence: `LastPoseTick` written/read as long and re-applies `EntityPose::Sitting`
  (`camel.rs:478-500`), matching vanilla `Camel.addAdditionalSaveData` / `readAdditionalSaveData`.
- Sitting blocks movement and clamps head yaw (`travel`, `subclass_ai_step`) and water forces stand-up; dash
  cooldown/min-duration constants and dash-ready sound wired through `entity_type` data (`camel.rs:105-146`).

## Remaining gaps / open questions

1. **Husk can fall in love** — `camel.rs:382-386` (`could_set_in_love` uses `is_tamed()`, and husks are
   `always_tamed`, `camel.rs:428`) so a fed camel husk emits love hearts and can start `AnimalMakeLove`
   (`camel/ai.rs:72`) even though `can_mate` always refuses. Vanilla `CamelHusk` cannot breed, so `handleEating`
   should gate on `!is_husk()`. Needs vanilla confirmation of `CamelHusk.canFallInLove`; size S.
2. **Love/tempt goals present in the husk brain** — `camel/ai.rs:66-97` runs `AnimalMakeLove` and
   `FollowTemptation` for husks. Harmless today (breeding is refused) but extra sensor work and hearts if #1 is
   fixed only partially. Size S if confirmed against `CamelHusk.createBrain`.
3. **`is_preventing_player_rest` inherits `true`** — default at `entity/mob/mod.rs:2392`; vanilla camel husk extends
   `AbstractHorse` (an `Animal`), which returns false, so a camel husk blocks nearby beds here. Family-wide base
   default issue (affects every non-overriding mob, e.g. camel), not husk-only; fix belongs to the `Mob`/
   `Animal` defaults, not this row. Size S but out of this row's scope.
4. **Husk step volume 0.4** (`camel.rs:48`, used at `:450-451`) and **`charge_speed_modifier` 4.0**
   (`camel.rs:430`) are unverified numeric constants; they need `CamelHusk.java` / `Camel.java` decompiled values.
   Size M (needs decompiled reference drop in `.vendor/`).
5. **Jockey pairing** — `mob/zombie/husk.rs:64-90` spawns camel husk + parched rider at 10% on natural spawn and
   the rider is set as pending rider; consistent with the husk jockey path, but the camel-husk-only jockey
   variant (husk on camel husk) and the `#minecraft:camel_husk` spawn placement were not exercised at runtime.
   Size M (runtime/gametest).

No code change committed from this audit: nothing above is certain enough to edit without the decompiled
`CamelHusk` source, and item 3 is out of row scope.
