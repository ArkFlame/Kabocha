# Final rows from docs/tracker/tentities-fishing-bobber-e0f308.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: fishing_bobber.rs fishing_rod.rs predicates.rs type.rs

| `world/loot/predicates.rs:363-380` | `FishingHook.calculateOpenWater:442-489` plus the `openWater` update at `233-237` | `in_open_water` tests only the block at the hook, so a 1x1 hole against a wall rolls treasure | Track `openWater` on the bobber (reset when `nibble <= 0 && timeUntilHooked <= 0`, else AND a four-layer 5x5 scan) and read it from the predicate | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `entity/type.rs:143` (`from_type`) | `EntityType.FISHING_BOBBER` deserialises to `FishingHook`; `tick:170-172` discards a hook with no player owner | No `FISHING_BOBBER` arm: a `/summon`ed or chunk-loaded bobber is a plain `Entity` that never ticks or despawns | Rebuild `FishingBobberEntity` from the projectile-owner NBT (needs a `from_loaded` constructor) or discard it on load | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `fishing_bobber.rs:688-724` | `FishingHook.catchingFish:415-428` fires `PlayerFishEvent.State.LURED` when `timeUntilLured` reaches 0 and zeroes `timeUntilHooked` when cancelled | `PlayerFishState` has no `Lured`, so plugins cannot veto a lure | Add `PlayerFishState::Lured` (native + WIT, additive) and fire it where `hook_countdown` is rolled | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `item/items/fishing_rod.rs:54-88` | `FishingRodItem:38-44` fires `PlayerFishEvent.State.FISHING` after constructing the hook and skips the cast when cancelled | `PlayerFishState::Fishing` exists but is never fired, so plugins cannot block a cast | Fire it before the throw sound and bail out on cancel | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
