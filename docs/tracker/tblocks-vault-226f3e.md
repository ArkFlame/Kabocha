# blocks/Vault (RE-REVIEW)

Source reviewed at 89fe0bf0b ("fix(block): persist rewarded vault players and fire vault loot event"):
crates/pumpkin-core/src/block/blocks/vault.rs, crates/pumpkin-core/src/block/entities/vault.rs.
That commit is what invalidated the previous assessment. No decompiled vanilla in `.vendor`; comparison is
against VaultBlock / VaultBlockEntity (1.21) plus docs/VANILLA_REF.md ("128-player history, unlock 14t,
eject 20t apart, 2-5 stacks").

## Fixed in this pass
- Rewarded set was unbounded (`FxHashSet<Uuid>`) and unordered, so `server_data.rewarded_players` grew without
  limit and could not evict oldest entries. Vanilla `VaultServerData` keeps an insertion-ordered set capped at
  `MAX_REWARD_PLAYERS = 128`, dropping the oldest. entities/vault.rs:14 now stores an ordered `Vec<Uuid>`,
  :19 caps it, :165 evicts the front before inserting, and read/write preserve order and de-duplicate.

## Previously fixed (kept for context)
- 89fe0bf0b: rewarded players are serialised as four-int UUID arrays in `server_data.rewarded_players`.
- 89fe0bf0b: `BlockDispenseLootEvent` fires once before the loot drop.

## Remaining gaps
1. No ejecting/display state machine. blocks/vault.rs:96-146 sets EJECTING, drops every stack and returns to
   ACTIVE in the same tick. Vanilla VaultBlock#tick drives UNLOCKING -> EJECTING -> ACTIVE over ~40 ticks.
   Fix: scheduled tick + `server_data.stateUpdatingResumesAt`. Size M.
2. Loot is generated at insertion and never stored (`server_data.itemsToEject` missing), so a reload mid-eject
   loses rewards. Fix alongside gap 1. Size M.
3. `shared_data.display_item` is never written, so VaultDisplayItemEvent
   (plugin/api/events/block/vault_display_item.rs:11) never fires. Needs gap 1. Size S/M.
4. Non-key items: blocks/vault.rs:53-67 treats any non-matching item as a key failure and plays
   BlockVaultInsertItemFail + returns SuccessServer. Vanilla validates `ItemTags.TRIAL_KEYS` first and returns
   TRY_WITH_EMPTY_HAND (no sound, no consume) for other items, only playing the fail sound for a trial key that
   does not match the vault's ominous flag. Verify against decompiled VaultBlock before changing. Size S, verify.
5. Check ordering: ours validates the key before the rewarded set (blocks/vault.rs:54-80); vanilla
   `tryInsertKey` tests the rewarded set first and plays VAULT_REJECT_REWARDED_PLAYER. Size S, verify.
6. Ominous activation: blocks/vault.rs:54,115 only read the property; nothing flips it. Vanilla converts a vault
   for a nearby player with Trial Omen. Verify entry point in VaultBlock/TrialSpawner. Size M, verify.
7. Ejection shape: blocks/vault.rs:145 uses `World::drop_stack` (random offset inside the block). Vanilla
   spawns at `pos.getCenter() + (0, 0.4, 0)` with a small upward velocity and pickup delay. Part of gap 1. Size S.
8. Client-only: shutter animation, VAULT_AMBIENT loop, VAULT_ACTIVATING/DEACTIVATE and particles are not driven
   server-side; no gameplay blocker.
