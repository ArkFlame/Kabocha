# Parity rows for crates/pumpkin-core/src/item/items/fishing_rod.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **items/Fishing Rod** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.

## Result

THIRD PASS. One row: items/Fishing Rod [partial/AUDITED]. Re-reviewed the whole
file against .vendor/vanilla-decompiled/net/minecraft/world/item/FishingRodItem.java
(use, lines 22-74), InteractionResult.java (11-16, 51-53) and
ServerGamePacketListenerImpl.java (2442-2443), plus FishingHook.java
(constructor 86-113, retrieve 505-593).

- items/Fishing Rod: FIXED (one source change in fishing_rod.rs). Vanilla
  FishingRodItem.use returns InteractionResult.SUCCESS on BOTH the cast and reel
  branches (FishingRodItem.java:73). InteractionResult.SUCCESS is Success(PREDICTED,
  DEFAULT) with shouldSwing() == true (InteractionResult.java:11, 51-53), and the
  server swings the arm on such a result (ServerGamePacketListenerImpl.java:2442-2443,
  sendToSelf = false). Pumpkin's rod leaves ItemBehaviour::use_result at its default
  Pass, so the generic packet handler (net/java/play/use_item.rs:82-87) does not swing;
  the reel branch already called player.swing_hand(hand, false) but the cast branch did
  not, so other players saw no arm swing on a cast. Added player.swing_hand(hand, false)
  at the end of the cast branch, after the ITEM_USED award and ITEM_INTERACT_START event.
  Same established pattern as the reel branch and spear.rs:129. The method already used
  in this file, so no new imports and no clippy impact.

- Verified matching, no change: PlayerFishEvent.State.FISHING fired after hook
  construction and cancellable, returning early with player.fishing = null and no
  throw sound/spawn/stat on cancel (FishingRodItem.java:46-67 and CraftBukkit patches);
  lure/luck reduction and (x*20) as i32 / as i32 casts (FishingRodItem.java:44-45);
  throw/retrieve sound volumes 0.5/1.0 and pitch 0.4/(r*0.4+0.8) (:38, :63);
  ITEM_USED awarded only on the cast branch (:69); ITEM_INTERACT_START/FINISH vibrations
  (:40, :70); stale bobber-id guard and safe downcast before remove, matching
  FishingHook.retrieve ending in discard(); rod damage applied through retrieve's
  1/2/3/5 return values (FishingHook.java:526, 567, 577). The reel-branch swing is kept:
  it supplies the SUCCESS arm swing the generic Pass result omits.

- No unit test added: the interaction decision needs a live Player, world and plugin
  manager, so there is no pure seam; the only pure helper (bobber_sound_pitch) is a
  literal formula mirror, which the MVP-test rules forbid testing.

- No generated pumpkin-data field, WIT or cross-crate change needed. Compilation and
  rustfmt are deferred to the orchestrator batch gate.
