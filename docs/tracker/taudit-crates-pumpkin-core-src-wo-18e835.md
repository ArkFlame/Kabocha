# Tracker: world / player_touch

Reviewed: `crates/pumpkin-core/src/world/player_touch.rs` (52 lines) on branch
`kab/taudit-crates-pumpkin-core-src-wo-18e835`.
Vanilla reference: `net.minecraft.world.entity.player.Player#aiStep` (touch loop,
`getBoundingBox().inflate(1.0, 0.5, 1.0)`), `Entity#getBoundingBox` / passenger union,
`ServerPlayer#pushEntities`, `ExperienceOrb#playerTouch` (`takeXpDelay`),
`AbstractArrow#playerTouch`, `ItemEntity#playerTouch`.

## Markers

No `TODO`, `FIXME`, `XXX` or `HACK` in the file. The two "vanilla" markers counted by the
tracker row are the doc comments at `player_touch.rs:31-33` and none of them describe
unfinished work. Nothing to delete.

## Verified matching

| Behaviour | Code | Vanilla |
|---|---|---|
| Pickup box is the player hitbox expanded by 1.0 / 0.5 / 1.0 | `player_touch.rs:22-29` | `Player#aiStep` inflate(1.0, 0.5, 1.0) |
| Riding a live vehicle widens the box to the player+vehicle union | `player_touch.rs:10-20` | Player hitbox on a mount comes from the vehicle box (ported upstream in #3793, "player touch vehicle") |
| Dead and spectator players do not touch anything | `player_touch.rs:35-40` | `Player#aiStep` runs the loop only for an alive, non-spectator player |
| Abort the scan if a touch callback changed the player's world (portal/respawn) | `player_touch.rs:48-50` | Vanilla touches the level-local list only |
| Already-removed entities are skipped | `player_touch.rs:44` | `Level#getEntities` never returns removed entities |
| One pass per player, after the player ticked and before non-player entities tick | `world/mod.rs:1951-1963` | `ServerPlayer#doTick`: `aiStep` touch, then `pushEntities` |
| XP delay allows one orb per two ticks | `experience_orb.rs:387-397`, decrement at `player.rs:3421-3425` | `takeXpDelay` set to 2, decremented once per `aiStep` |

## Remaining gaps

None that change observable behaviour. Two documented deviations, both unobservable:

1. **Touch runs after `push_entities`, vanilla runs it before (aiStep), size S, no edit recommended.**
   `world/mod.rs:1957-1963` calls `player.push_entities()` for every player and only then
   `touch_entities`. Vanilla `ServerPlayer#doTick` touches entities in `aiStep` and pushes
   afterwards. The only entities whose order could matter are item entities, and the pushed
   item is either picked up and removed later in the same tick (push is discarded with it) or
   blocked by `pickupDelay` and pushed in both orders, so the result is identical. Reordering
   would churn the push-velocity sync from #3793 for no behavioural gain; leave as is.

2. **Other players are not in the scan, vanilla includes them, size S (no-op).**
   `world/mod.rs:5344-5354` (`get_entities_at_box`) only queries non-player entities (plus
   dragon parts). Vanilla `Level#getEntities(this, aabb)` returns other players too, but
   `Player` does not override `playerTouch`, so the extra calls have no effect. No edit needed.

Not verified here because no decompiled 1.21.5 source is vendored (`.vendor/` is absent):
the mounted branch drops the vertical 0.5 inflation (`player_touch.rs:24-25`) while the
union already covers the mount. This came from upstream #3793, which ported the vanilla
vehicle box, so it is treated as parity; a future audit with vanilla source should confirm
`Player#aiStep` uses the vehicle box and not `inflate(1.0, 0.5, 1.0)` on the rider box.
