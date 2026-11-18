# Tracker: entities / Item

Reviewed: `crates/pumpkin-core/src/entity/item.rs` at branch `kab/tentities-item-0024a5`
(re-review after the earlier `kab/tentities-item` review; every row re-checked against the
current file).
Vanilla reference: `net.minecraft.world.entity.item.ItemEntity` (1.21.x), `Player.drop`,
`Player.take`, `Containers.dropContents`, `BundleItem.onDestroyed`. No decompiled tree is
checked out in this worktree (`.vendor/` is absent), so the ordering claims below are marked as
needing a decompiled confirmation.

## Fixed in this review

| Gap | Change |
|---|---|
| A merged never-pickup stack lost the flag. Vanilla `PickupDelay` is an `int`, so `Math.max` keeps `32767` across a merge; Pumpkin's `u8` field saturates at `255` and `never_pickup` was not copied, so `write_custom_nbt` saved `PickupDelay = 255` (which decodes back as a 255-tick delay, not the sentinel). | `target.never_pickup` is now set from the source in `try_merge_with` (`item.rs:383`). |

## Remaining gaps

### 1. Despawn runs before the merge check - size S, needs decompiled confirmation

`item.rs:551` (`process_age_and_merge`): the `age >= LIFETIME` branch discards and returns `false`
before `merge_with_neighbours` (`item.rs:583`). If vanilla `ItemEntity.tick` merges neighbours
before the `age >= 6000` discard, the last tick of a 6000-tick item can still merge into a
neighbour in vanilla but never in Pumpkin. Fix: run the merge block first, then the discard.
Confirm the order in decompiled `ItemEntity.tick` before changing it.

### 2. `is_mergeable` keeps an age guard vanilla does not have - size S

`item.rs:232`. Vanilla `isMergable` is `isAlive() && getCount() < getMaxStackSize()`. The
`item_age >= LIFETIME` guard is unreachable today (the merge only runs when
`process_age_and_merge` returned `true`), so this is dead code, not an observable divergence.
Remove it together with gap 1.

### 3. Persisted `Health` tag is Pumpkin-only - size S

`item.rs:916` writes `Health` and `item.rs:948` reads it back. Vanilla `ItemEntity` has an
in-memory `health = 5` field and `addAdditionalSaveData` stores only `Item`, `Age`,
`PickupDelay`, `Thrower`/`Owner`; health resets to 5 on load. Saving the extra tag makes chunk
NBT differ from vanilla byte-for-byte (harmless to vanilla readers). Fix: drop the write and the
read.

### 4. Owner and Thrower are collapsed into one UUID - size M, needs decompiled confirmation

`item.rs:773` (`on_player_collision`) gates only when the thrower resolves to a player
(`get_player_by_uuid`), so an allay/fox-thrown item is collectable immediately. Vanilla keeps a
separate `owner` (used by `playerTouch`) from `thrower`, and writes both `Owner` and `Thrower` to
NBT (`item.rs:918`). Confirm the 1.21 field split and the exact `playerTouch` condition before
porting.

## Reviewed and matching

Velocity/gravity 0.04, water and lava drag with buoyancy, `noPhysics` push-out, every-4th-tick
resting movement, block friction and half-speed bounce, merge radius 0.5 with rate 2/40 and the
64 cap, atomic reservation plus `ItemMergeEvent` cancel, larger stack becomes the merge target,
max pickup delay and min age on merge (vanilla `ItemEntity.merge` sets
`target.age = Math.min(target.age, source.age)`; the earlier review flagged this as invented, it
is vanilla), Bedrock `UpdateStackSize` resync, `can_be_hurt_by` fire immunity and integer item
health with `mobGriefing`, container and bundle spill on destroy with the default pickup delay,
advancement `ThrownItemPickedUpByPlayer` and the `PickedUp` statistic, Java item metadata on
spawn, `Item`/`Age`/`PickupDelay`/`Thrower` NBT round trip, pickup sound, thrower pickup
protection and the `-32768`/`32767` sentinels.

Known intentional deviation: creative players destroy a stack that fits nowhere
(`item.rs:810`, upstream #4383); vanilla `Inventory.add` leaves it on the ground.

## Gap status
- 1 despawn before merge check (S): RESOLVED (decompile verified; already merged before age/despawn in code; see Result)
- 2 is_mergeable age guard (S): RESOLVED (no age guard remains; matches vanilla; see Result)
- 3 Pumpkin-only Health NBT tag (S): RESOLVED (vanilla persists Health; doc premise wrong; no change; see Result)
- 4 owner and thrower collapsed (M): RESOLVED (already ported in main checkout: separate target/thrower + Owner NBT + call sites + test; see recheck Result)

## Result

Verified against .vendor/vanilla-decompiled/net/minecraft/world/entity/item/ItemEntity.java
(Paper/Folia 1.21.x fork behind the symlink).

- 1 despawn before merge check (S): RESOLVED, no change needed. Decompiled ItemEntity.tick
  runs mergeWithNeighbours() (line 203) before age++ (line 206) and the
  "age >= despawnRate" discard (line 215), so a final-tick item can still merge. The current
  process_age_and_merge (item.rs:549) already does exactly this: merge, then age, then the
  despawn check, returning false only after merge/age. The comment already records the vanilla
  ordering. Left as-is.
- 2 is_mergeable age guard (S): RESOLVED, no change needed. Current is_mergeable
  (item.rs:230) checks only !removed and count < maxStackSize, matching vanilla
  "isAlive() && getCount() < getMaxStackSize()". No item_age >= LIFETIME guard remains.
- 3 Pumpkin-only Health NBT tag (S): RESOLVED, no change needed; the doc premise was wrong.
  Decompiled addAdditionalSaveData (line 385) writes putShort("Health", ...), and
  readAdditionalSaveData (line 402) reads getShortOr("Health", 5). Vanilla DOES persist Health,
  so Pumpkin's write (item.rs:916) and read (item.rs:949) are correct. Nothing dropped.
- 4 owner and thrower collapsed (M): STILL OPEN (out of unit). Decompiled source confirms a
  real field split: thrower is an EntityReference written as "Thrower" and used by getOwner,
  while target is a UUID written as "Owner" (line 389) and is the sole owner gate in
  playerTouch (target == null || target == player.getUUID(), line 490) and in tryToMerge
  (Objects.equals(this.target, other.target), line 284). A faithful port needs a new owner
  field plus Owner NBT and pickup/merge checks in item.rs, AND call sites that set the owner
  (GiveCommand.setTarget(player.getUUID()), player drops) which live outside item.rs.
  Replacing the existing thrower-based 60-tick head start also changes behaviour for freshly
  thrown items, so this cannot be completed within this unit's file scope. No code change made.

No .rs file was modified, so no pure unit test was added (no new seam introduced) and rustfmt
was not required.

## Result (recheck 2026-11-18, current main checkout)

- 4 owner and thrower collapsed (M): RESOLVED, already ported in the main checkout; no change needed.
  Rechecked .vendor/vanilla-decompiled/net/minecraft/world/entity/item/ItemEntity.java: `thrower` is
  an `EntityReference` written as "Thrower" and read by getOwner (lines 53/94/388), `target` is a
  UUID written as "Owner" (lines 54/389/405), playerTouch gates on
  `pickupDelay == 0 && (target == null || target.equals(player.getUUID()))` (line 490), and
  tryToMerge gates on `Objects.equals(this.target, other.target)` (line 284).
  Current item.rs already carries the split: separate `thrower` and `target` fields (item.rs:41-43),
  `set_target`/`get_target`/`target_allows_pickup` (item.rs:187-198), pickup gate (item.rs:822),
  merge gate (item.rs:315), both "Thrower" and "Owner" NBT (item.rs:963-967, 972-973), and the
  thrower-only 60-tick head start kept separately (item.rs:827-836). Call sites are also in place:
  give.rs:67 calls `dropped.set_target(...)`, player.rs:6991 and the mob drops (dolphin/allay/fox/
  brain utils) call `set_thrower(...)`. A pure unit test already covers the pickup gate
  (`target_gate_matches_vanilla_player_touch`, item.rs:1116). Nothing left to fix in this unit.

No .rs file was modified this pass (only this doc), so no new test or rustfmt run was needed.

## Result (give.rs pass 2026-11-18, current main checkout)

Checked `give.rs` against
`.vendor/vanilla-decompiled/net/minecraft/server/commands/GiveCommand.java` (73-118) and
`LivingEntity.createItemStackToDrop` (4257-4299), `ItemEntity.setNoPickUpDelay`/`makeFakeItem`
(561-588).

- 4 follow-up, the `Owner` target was set but not the matching no-delay part: vanilla giveItem's
  not-added branch does `drop.setNoPickUpDelay(); drop.setTarget(player.getUUID())`, so the player
  the command was run for can collect at once while everyone else cannot. Pumpkin only called
  `set_target`, leaving `pickup_delay` at the 40 from `drop_item`, so the giving player had to wait
  ~2 s before collecting. Fixed in give.rs: `dropped.set_pickup_delay(0)` before `set_target`.
- Still open, out of this unit's file scope (item.rs): `on_player_collision` keeps an age-based
  thrower gate (`item_age <= 60` blocks every player except the thrower, item.rs:826-838). Vanilla
  `playerTouch` gates only on `pickupDelay` and `target`; the thrower reference is not a pickup
  gate. Needs confirmation of the 1.21 behaviour before it is removed, and it is in item.rs.
- Still open, client-only: the `added && copyToDrop.isEmpty()` branch spawns a fake item entity
  (`makeFakeItem`: never pickable, `age = despawnRate - 1`) purely so the client animates the
  pickup. Pumpkin has no such visual entity; it needs a `make_fake_item` helper in item.rs plus a
  spawn here, so it is left for an item.rs unit.

No pure unit test: testing the pickup delay needs a spawned ItemEntity and a world, no seam exists
in a command executor.

`rustfmt --edition 2024 crates/pumpkin-core/src/command/commands/give.rs` was NOT run (no shell or
cargo access in this worker; the edit is a single added line inside an existing block).


## Result (give.rs recheck 2026-11-18)

Re-verified the two rows left STILL OPEN by the previous give.rs pass against
.vendor/vanilla-decompiled/net/minecraft/world/entity/item/ItemEntity.java and
server/commands/GiveCommand.java. No .rs source change was possible in this unit.

- Age-based thrower gate (item.rs:826-838): STILL OPEN, out of unit. Decompiled
  playerTouch (ItemEntity.java:490) gates only on
  `pickupDelay == 0 && (target == null || target.equals(player.getUUID()))`;
  the thrower/owner reference is not a pickup gate, so the age <= 60 block in
  on_player_collision is a real divergence. Removal lives in item.rs, not give.rs.
- Fake item entity on the `added && copyToDrop.isEmpty()` branch: STILL OPEN,
  out of unit and client-only. Vanilla giveItem (GiveCommand.java:82-100) spawns
  a makeFakeItem() entity (setNeverPickUp + age = despawnRate - 1,
  ItemEntity.java:585), plays ITEM_PICKUP and broadcasts container changes.
  Pumpkin has no make_fake_item helper / never_pickup setter in item.rs; wiring
  it needs item.rs changes. Per project rule client-only effects do not block,
  so this is left for an item.rs unit.

In-scope fix from the previous pass is present and correct: give.rs:68-69 calls
set_pickup_delay(0) before set_target, matching vanilla setNoPickUpDelay +
setTarget (GiveCommand.java:104-105). No new give.rs change needed.

No pure unit test: both remaining rows are item.rs-only seams; a command
executor has no testable seam here. No rustfmt needed (no .rs change).

## Result (give.rs recheck 2026-11-18, second pass)

Re-verified the in-scope fix and re-confirmed both remaining STILL OPEN rows against
.vendor/vanilla-decompiled/net/minecraft/server/commands/GiveCommand.java (73-118).

- In-scope fix present and correct: give.rs:68-69 calls `set_pickup_delay(0)` then
  `set_target(target.gameprofile.id)` before dropping, matching vanilla's
  `drop.setNoPickUpDelay(); drop.setTarget(player.getUUID())` (GiveCommand.java:104-105)
  in the `else` (did-not-all-fit) branch. No change needed.
- Age-based thrower gate (item.rs:826-838): STILL OPEN, out of unit. Removal of the
  `item_age <= 60` block in `on_player_collision` lives in item.rs, not give.rs.
- Fake item entity on the `added && copyToDrop.isEmpty()` branch: STILL OPEN, out of
  unit and client-only. Vanilla spawns a `makeFakeItem()` (GiveCommand.java:83-87). item.rs
  exposes no public `set_never_pickup`/`make_fake_item` helper (`never_pickup` is a private
  AtomicBool), so this cannot be wired from give.rs alone; it needs item.rs changes. Per
  project rule client-only effects do not block.

No .rs file was modified this pass, so no new pure unit test or rustfmt run was needed
(the only in-scope change is already present from the earlier pass).
