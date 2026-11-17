# entities / Armadillo — audit after source change

Source: `crates/pumpkin-core/src/entity/passive/armadillo.rs` (+ `entity/type.rs:215`, `entity/mob/spider.rs:50`, `block/blocks/redstone/dispenser.rs:973`).
Vanilla reference class: `net.minecraft.world.entity.animal.armadillo.Armadillo` / `Armadillo.ArmadilloState`.

## Fixed in this pass

- Ambient sound: `Mob::get_ambient_sound` defaults to `None` (`entity/mob/mod.rs:2275`) and living entities never fall back to `EntityType.ambient_sound` (`entity/living.rs:4599` -> `tick_ambient_sound`). Vanilla `Armadillo.getAmbientSound` = `ARMADILLO_AMBIENT`, so the mob was silent except for step/roll/brush. Added the override.
- Hurt sound while rolled up: vanilla `Armadillo.getHurtSound` returns `ARMADILLO_HURT_REDUCED` while `shouldHideInShell(inStateTicks)` is true, otherwise `ARMADILLO_HURT` (generated `entity_type.rs:518` supplies the plain one). `should_hide_in_shell` had no caller at all. Added `CustomSound` (`hurt_sound`/`death_sound`) following `mob/bat.rs:301`; `living.rs:3778` falls back to the `EntityType` sound when `hurt_sound()` is `None`, so only the shell case changes.

## Remaining gaps

1. **No periodic threat scan — armadillo never rolls up on its own.** `is_scared_by` (`armadillo.rs:273`) has zero callers and `SCARE_CHECK_INTERVAL` (`armadillo.rs:34`) is only reused as a danger-counter value (`:373`), never as an interval. Vanilla `Armadillo.customServerAiStep` re-scans every 80 ticks for living entities inside `inflate(7.0, 2.0, 7.0)` and calls `rollUp()`; only the hurt path triggers the shell today.
   Fix: in `custom_server_ai_step`, when state is `Idle` and `danger_detected_recently_ticks == 0` and `tick_count % SCARE_CHECK_INTERVAL == 0`, iterate `world.get_nearby_entities(pos, SCARE_DISTANCE_HORIZONTAL)` (`world/mod.rs:5371`), keep entities with a living entity and `self.is_scared_by(entity)`, then `self.roll_up()`. Needs the tick counter on `Entity`. Size **M**.

2. **`on_damage` conditional is dead.** `armadillo.rs:367-379` stores `SCARE_CHECK_INTERVAL` and then, if `can_stay_rolled_up()`, stores `80` — both are 80, so the branch does nothing. Vanilla sets `dangerDetectedRecently = 80` once when the source entity is not an item entity. Collapse to a single store (and confirm whether vanilla zeroes the counter when `!canStayRolledUp()`). Size **S**.

3. **Damage reduction keyed on the wrong predicate.** `modify_incoming_damage` (`armadillo.rs:359`) halves damage whenever `is_scared()`, i.e. also during `Rolling`/`Unrolling` and while not yet hidden in the shell. Vanilla ties the reduction to the shell (`shouldHideInShell`), same predicate as the reduced hurt sound. Gate on `self.should_hide_in_shell()` and re-derive the exact vanilla formula (`(amount - 1.0) / 2.0` here) before changing the arithmetic. Size **S**.

4. **Client-side shell pose not driven.** `should_hide_in_shell` (`armadillo.rs:191`) still has no server-side effect on the model or hitbox; vanilla's renderer hides the head via the same state ticks. Acceptable as client-only, tracked here so it is not mistaken for a server gap. Size **M** (client/plugin side).

## Verified as matching vanilla (no action)

- State enum ids/ordering (0..3), `animation_duration` (10/50/30), `should_hide_in_shell` thresholds (>5, true, <26), string names for NBT.
- `SCARE_DISTANCE_HORIZONTAL` 7.0 / `SCARE_DISTANCE_VERTICAL` 2.0 and the undead-tag + sprinting/rider player checks in `is_scared_by`.
- `scute_time` init `rand(6000) + 6000`, decrement while alive, gift-loot-table shed gated on `should_drop_loot`, reset only on drop.
- NBT `state` (string, with int fallback) and `scute_time` (`armadillo.rs:339`).
- Brush: baby/dead rejection, `minecraft:brush/armadillo` loot table, `ARMADILLO_BRUSH` sound, `ENTITY_INTERACT` game event, 16 durability (`armadillo.rs:236`, `:457`).
- `mob_interact` passes while scared; food tag `#minecraft:armadillo_food`; `ARMADILLO_BABY_START_AGE` -48000; baby tracked-data resync on growth (`ageable.rs:115`).
- Goals: swim 0, escape-danger 2 (speed 2.0 -> vanilla 1.5 double is the shared helper's choice), breed 2, tempt 3 (1.25, no-scared), follow-parent 4, wander 5, look-at-player 6 (6.0), random-look 7.
- Spider flee target uses `is_scared()` to only flee calm armadillos (`mob/spider.rs:50`).
