# world / Explosion

Audit, 2026-11-16, branch kab/taudit-crates-pumpkin-core-src-world-expl. File:
`crates/pumpkin-core/src/world/explosion.rs` (1023 lines, no TODO/FIXME markers; the three
"vanilla" hits are doc comments at :119, :631, :779). No `.vendor/` vanilla tree is present in
this worktree, so every claim below is against `ServerExplosion`/`Explosion` knowledge and the
call sites in this repo; items marked *verify* need a decompiled-source check before editing.

## Source map

| File | Class it ports |
|---|---|
| crates/pumpkin-core/src/world/explosion.rs | ServerExplosion, Explosion, ExplosionDamageCalculator, SimpleExplosionDamageCalculator |
| crates/pumpkin-core/src/world/mod.rs:4609-4726 | Level.explode / ServerLevel.explode + ClientboundExplodePacket fan-out |
| crates/pumpkin-core/src/plugin/api/events/block/block_explode.rs | Paper BlockExplodeEvent |

## Verified matching vanilla

- Ray tracing: 16^3 shell directions (skip interior), `dir/7.5-1` normalisation, 0.3 step,
  `h = power * (0.7 + rand*0.6)`, `h -= (resistance + 0.3) * 0.3` then `h -= 0.22500001`, break
  outside build limit (explosion.rs:421-529).
- `getBlockExplosionResistance`: air + empty fluid -> None, else `max(block, fluid)` (explosion.rs:57-70).
- Fire roll: one roll per affected position, `roll == 0 && state.isAir() && below.isSolidRender()`,
  `BaseFireBlock.getState`, `setBlockAndUpdate` flags (explosion.rs:376-397, 751-757).
- Entity phase: radius `power*2`, AABB floor(center +- radius +- 1), skip immune/spectator/direct
  source, `distance > 1.0` skip, exposure sampling grid + `ClipContext(Collider, Fluid.NONE)`,
  damage `((e*e + e)/2 * 7 * radius) + 1`, creative-flying knockback skip, TNT uses feet position
  for the knockback direction, `explosion_knockback_resistance` attribute, Java players receive the
  impulse in the packet instead of a server velocity (explosion.rs:536-658, 680-738).
- Block phase: `BlockInteraction` keep/trigger/destroy/decay mapping, `should_drop_items_on_explosion`,
  `explosion_radius` loot parameter, block `explode` hook, container-screen close, air replace with
  NOTIFY_ALL (explosion.rs:751-846).
- Paper yield argument `1.0 / power` on BlockExplodeEvent (explosion.rs:786-790).

## Fixed in this audit

- explosion.rs:742-762: explosions never announced `GameEvent::Explode`. Vanilla
  `ServerExplosion#explode` calls `level.gameEvent(source, GameEvent.EXPLODE, center)` first, which
  drives sculk sensors / vibration listeners. Now emitted through
  `World::emit_game_event_with_context` with the direct source, before any damage or block change.

## Remaining gaps

| # | Location | Vanilla / Paper behaviour | Suggested fix | Size |
|---|---|---|---|---|
| 1 | explosion.rs:781-799 (and world/mod.rs:4695-4708, block_explode.rs:10-14) | Paper builds `BlockExplodeEvent`/`EntityExplodeEvent` with the affected block list and replaces the to-blow list with `event.blockList()`; `yield` also scales drops | Add a block list (and mutable yield) to both events, fire after `get_blocks_to_destroy`, filter `blocks` by the returned list, apply yield to drops | M |
| 2 | explosion.rs:451-470 | `level.getBlockState` loads/forces the chunk, so explosions still destroy blocks in unloaded areas; here `read_chunk_sync` returning `None` is treated as air | Fall back to `world.get_block_state` (which loads) or force-load the chunk before sampling | M |
| 3 | explosion.rs:506-515 | `Entity#getBlockExplosionResistance` is applied on top of the calculator value even when a custom calculator is set (wind charge, wither skull); here the hook is skipped whenever `damage_calculator.is_some()` | Apply `source.get_block_explosion_resistance` to any `Some(resistance)`; *verify* against the target version's `calculateExplodedPositions` first | S |
| 4 | explosion.rs:441, 744 | Ray power and fire rolls come from the level `RandomSource`, so a seed replays explosions deterministically; here `rand::random` / `rand::rng()` use thread RNG | Draw from a world-owned seeded RNG (like `loot_random`) | S-M |
| 5 | explosion.rs:339-343 | `ServerExplosion#isSmall` is `radius < 2.0` in common mappings; the extra `Keep` clause makes a mobGriefing-off creeper (power 3) use the small particle | Drop the `Keep` clause if the target version has no such rule; *verify*. Client-visual only | S |
| 6 | explosion.rs:843, world/mod.rs:4717 | `ClientboundExplodePacket.blockCount` in vanilla is the destroyed-block count fed to the client's smoke particles; confirm affected-vs-destroyed | Confirm and align; *verify*. Client-visual only | S |
| 7 | explosion.rs:308-318 | `player_explosion` damage type is used when the credited entity is a player; here any living indirect source (e.g. wither skull -> wither) selects it | Gate on the indirect source being a player; *verify* `DamageSources#explosion` | S |
| 8 | world/mod.rs:4711-4725 (not this file) | Vanilla `finalizeExplosion` emits its own particle/sound; here the packet is the only source, sent to players within 64 blocks | Confirm the radius and that every caller sets the right particle/sound; client-only | S |
