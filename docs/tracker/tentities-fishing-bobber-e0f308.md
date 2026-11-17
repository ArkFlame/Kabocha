# entities / Fishing Bobber (RE-REVIEW)

Target: `JavaMinecraftVersion::V_26_3`. Reference: decompiled Paper/Canvas tree in
`/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft`
(`world/entity/projectile/FishingHook.java`, `world/item/FishingRodItem.java`); `.vendor` absent.
Source reviewed: `crates/pumpkin-core/src/entity/projectile/fishing_bobber.rs`,
`crates/pumpkin-core/src/item/items/fishing_rod.rs` at `8daaf141b`.

Source changed since the last audit (`ea1097aaa`, `577c8f12d`), so the whole tick and retrieve
paths were re-checked against the decompiled reference.

## Fixed in this pass

- `fishing_bobber.rs:70-100` - spawned 0.1 below the owner eye; vanilla `FishingHook:109-112`
  `snapTo(x, player.getEyeY(), z, ...)` uses the eye height exactly.
- `fishing_bobber.rs:70-120` - luck bonus was clamped to 1; vanilla `FishingHook:88` is
  `Math.max(0, luck)`, so an unenchanted rod must not add a luck point to the fishing loot.
- `fishing_bobber.rs:729-741` - wait roll could reach 0 or below with a high Lure speed and then
  loop forever; vanilla Paper `resetTimeUntilLured:436-439` leaves one tick when the lure is at
  or above the maximum wait. Both the constructor roll and `catching_fish` use the helper now.
- `fishing_bobber.rs:345-456` - a bobbing hook was discarded once `outOfWaterTime` reached 10.
  Vanilla `FishingHook.tick:250-252` only clamps the field; it never removes the hook, it only
  loses open-water status.
- `fishing_bobber.rs:378-383,456` - gravity was gated on the resolved water height; vanilla
  `FishingHook.tick:256` gates on `fluidState.is(FluidTags.WATER)`, which also covers water
  blocks with zero height.

## Verified matching (decompiled)

- Throw setup: origin offset `-sin*0.3 / -cos*0.3`, `clamp(-(sin/cos), -5, 5)`, `0.6/len` plus a
  per-axis `triangle(0.5, 0.0103365)`, rotation from the movement (`FishingHook:100-125`).
- Tick order: 1200-tick ground life, water probe, `FLYING -> BOBBING` velocity `*(0.3,0.2,0.3)`,
  `HOOKED_IN_ENTITY` at `getY(0.8)`, `outOfWaterTime` -1 water / +1 dry clamped to 10, bob force
  with 0.9 damping, gravity only while dry and airborne, inertia 0.92 (`FishingHook:167-271`).
- `checkCollision` raycast, `onHitBlock` normalize-and-shorten, `setHookedEntity` id + 1,
  entity filter incl. `ItemEntity` (`FishingHook:293-320`).
- `retrieve`: entity pull `delta * 0.1`, damage 3 items / 5 entities, ground override 2, loot
  params (origin, tool, this entity, `luck + owner.getLuck()`), per-item velocity, per-item XP
  orb at `x, y + 0.5, z + 0.5` (`ExperienceOrbEntity::spawn` no-ops at 0), `FISHES` stat,
  `CAUGHT_FISH` cancellation aborting the catch, `IN_GROUND` and `REEL_IN` states
  (`FishingHook:505-593`).
- `catchingFish`: rain +1, sky -1, `nibble` 20..=40, `timeUntilHooked` 20..=80, tease table,
  particles, splash at 0.25 volume, bite kick from `uuid ^ gameTime`, `FAILED_ATTEMPT` on nibble
  expiry, bite `BITE` cancel leaves the window closed (`FishingHook:322-433`, `144-158`).
- `shouldStopFishing`: rod in either hand, `distanceToSqr <= 1024`, alive (`FishingHook:273-291`).
- Cast/retrieve sounds, pitch `0.4 / (rand*0.4 + 0.8)`, throw 0.5 and retrieve 1.0
  (`FishingRodItem:52-57,24-31`).

## Remaining gaps

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| `world/loot/predicates.rs:363-380` | `FishingHook.calculateOpenWater:442-489` plus the `openWater` update at `233-237` | `in_open_water` tests only the block at the hook, so a 1x1 hole against a wall rolls treasure | Track `openWater` on the bobber (reset when `nibble <= 0 && timeUntilHooked <= 0`, else AND a four-layer 5x5 scan) and read it from the predicate | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `entity/type.rs:143` (`from_type`) | `EntityType.FISHING_BOBBER` deserialises to `FishingHook`; `tick:170-172` discards a hook with no player owner | No `FISHING_BOBBER` arm: a `/summon`ed or chunk-loaded bobber is a plain `Entity` that never ticks or despawns | Rebuild `FishingBobberEntity` from the projectile-owner NBT (needs a `from_loaded` constructor) or discard it on load | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `fishing_bobber.rs:688-724` | `FishingHook.catchingFish:415-428` fires `PlayerFishEvent.State.LURED` when `timeUntilLured` reaches 0 and zeroes `timeUntilHooked` when cancelled | `PlayerFishState` has no `Lured`, so plugins cannot veto a lure | Add `PlayerFishState::Lured` (native + WIT, additive) and fire it where `hook_countdown` is rolled | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `item/items/fishing_rod.rs:54-88` | `FishingRodItem:38-44` fires `PlayerFishEvent.State.FISHING` after constructing the hook and skips the cast when cancelled | `PlayerFishState::Fishing` exists but is never fired, so plugins cannot block a cast | Fire it before the throw sound and bail out on cancel | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `fishing_bobber.rs:456-484` | `FishingHook.tick:260-269` runs `move`, `applyEffectsFromBlocks`, `updateRotation`, `reapplyPosition` | No `updateRotation` (the bobber keeps its launch rotation for the whole flight) and no `applyEffectsFromBlocks` (fire/lava/piston effects on the hook skipped) | Set yaw/pitch from the post-move velocity; call the entity block-effects helper | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `fishing_bobber.rs:549-559` (`discard`) | `FishingHook.preRemove:636-641` and `onClientRemoval:644-647` clear `owner.fishing` for every removal path | Only tick and reel-in discards clear `player.fishing_bobber`; a bobber removed by chunk unload or `/kill` leaves a stale id that blocks the next cast | Clear the owner id from an entity-removal hook | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `fishing_bobber.rs:139-290` (`reel_in`) | `FishingHook.retrieve:521-527,609-613` re-reads `hookedIn` after the event and requires `canSimulateMovement` for the pull | The hooked entity captured before the event is pulled even if a plugin unhooks/removes it, and the pull ignores `canSimulateMovement` | Re-check the hooked entity after the event; gate the velocity add | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `fishing_bobber.rs:139-290` (`reel_in`) | `FishingHook.retrieve:524,537` triggers `FISHING_ROD_HOOKED` on the entity path with an empty item list and once for the item list | Only the per-item `FishedItem` trigger fires, so pulling an entity never advances the advancement | Fire the empty-list trigger on the entity path, move the item list trigger out of the loop | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |

Note: no test added; `docs/WORKER.md` forbids running cargo in this pass, and the changed paths
are tick-order and event-ordering fixes with no existing harness.

## Result

Verified against `.../mc-proof/ref/net/minecraft/world/entity/projectile/FishingHook.java` and
`.../mc-proof/ref/net/minecraft/world/item/FishingRodItem.java`.

- `world/loot/predicates.rs:389-437` (open water): fixed the loot predicate to follow vanilla
  `FishingHook.calculateOpenWater`: each 5x5 layer must be uniform, air may only sit above water and
  the bottom layer must be water; lily pads count as air. Added the pure
  `water_above_air_is_not_open_water` unit test. Per-tick `openWater` caching on the bobber is not
  added; the predicate now applies the same layer rule at loot time instead.
- `entity/type.rs:143` (bobber arm): STILL OPEN - needs a `from_loaded` constructor and owner-NBT
  rehydration; the plain `Entity` fallback never reaches `process_tick`'s no-owner discard.
- `fishing_bobber.rs` (Lured event state): STILL OPEN - needs an additive `PlayerFishState::Lured`
  in native + WIT + host, outside this unit's files.
- `item/items/fishing_rod.rs` (Fishing event on cast): STILL OPEN - the event is already fired and
  cancellable in out-of-unit `net/java/play/use_item.rs:192`; passing the real hook UUID requires
  editing that file.
- `fishing_bobber.rs:473-478` (rotation + block effects): fixed - call `Entity::check_block_collision`
  and `update_rotation` after the move, matching `FishingHook.tick:260-269`.
- `fishing_bobber.rs:847-857` (removal clears owner): fixed - `on_removed()` clears
  `player.fishing_bobber` on every removal path, matching vanilla `preRemove`/`onClientRemoval`.
- `fishing_bobber.rs:161-199` (reel_in re-check): fixed - re-reads the hooked entity after the
  `CaughtEntity` event and requires `!is_removed()` before pulling; also fires the entity-path
  empty-list `FISHING_ROD_HOOKED` advancement (vanilla `retrieve:521-527`).
- Per-item `FishedItem` trigger: left as-is - `AdvancementTrigger::FishedItem` carries a single item
  id, so a single full-list trigger needs a trigger API change outside this unit; firing once per
  item is equivalent for the `fishing_rod_hooked` predicate.
