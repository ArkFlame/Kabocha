# Verification: upstream PRs #3826 (ice creative) and #3978 (gliding fall distance)

Both PRs were marked SKIPPED "already on master" in the ledger. Each was verified
against its upstream diff and the decompiled vanilla source. Both behaviours are
already present in this checkout, so no code changes were needed. Upstream paths
(`crates/pumpkin/src/...`) map to ours under `crates/pumpkin-core/src/...`.

## PR #3826 - ice must not melt/spawn water when broken by a creative player

Upstream diff (`/tmp/kabocha-runs/prs/3826.diff`) adds an early creative guard at
the top of `IceBlock::broken`.

Present on master: `crates/pumpkin-core/src/block/blocks/ice.rs:71-74`:

```rust
fn broken(&self, args: BrokenArgs<'_>) {
    if args.player.gamemode.load() == GameMode::Creative {
        return;
    }
    // ... silk-touch / water-evaporates / below-block checks unchanged
```

`gamemode.load() == GameMode::Creative` is equivalent to `player.is_creative()`
(`crates/pumpkin-core/src/entity/player.rs:8487-8489`). The remaining `broken` logic
matches vanilla `IceBlock.afterDestroy` (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/IceBlock.java:43-56`)
and vanilla `playerDestroy` (IceBlock.java:30-42) only reaches the melt via
`afterDestroy` on a real destroy, which creative breaking does not trigger.

Verdict: already correct. No fix.

## PR #3978 - cap fall distance while gliding

Upstream diff (`/tmp/kabocha-runs/prs/3978.diff`) adds, in `LivingEntity::fall`, a
call to a new `check_fall_distance_accumulation(height_difference)` when
`is_fall_flying()`; the helper caps `fall_distance` at 1.0 when `velocity_y > -0.5`.

Present on master in a different but equivalent location:
`crates/pumpkin-core/src/entity/player.rs:4072-4080`, inside `tick_fall_flying()`:

```rust
if !entity.is_fall_flying() {
    self.fall_fly_ticks.store(0, Ordering::Relaxed);
    return;
}
// Vanilla `checkFallDistanceAccumulation`: only a steep dive builds it up. ...
if entity.movement.load().y > -0.5 && self.living_entity.fall_distance.load() > 1.0 {
    self.living_entity.fall_distance.store(1.0);
}
```

`tick_fall_flying()` runs every player tick (`player.rs:3799`). This matches vanilla
`LivingEntity.updateFallFlying()` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/LivingEntity.java:3911-3912`),
which calls `this.checkFallDistanceAccumulation()`. That method
(`.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java:4215-4219`)
caps `fallDistance` to 1.0 when `getDeltaMovement().y() > -0.5`. Our check is gated
on `is_fall_flying()` (the PR's stated intent, gliding), uses the delta movement y
(closer to vanilla than the upstream diff's `height_difference`), and runs each tick.

Verdict: already correct. No fix.

## Result

- fixed = 0 (both behaviours already present on master)
- verified = 2 (#3826, #3978)
- No code changes; investigation doc only. No cargo run per the shared-checkout worker contract.
