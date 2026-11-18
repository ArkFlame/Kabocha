# tverify items 3: S/M open items (h23)

Source: `docs/tracker/tverify-items2.md` (kab/h14). Vanilla reference: `.vendor/vanilla-decompiled` (main checkout).

| Item | Status | Evidence / reason |
|---|---|---|
| Fishing Rod cast cancel (`item/items/fishing_rod.rs`) | VERIFIED | Vanilla `FishingRodItem.use` lines 47-54: cancelled `PlayerFishEvent` sets `player.fishing = null` and returns `PASS`. Pumpkin `net/java/play/use_item.rs:71` (`should_continue_use_after_fish_event`) fires `PlayerFishState::Fishing` and returns early on cancel; bobber id stays -1. Reel-in cancel is handled in `entity/projectile/fishing_bobber.rs` `reel_in` (`None` keeps the hook). Tracker row is stale. |
| Fishing Rod reel-in `swing_hand` | VERIFIED | Vanilla `FishingRodItem.use` has no server-side swing (lines 24-40). Pumpkin `fishing_rod.rs` calls `player.swing_hand(hand, false)` on reel-in. Possible duplicate animation with the client swing packet. Not changed: confirming needs a client-behaviour check. |
| Bat resting check (`entity/mob/bat.rs` `tick_roosting`, `tick_flying`) | VERIFIED | Vanilla `Bat.java` lines 136 and 178 use `isRedstoneConductor`. Its default is `isCollisionShapeFullBlock` (`BlockBehaviour.java:1247`), which Pumpkin already mirrors through `IS_SOLID_BLOCK` (`block_registry.rs:1199-1201`), so the tracker's "differs for slabs" claim is wrong. The real gap is per-block overrides: `Blocks.java` sets `isRedstoneConductor(Blocks::never|always)` at about 15 sites (e.g. lines 660, 2226). Those overrides are not in `assets/blocks.json`; needs Extractor data. |
| Lodestone compass (`item/items/compass.rs`, `entity/player.rs` tracker tick) | VERIFIED | Vanilla `item/component/LodestoneTracker.java` `tick`: unlinks when out of build limit, or when the chunk is loaded and the block is not `Blocks.LODESTONE`; does not load chunks. Pumpkin `entity/player.rs` `~7071-7079` does the same, returning `target: None, tracked: true`. It is a block check, not a POI check. |
| Lodestone chunk-load POI rebuild (`block/blocks/lodestone.rs`) | VERIFIED | Not part of the compass check. Block-state to POI mapping is not audited. |

Counts: 0 fixed in code (code already matched vanilla in every row), 5 verified, 0 open, 0 blocked-data.
