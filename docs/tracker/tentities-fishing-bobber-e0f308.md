# entities / Fishing Bobber (RE-REVIEW)

Target: `JavaMinecraftVersion::V_26_3`. Reference: decompiled Paper/Canvas tree in
`/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft`
(`world/entity/projectile/FishingHook.java`, `world/item/FishingRodItem.java`); `.vendor` absent.
Source reviewed: `crates/pumpkin-core/src/entity/projectile/fishing_bobber.rs`,
`crates/pumpkin-core/src/item/items/fishing_rod.rs`.

Reviewed source change: `97beb89a3` (bobber implementation) and `4ff9297c3` (previous
knowledge-only audit). A decompiled reference is available now, so the previous audit is
superseded: four of its gaps (spawn 0.1 below the eye, one XP orb per catch, the
`FAILED_ATTEMPT` fire point, `catchingFish` on dry ticks) were wrong and are listed as matching.

## Fixed in this pass

- `fishing_bobber.rs:203-212` - a cancelled `CAUGHT_FISH` event must abort the whole retrieve
  (`FishingHook.retrieve:539-546` returns 0), not only skip that item.
- `fishing_bobber.rs:253-278` - the `IN_GROUND` and `REEL_IN` `PlayerFishEvent` states were never
  fired (`FishingHook.retrieve:570-586`); both now fire and a cancel returns 0 as vanilla does.
- `fishing_rod.rs:80` - the cast awarded `ITEM_USED` twice: the generic Java use handler
  (`net/java/play/use_item.rs:40`) already awards it, and `FishingRodItem:64` awards it once.

## Verified matching (decompiled)

- Throw setup: origin `(-sin*0.3, eyeY, -cos*0.3)`, `clamp(-(sin/cos), -5, 5)`, `0.6/len` plus a
  per-axis `triangle(0.5, 0.0103365)`, rotation derived from the movement (`FishingHook:100-125`).
- Tick order: 1200-tick ground life, water probe, `FLYING -> BOBBING` velocity `*(0.3, 0.2, 0.3)`,
  `HOOKED_IN_ENTITY` at `getY(0.8)`, `outOfWaterTime` -1 in water / +1 dry clamped to 10, bob force
  with 0.9 damping, gravity 0.03 while dry and airborne, inertia 0.92 (`FishingHook:167-271`).
  Vanilla never discards on `outOfWaterTime`; neither does this code.
- `checkCollision` raycast, `onHitBlock` normalise-and-shorten, `setHookedEntity` syncs id + 1,
  entity filter incl. `ItemEntity` (`FishingHook:293-320`).
- `retrieve`: entity pull `delta * 0.1`, damage 3 for items / 5 for entities, ground override 2,
  loot params (origin, tool, this entity, `luck + owner.getLuck()`), per-item launch velocity,
  per-item XP orb at `player.y + 0.5, z + 0.5` gated on `expToDrop > 0`, `FISHES` stat
  (`FishingHook:505-593`).
- `catchingFish`: rain +1, sky -1, `nibble` `20..=40`, `timeUntilHooked` `20..=80`, `timeUntilLured`
  `100..=600 - lure`, `triangle(0, 9.188)` drift, tease table and particles,
  `FISHING_BOBBER_SPLASH` at 0.25 volume, bite kick from the `uuid ^ gameTime` seeded random
  (`FishingHook:322-433`, `144-158`). `FAILED_ATTEMPT` on nibble expiry and no event on a lure tick
  match; `catchingFish` is water-only in vanilla too.
- `shouldStopFishing`: rod in either hand, `distanceToSqr <= 1024`, alive (`FishingHook:273-291`).

## Remaining gaps

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| `world/loot/predicates.rs:363-379` | `FishingHook.calculateOpenWater:442-489` plus the `openWater` update at `233-237`: four 5x5 layers (air/lily pad = above water, source water with empty collision shape = inside water, anything else invalid), gated by `outOfWaterTime < 10` | `in_open_water` tests only the block at the hook, so a 1x1 hole against a wall rolls treasure | Track `openWater` on the bobber as vanilla does (reset when `nibble <= 0 && timeUntilHooked <= 0`, else AND the scan) and read it from the predicate | M |
| `entity/type.rs:143` (`from_type`) | `EntityType.FISHING_BOBBER` deserialises to `FishingHook`; `tick` discards when `getPlayerOwner()` is null | No `FISHING_BOBBER` arm: a `/summon`ed or chunk-loaded bobber is a plain `Entity` that never ticks or despawns | Add an arm rebuilding `FishingBobberEntity` from `ProjectileOwner` NBT (needs a `from_loaded` constructor) or discard it on load | M |
| `fishing_bobber.rs:399-429` | `FishingHook.catchingFish:415-429` fires `PlayerFishEvent.State.LURED` when `timeUntilLured` reaches 0 and zeroes `timeUntilHooked` when cancelled | `PlayerFishState` has no `Lured`, so plugins cannot veto a lure | Add `PlayerFishState::Lured` (native + WIT, additive) and fire it before rolling `hook_countdown` | M |
| `fishing_bobber.rs:448-460` | `FishingHook.tick:260-269` runs `move`, `applyEffectsFromBlocks`, `updateRotation`, `reapplyPosition` | No `updateRotation` (bobber keeps its launch rotation for the whole flight) and no `applyEffectsFromBlocks` (fire/lava/piston effects on the hook skipped) | Set yaw/pitch from the post-move velocity; call the entity effects helper | S |
| `fishing_bobber.rs:436` | `FishingHook.tick:256-258` applies gravity when `!fluidState.is(WATER)`, independent of the resolved height | Uses `in_water` (height > 0), so a water block with height 0 still gets gravity | Test `fluid.matches_type(&Fluid::WATER)` instead | S |
| `fishing_bobber.rs:536` (`catching_fish`, wait reset) | Paper `resetTimeUntilLured:436-439` clamps lure to `timeUntilLured - 1` so a datapack lure speed at or above the roll cannot loop instantly | `range(100..=600) - wait_time_reduction_ticks` can reach 0 or below | Apply the Paper clamp | S |
| `fishing_bobber.rs:529-534` (`discard`) | `FishingHook.discard()` clears `owner.fishing` for every removal path | Only tick and `reel_in` discards clear `player.fishing_bobber`; a bobber removed by chunk unload or `/kill` leaves a stale id that blocks the cast branch until the next right-click | Clear the owner id from an entity-removal hook | S |
| `net/java/play/use_item.rs:40` vs `fishing_rod.rs` | `FishingRodItem:64` awards `ITEM_USED` on cast only | The generic handler also awards it on reel-in | Skip the generic award while reeling (`fishing_bobber != -1`) or move it into behaviours | S |

Note: no test added. The changed paths are plugin-event ordering with no existing harness and
`docs/WORKER.md` forbids running cargo in this pass.
