# entities / Piglin (RE-REVIEW)

Reviewed: `crates/pumpkin-core/src/entity/mob/piglin.rs`, `piglin_ai.rs`, `piglin_brute.rs`, `zombified_piglin.rs`, plus the piglin branches of `crates/pumpkin-core/src/entity/type.rs::check_spawn_rules`.

## Fixed this pass

- `piglin_ai.rs:280` throw direction used `target.y + 1.0 - pos.y`, an invented +1 offset that punted barter drops upward. Vanilla `PiglinAi.throwItemsTowardPos` uses `target.y - (eyeY - 0.3)` from the hand position. Commit: `fix(entity): piglin barter throw velocity and item pickup delay`.
- `piglin_ai.rs:298` thrown items used `ItemEntity::DEFAULT_PICKUP_DELAY` (10). Vanilla sets `setPickUpDelay(40)` on every barter/inventory-overflow drop. Same commit.

## Previously listed gaps still open

| file:line | vanilla | suggested fix | size |
|---|---|---|---|
| piglin.rs:112 (single adult goal set), piglin_ai.rs:33 (`BABY_FLEE_DURATION` unused) | `BabyPiglinTasks` gives babies no melee/hunt goals; they flee 100 ticks from an attacker and `Play` | build a baby goal set in `PiglinEntity::new` (drop `MeleeAttackGoal`/`RangedCrossbowAttackGoal`/hoglin target goal, add a flee goal driven by `BABY_FLEE_DURATION`) | M |
| piglin_ai.rs:37 (`DESIRED_DISTANCE_FROM_ZOMBIFIED` unused) | `PiglinAi.avoidZombified`: adult piglins keep 6 blocks from `ZOMBIFIED_PIGLIN`/`ZOGLIN` | add an avoidance goal or target-goal exclusion at 6 blocks, adults only | S |
| piglin.rs:753 (`was_hurt_by`), piglin_ai.rs:29 (`ANGER_DURATION` unused) | `PiglinAi.angerNearbyPiglins`: damaging a piglin angers nearby piglins (600 ticks) at the attacker | add a `angered_by`/`anger_timer` pair set from `was_hurt_by`, target through it in `ActiveTargetGoal` instead of relying on `RevengeGoal` only | M |
| piglin.rs:1192 (`killed_entity`) | `PiglinTasks.setHuntedRecently` also fires when a hunt ends without a kill | set the 600..=2400 `hunt_cooldown_timer` when a hoglin target is dropped/lost, not only in `killed_entity` | S |

## Resolved / re-checked, no action

- `check_piglin_spawn_rules` (piglin.rs:805) is dead code: the nether-wart-block rule is already applied inline for `PIGLIN` in `entity/type.rs:587`. Vanilla `Piglin.checkPiglinSpawnRules` ignores the spawn type as well, so the spawn-type row from the previous review is not a gap. Same for `piglin_brute.rs:107` (`PIGLIN_BRUTE` only comes from bastions). Optional cleanup, not behaviour.
- `stop_holding_off_hand_item` (piglin.rs:709) matches `PiglinAi.stopHoldingOffHandItem`: adult barter path, `equip_item_if_possible` leftover semantics and the baby main-hand swap are all correct.
- `wants_to_pickup`, `can_admire`, `is_wearing_safe_armor`, `get_barter_response_items`, `get_sound_for_activity` and the guarded-container anger path (`on_guarded_block_interacted`) match vanilla ordering.

## New findings (not fixed, need reference or a decision)

| file:line | vanilla | suggested fix | size |
|---|---|---|---|
| piglin.rs:501 nearest-item scan | `PiglinSpecificSensor` only considers `ItemEntity`s with `!hasPickUpDelay()`, so a piglin does not instantly re-seek the item it just threw | add `item.get_pickup_delay() == 0` (or the entity's delay accessor) to the candidate filter | S |
| piglin.rs:926 default equipment | vanilla `Piglin.populateDefaultEquipmentSlots` is `random < 0.5 ? CROSSBOW : GOLDEN_SWORD`; the `1/10 GOLDEN_SPEAR` branch here is not in any decompiled version I can confirm | verify against the current vanilla version (spears are new data); if unsupported, drop the spear branch | S (verify) |
| piglin.rs:262 `expiring_memories` | vanilla persists the whole `Brain.memories` map; Pumpkin whitelists 3 memories and drops e.g. `NEAREST_REPELLENT` | acceptable MVP approximation, list as a known difference unless a bug report needs it | S |
