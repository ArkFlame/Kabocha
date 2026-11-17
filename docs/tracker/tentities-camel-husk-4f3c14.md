# entities / Camel Husk — re-review audit (2026-11-16)

Source re-reviewed because it changed since the prior assessment (commit 4304228f5 "feat(entity): bring
zombie family to parity" added `charge_speed_modifier` to `CamelEntity`). Reviewed:
`crates/pumpkin-core/src/entity/passive/camel.rs`, `crates/pumpkin-core/src/entity/passive/camel/ai.rs`,
`crates/pumpkin-core/src/entity/mob/zombie/husk.rs`, `crates/pumpkin-core/src/entity/type.rs`.
No decompiled vanilla tree (`.vendor/`) and no network access in this worktree, so numeric constants were
cross-checked against `pumpkin-data`/generated tags only. No cargo/tests run (worker rule).

## Fixed in this pass

- **Camel husk could fall in love.** `camel.rs:373-385` (`handle_eating`) computed
  `could_set_in_love = is_tamed() && age == 0 && !is_in_love()`. Husks are `always_tamed`
  (`camel.rs:431`), so every adult camel husk fed its food (rabbit foot, `#minecraft:camel_husk_food`)
  emitted love hearts and started the love timer, while `can_mate` (`camel.rs:425`) always refuses.
  Vanilla `CamelHusk` (undead, not breedable) returns false from `canFallInLove`, and
  `AbstractHorse.handleEating` gates the love branch on `canFallInLove()`. Added
  `CamelEntity::can_fall_in_love` (`camel.rs:422-424`, `!is_husk() && !is_in_love()`) and used it in
  `handle_eating`, matching the vanilla call shape. The husk brain's `AnimalMakeLove` (`camel/ai.rs:72`)
  can no longer be fed love state either.

## Verified correct (no change needed)

- Husk flag from `EntityType::CAMEL_HUSK` (`camel.rs:66`), not NBT, survives save/load.
- Undead traits are data-driven: `camel_husk` is in `#minecraft:undead`/`#minecraft:zombies`
  (`pumpkin-data/src/generated/tag.rs:20697`) but not `#minecraft:burn_in_daylight`, so Smite/inverted
  healing apply and sun burn does not. Matches husk-family behaviour.
- Not an `Enemy`: excluded from `TargetMatch::Enemy` (`entity/ai/target_match.rs:32`), so no attack goals
  is correct; `MobCategory::MONSTER` + `is_enemy: false` in `pumpkin-data/.../entity_type.rs:1875-1881`.
- No breeding/babies: `can_mate` husk-refusing (`camel.rs:425`), `can_be_a_baby` false for husk
  (`camel.rs:406`), `is_food` uses `#minecraft:camel_husk_food` (`camel.rs:413`).
- Despawn: `remove_when_far_away` returns `is_husk()` (`camel.rs:463`); monster 6/chunk from data.
- Saddling/riding allowed by vanilla data (`#minecraft:can_equip_saddle`), so the ride path
  (`camel.rs:608+`) is not a gap.
- Pose persistence: `LastPoseTick` long write/read and sitting re-application (`camel.rs:481-500`).
- Sitting blocks movement, clamps head yaw, water forces stand-up; dash cooldown/min-duration and
  dash-ready sound are data-wired (`camel.rs:106-133`).
- `charge_speed_modifier` for the husk is `4.0` vs `1.0` for a camel (`camel.rs:433-439`); the branch is
  keyed on `self.husk`'s type and is only consumed by spear charging (`ai/goal/spear_use.rs:157`). It rides
  the vehicle path as vanilla `CamelHusk.getChargeSpeedModifier`.

## Remaining gaps

1. **Unverified numeric constants** — `HUSK_STEP_VOLUME = 0.4` (`camel.rs:48`, used `camel.rs:453-454`) and
   the `4.0` charge modifier (`camel.rs:433-439`) need `CamelHusk.java`/`Camel.java` line-by-line
   confirmation. Suggested fix: drop the decompiled sources into `.vendor/` and diff. Size M.
2. **Jockey formation** — `mob/zombie/husk.rs:66-88` spawns a camel husk, a `PARCHED` rider on it, and sets
   the spawning husk as a second passenger (holding an iron spear), gated by `group_data.is_some()` and
   10%. Whether vanilla `Husk.finalizeSpawn` spawns the parched, the husk, or both, and whether the husk
   itself should be the single rider, is unconfirmed. Suggested fix: compare against the 1.21.x
   `Husk.finalizeSpawn` and add a `pumpkin-gametest` that spawns at most 2 riders. Size M.
3. **`is_preventing_player_rest` inherits `true`** — default at `entity/mob/mod.rs:2485`; a camel husk
   (an `Animal`/`AbstractHorse`) blocks nearby beds here. Family-wide default issue (affects every
   non-overriding mob), so fix belongs to the `Mob`/`Animal` defaults, not this row. Size S, out of scope.
4. **Persistence on any hand interaction** — `mob_interact_with_hand` (`camel.rs:467-479`) sets
   `persistence_required` for a husk whatever the item. Vanilla `CamelHusk` persistence trigger
   (`mobInteract` vs `handleEating`) is unconfirmed. Suggested fix: confirm in `CamelHusk.java` whether
   only feeding/saddling persists; if so gate on `handle_important_interactions`/`is_food`. Size S.
5. **Vanilla `CamelHusk.canFallInLove` confirmation** — the fix above rests on the undead/no-breeding
   invariant and `can_mate == false`; when `.vendor/` is available, confirm `CamelHusk` returns false
   (it is the only stable reading: it cannot produce offspring). Size S.
