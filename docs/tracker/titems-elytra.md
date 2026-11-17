# items / Elytra (re-review 2026-11-16)

Source changed since the last review, so every elytra path was re-audited against vanilla
`net.minecraft.world.entity.player.Player`, `LivingEntity#updateFallFlying` and
`FireworkRocketEntity`. No decompiled `.vendor/` copy exists in this worktree; class names below
are the vanilla ones, line numbers are Pumpkin's.

## Matches vanilla (no action)

- Item data `assets/items.json` (`elytra`): `minecraft:glider`, `minecraft:equippable` chest with
  `equip_elytra` sound and `damage_on_hurt: false`, `minecraft:max_damage` 432,
  `minecraft:repairable` phantom membrane, rarity epic. No `minecraft:enchantable`, same as vanilla
  (elytra take books through an anvil, not the enchanting table).
- `Player#canGlide`: `crates/pumpkin-core/src/entity/player.rs:3826` - flying, on-ground, passenger,
  levitation, `canGlideUsing`.
- `canGlideUsing`: `player.rs:3807` (`glider_slots`) - glider component, equippable slot match,
  `!nextDamageWillBreak`. Main hand is iterated explicitly because `build_equipment_slots`
  (`crates/pumpkin-inventory/src/lib.rs:98`) holds only armour + off hand, so no duplicate.
- `updateFallFlying` / `checkFallDistanceAccumulation`: `player.rs:3869` - stop when `!canGlide`,
  `fallFlyTicks` counter, `ELYTRA_GLIDE` game event every 10 ticks, one random glider damaged per 20.
- Wall crash (`(speed - speed) * 10 - 3` into `FLY_INTO_WALL`): `player.rs:3903`.
- Start / stop entry points: `net/java/play/player_command.rs:31`, `entity/player/client_move.rs:351`,
  `net/bedrock/play/player_auth_input.rs:152` and `:165`.
- Firework boost: `entity/projectile/firework_rocket.rs:252` - look angle `0.1 / 1.5 / 0.5` formula,
  launch sound on `life == 0`, lifetime `10 * (1 + flight) + rnd(6) + rnd(7)`.
- `aviate_one_cm`: `player.rs:4430` plus `client_move.rs:336`.
- Right-click equip is generic (`net/java/play/use_item.rs:118`, chest is `HumanoidArmor` and
  swappable), so elytra needs no `ItemBehaviour`, matching vanilla.

## Fixed in this pass

1. `try_to_start_fall_flying` rejected the start in any liquid; vanilla `tryToStartFallFlying` only
   checks `isInWater()`. `player.rs:3838`.
2. The Java `StartFlyingElytra` path also carries the client's request to *stop* gliding, and it
   called `Entity::stop_fall_flying` directly, bypassing the `EntityToggleGlideEvent` veto honoured
   by the tick, landing and Bedrock stop sites. `net/java/play/player_command.rs:31`.

## Remaining gaps

| # | file:line | vanilla class | gap | suggested fix | size |
|---|---|---|---|---|---|
| 1 | `entity/player.rs:3877` | `LivingEntity#checkFallDistanceAccumulation` | Rust only lowers `fall_distance` to 1.0 when it is already above 1.0; vanilla assigns 1.0 unconditionally while `delta.y > -0.5`, which can also raise a smaller value | store 1.0 unconditionally inside the branch, after confirming against the decompiled method (observable only below the 3.0 fall-damage threshold) | S |
| 2 | `entity/player.rs:3976` | `Player#causeFoodExhaustion` | `progress_motion` still has `TODO: gliding...`; vanilla grants no exhaustion while fall flying, so the TODO looks stale | confirm and delete the TODO, or implement whatever vanilla behaviour it meant | S |
| 3 | `entity/player.rs:3869` | `LivingEntity#updateFallFlying` | no automated test covers glide stop (plugin veto) or the start gate; the veto change above is untested | add a `pumpkin-gametest` that starts a glide, cancels `EntityToggleGlideEvent` and asserts the flag stays | M |
| 4 | `entity/player.rs:2173` | `ItemStack#hurtAndBreak` | elytra wear goes through `damage_item_in_slot`; Unbreaking / durability-enchantment application inside `damage_item_stack` was not verified without a decompiled reference | compare `damage_item_stack` against `ItemStack#hurtAndBreak` before relying on it | M |
