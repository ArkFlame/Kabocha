# blocks / Sculk Shrieker - audit

Sources: `crates/pumpkin-core/src/block/blocks/sculk/sculk_shrieker.rs`,
`crates/pumpkin-core/src/block/entities/sculk_shrieker.rs`,
`crates/pumpkin-core/src/world/game_event.rs`, `crates/pumpkin-core/src/entity/mob/warden_spawn_tracker.rs`.
Reference: vanilla `SculkShriekerBlock`, `SculkShriekerBlockEntity` (incl. inner
`VibrationUser`), `WardenSpawnTracker`, `WardenEntity#addDarknessToClosePlayers`.
No decompiled tree is present in this worktree, so the comparisons below rely on
the vanilla classes from memory and `docs/VANILLA_REF.md`.

## Fixed in this audit
- `SculkShriekerBlock::try_shriek` emitted `GameEvent.SHRIEK` with no context
  (`blocks/sculk/sculk_shrieker.rs:112`). Vanilla calls
  `level.gameEvent(GameEvent.SHRIEK, pos, GameEvent.Context.of(player))`, so
  vibration listeners and the plugin `GenericGameEvent` must see the triggering
  player. Now uses `emit_game_event_with_context` with the player as
  `source_entity` (`sculk_shrieker.rs:114-121`).

## Already correct (checked, no change)
- Game-event fan-out to block entities exists: `world/game_event.rs:99-130`
  collects block entities with a listener radius and calls
  `BlockEntity::on_game_event` (landed in e1ed99c1b, issue #3107).
- Listener radius 8 (`GAME_EVENT_RADIUS`), `SHRIEK` self-suppression, player-only
  source, warning cap 4, 200t trigger cooldown, 12000t decay, nearby-player
  merging, 48-block warden exclusion box, 16-block player search
  (`warden_spawn_tracker.rs:10-14,37-67,80-99`).
- 90t shrieking state plus scheduled tick, `can_summon`/difficulty/gamerule gate,
  20 spawn attempts in 11x13x11 with `OnTopOfCollider`, warning-level reply
  sounds at a random point within 10 blocks, Darkness radius 40 and 260t
  (`blocks/sculk/sculk_shrieker.rs:30-181`, `entity/mob/warden.rs:38-40,288-314`).
- `warning_level` NBT save/load and chunk data (`entities/sculk_shrieker.rs:64-86`),
  player-placed `can_summon=false` via the default state in `on_place`
  (`blocks/sculk/sculk_shrieker.rs:186-191`), warning level not reset on summon
  (matches `VANILLA_REF.md`).

## Remaining gaps
| # | Location | Vanilla | Gap | Size |
|---|---|---|---|---|
| 1 | `blocks/sculk/sculk_shrieker.rs:132-141` | `SculkShriekerBlockEntity#tryShriek` applies `WardenEntity.addDarknessToClosePlayers(level, Vec3.atCenterOf(pos), player, 40)` immediately when the shriek starts | Darkness is applied in `respond` instead, i.e. 90 ticks later on the scheduled tick. In-game a single triggered shriek darkens the screen at once. Move the `apply_darkness_around` call into `try_shriek` after the state change; re-verify the exact radius/entity argument against decompiled source | S |
| 2 | `blocks/sculk/sculk_shrieker.rs:164-182` / `entities/sculk_shrieker.rs:88-96` | Breaking a SHRIEKING shrieker only runs `SculkShriekerBlockEntity#trySummonWarden` (`preRemoveSideEffects`) | `on_block_replaced` calls `respond`, so a broken shrieker can also play the nearby reply sound and apply Darkness. Split out a summon-only path for block removal | S |
| 3 | `entities/sculk_shrieker.rs:47-58` | `SculkShriekerBlockEntity#getPlayer` also resolves `Projectile#getOwner()` to a player (arrow/trident vibrations) | Only direct player and first passenger are resolved. Dead code today: no projectile step/impact path emits a game event with a source entity (`emit_game_event_with_source` has no projectile caller), so add it together with projectile vibration emission | M |
| 4 | `entity/mob/warden_spawn_tracker.rs:42-48` | `WardenSpawnTracker#tryWarn` only counts non-spectator, alive nearby players | The trigger player is pushed into the list unconditionally, so a spectator/dead trigger could raise the warning when it is not already among the 16-block candidates. Filter the fallback with the same predicate | S |
| 5 | `world/game_event.rs:114-117` | `VibrationSystem` compares listener distance against the actual event position | Distance is measured to the floored block position, so the acceptance sphere shifts up to one block at the border. Low impact (one-block edge case) | S |
| 6 | (tests) | vanilla behaviour is observable through step/shriek state | No automated test covers a game event reaching a shrieker and setting SHRIEKING (only stepping is covered indirectly). Add a `pumpkin-gametest` case: place can_summon=false shrieker, emit a player-sourced vibration within 8 blocks, assert no shriek; with can_summon=true assert SHRIEKING and the 90t tick | M |

## h6 gap status
- Row 1 (darkness at shriek start): FIXED 483c3018a (`apply_darkness_around` moved into `try_shriek`, removed from `respond`).
- Row 2 (broken shrieker reply and darkness): FIXED 483c3018a (block removal calls new `summon_warden_on_removal`, summon only).
- Row 4 (trigger player bypasses spectator/alive filter in `try_warn`): FIXED 483c3018a.
- Row 5 (vibration distance to floored position): STILL OPEN, needs `VibrationSystem` source check; not changed.
