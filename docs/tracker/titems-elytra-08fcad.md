# items / Elytra (re-review 2026-11-16, slug titems-elytra-08fcad)

Tracker row `items/Elytra` was flagged RE-REVIEW ("reviewed source has changed"). Re-audited
every elytra path against vanilla `Player#canGlide` / `#tryToStartFallFlying` /
`LivingEntity#updateFallFlying` / `#checkFallDistanceAccumulation` /
`#handleFallFlyingCollisions` and `FireworkRocketEntity`. No decompiled `.vendor/` copy
exists in this worktree, so class names are vanilla and line numbers are Pumpkin's.

## What actually changed since the previous review (docs/tracker/titems-elytra.md, 5a422185e)

- `crates/pumpkin-core/src/entity/player.rs` last changed by `010cac8ee fix(items): check water
  only when starting elytra flight`, i.e. the previous review's own fix; nothing after it
  (`git log 5a422185e..HEAD -- crates/pumpkin-core/src/entity ...` shows only unrelated
  dragon/bedrock/fishing slices).
- `assets/items.json` and `crates/pumpkin-data/src/data_component_impl/utility.rs` unchanged
  since 5a422185e, so the item data and the skin/profile `elytra` field are untouched.
- Both previous fixes are present: water-only start gate `player.rs:3838-3841`; the Java
  `StartFlyingElytra` stop path goes through `glide_stop_allowed`
  (`net/java/play/player_command.rs:34-38`).

Conclusion: the RE-REVIEW flag is stale for this row; the code already contains the review's
fixes. Only the gaps below remain, and none is small *and* certain enough to change blindly.

## Verified this pass (no action)

- Item data `assets/items.json` `elytra`: `minecraft:glider`, `minecraft:equippable`
  chest with `minecraft:item.armor.equip_elytra`, `damage_on_hurt: false`,
  `minecraft:max_damage` 432, repairable phantom membrane, rarity epic, no
  `minecraft:enchantable`. Generated `item.rs:38995` matches.
- `can_glide` `player.rs:3826` and `glider_slots` `player.rs:3807`: not flying, not on
  ground, no vehicle, no levitation, glider component + matching equippable slot +
  `!next_damage_will_break()`; main hand iterated once.
- `tick_fall_flying` `player.rs:3870`: stop when `!can_glide` behind the plugin veto,
  `ELYTRA_GLIDE` game event every 10 ticks, one random glider damaged per 20.
- Wall crash `player.rs:3904`: `(last - new) * 10 - 3` into `FLY_INTO_WALL`.
- Firework boost `entity/projectile/firework_rocket.rs:271-283`: `vel + look*0.1 +
  (look*1.5 - vel)*0.5` on the client-reported motion, plugin `PlayerElytraBoostEvent`
  veto honoured, launch sound on `life == 0`.
- Creative/spectator guard on exhaustion: `add_exhaustion` `player.rs:5732` returns while
  `abilities.invulnerable`, matching `Player#causeFoodExhaustion`.

## Remaining gaps

| # | file:line | vanilla class | gap | suggested fix | size |
|---|---|---|---|---|---|
| 1 | `entity/player.rs:3877` | `LivingEntity#checkFallDistanceAccumulation` | Rust lowers `fall_distance` to 1.0 only when it is already above 1.0; if vanilla assigns 1.0 unconditionally while `delta.y > -0.5`, a smaller value is never raised | confirm against a decompiled `LivingEntity`; if unconditional, store 1.0 in the branch (observable only below the 3.0 fall-damage threshold) | S |
| 2 | `entity/player.rs:3977` | `Player` movement withdrawal (exhaustion) | `progress_motion` still carries `TODO: gliding...`; the swim / in-water / climb / on-ground chain already falls through to `return` while gliding, so nothing looks implementable, but the original intent was never checked against the decompiled method | confirm vanilla adds no gliding exhaustion, then delete the stale TODO on the next touch | S |
| 3 | `entity/player.rs:3869` | `LivingEntity#updateFallFlying` | no automated test covers glide stop (plugin veto) or the start gate, including the water-only fix from 5a422185e | add a `pumpkin-gametest`: start a glide, cancel `EntityToggleGlideEvent`, assert the flag stays; same for landing | M |
| 4 | `entity/player.rs:2173` | `ItemStack#hurtAndBreak` | elytra wear goes through `damage_item_in_slot` -> `damage_item_stack`; Unbreaking and durability-enchantment handling there was not verified without a decompiled reference | compare `damage_item_stack` with `ItemStack#hurtAndBreak` (Unbreaking `1/(level+1)`, 1 durability per call) before relying on it | M |
