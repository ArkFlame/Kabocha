# blocks/Vault (RE-REVIEW)

Reviewed after b7f9d7cce (vault placement uses block properties):
crates/pumpkin-core/src/block/blocks/vault.rs, crates/pumpkin-core/src/block/entities/vault.rs.
No decompiled vanilla source in .vendor; comparisons are against VaultBlock / VaultBlockEntity (1.21).

## Fixed in this pass
- Rewarded players were never serialised: VaultBlockEntity only wrote the raw server_data compound it had
  loaded, so a chunk reload or restart emptied the rewarded set and every player could loot the same vault
  again. entities/vault.rs now reads and writes server_data.rewarded_players as four-int UUID arrays
  (vanilla VaultBlockEntity.ServerData#rewardedPlayers).
- BlockDispenseLootEvent (plugin/api/events/block/block_dispense_loot.rs:11) was declared but never fired.
  blocks/vault.rs now dispatches it once with the generated loot before dropping, and skips the drops when
  a plugin cancels it.

## Remaining gaps
1. No ejecting/display state machine. blocks/vault.rs:96-146 sets EJECTING, drops every stack and returns to
   ACTIVE in the same tick. Vanilla VaultBlock#tick drives UNLOCKING -> EJECTING -> ACTIVE over ~40 ticks
   (shutter open, then items ejected one at a time), with server_data.stateUpdatingResumesAt persisted.
   Fix: add a scheduled tick to VaultBlock plus that timestamp field. Size M.
2. Loot is generated at insertion and never stored, so a reload mid-eject loses (or, with 1, duplicates)
   rewards. Vanilla keeps VaultServerData.itemsToEject in server_data. Fix: generate on activation, store the
   list, eject from it. Size M.
3. Display item missing: shared_data.display_item is never written, so the item the vault shows while
   unlocking never appears and VaultDisplayItemEvent (plugin/api/events/block/vault_display_item.rs:11) stays
   unfired. Needs the state machine from gap 1. Size S/M.
4. Reward drop shape: vanilla spawns an ItemEntity at the vault centre with pickup delay and a small upward
   velocity; blocks/vault.rs:141 uses World::drop_stack(position, ..). Size S.
5. Ominous activation: blocks/vault.rs:54 and 115 only read the ominous property; nothing ever flips it.
   Vanilla converts a vault to ominous for a nearby player with Trial Omen. Verify the entry point in the
   decompiled VaultBlock/TrialSpawner before porting. Size M, verify first.
6. Check ordering: ours validates the key first (blocks/vault.rs:53-67) and only then the rewarded set;
   vanilla VaultBlock#useItemOn tests isRewarded first and plays VAULT_REJECT_REWARDED_PLAYER. Size S,
   verify against decompiled source.
7. Client-only: shutter animation, VAULT_AMBIENT loop, VAULT_ACTIVATING / VAULT_DEACTIVATE and ejection
   particles are not driven server-side; no gameplay blocker.
