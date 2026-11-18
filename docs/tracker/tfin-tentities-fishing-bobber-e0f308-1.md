# Final rows from docs/tracker/tentities-fishing-bobber-e0f308.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: fishing_bobber.rs

| `fishing_bobber.rs:456-484` | `FishingHook.tick:260-269` runs `move`, `applyEffectsFromBlocks`, `updateRotation`, `reapplyPosition` | No `updateRotation` (the bobber keeps its launch rotation for the whole flight) and no `applyEffectsFromBlocks` (fire/lava/piston effects on the hook skipped) | Set yaw/pitch from the post-move velocity; call the entity block-effects helper | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `fishing_bobber.rs:549-559` (`discard`) | `FishingHook.preRemove:636-641` and `onClientRemoval:644-647` clear `owner.fishing` for every removal path | Only tick and reel-in discards clear `player.fishing_bobber`; a bobber removed by chunk unload or `/kill` leaves a stale id that blocks the next cast | Clear the owner id from an entity-removal hook | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `fishing_bobber.rs:139-290` (`reel_in`) | `FishingHook.retrieve:521-527,609-613` re-reads `hookedIn` after the event and requires `canSimulateMovement` for the pull | The hooked entity captured before the event is pulled even if a plugin unhooks/removes it, and the pull ignores `canSimulateMovement` | Re-check the hooked entity after the event; gate the velocity add | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `fishing_bobber.rs:139-290` (`reel_in`) | `FishingHook.retrieve:524,537` triggers `FISHING_ROD_HOOKED` on the entity path with an empty item list and once for the item list | Only the per-item `FishedItem` trigger fires, so pulling an entity never advances the advancement | Fire the empty-list trigger on the entity path, move the item list trigger out of the loop | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
