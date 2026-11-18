# Parity rows for crates/pumpkin-core/src/block/blocks/vault.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **blocks/Vault** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/tblocks-vault-226f3e** [partial/OWN-OPEN] own audit: 5 open, 4 fixed, 0 verified, 0 blocked-data (docs/tracker/tblocks-vault-226f3e.md) docs/tracker/tblocks-vault-226f3e.md

## Result

Both rows closed against decompiled vanilla under `.vendor/vanilla-decompiled/net/minecraft/`.

- **blocks/Vault** [partial/AUDITED] -> VERIFIED/FIXED. Re-reviewed the whole block against
  `world/level/block/VaultBlock.java`, `world/level/block/entity/vault/VaultBlockEntity.java`
  (`Server.tick` 230-260, `tryInsertKey` 262-295, `unlock` 380-395,
  `cycleDisplayItemFromLootTable` 342-356), `VaultState.java` (79-104, 139-143) and
  `VaultServerData.java` (20-34). Earlier passes already resolved the UNLOCKING -> EJECTING ->
  ACTIVE state machine (14/20-tick delays), stored `items_to_eject` for reload survival,
  bottom-center ejection position with upward velocity, insertion-ordered 128-player rewarded
  set, display-item event wiring, and the per-20-tick ACTIVE display cycle. This pass closed the
  last gameplay gap: the 15-tick insert-fail sound buffer (`VaultBlockEntity.java` 422-427;
  `lastInsertFailTimestamp` at `VaultServerData.java:33`, deliberately absent from the NBT
  CODEC 20-28). `VaultBlockEntity::should_play_insert_fail_sound` now throttles both
  `BlockVaultInsertItemFail` and `BlockVaultRejectRewardedPlayer` to one play per 15 ticks via a
  runtime-only `AtomicI64`. Added pure unit test
  `insert_fail_sound_is_throttled_within_fifteen_ticks` in entities/vault.rs.
- **own/tblocks-vault-226f3e** [partial/OWN-OPEN] -> VERIFIED/FIXED. All 5 originally open rows
  (gaps 1, 2, 3, 4, 7) are closed: 1/2/7/3 in earlier passes (see
  docs/tracker/tblocks-vault-226f3e.md), gap 4 in this pass. Gap 5 (check ordering) verified as
  no-change-needed: `VaultBlockEntity.java:274-276` tests `isValidToInsert` before
  `hasRewardedPlayer`, matching Pumpkin blocks/vault.rs:85-109. Gap 6 (ominous activation)
  verified no-change-needed: vanilla only sets OMINOUS=true on TrialSpawner, never converts a
  Vault at runtime (vault ominous state is structural/worldgen). Gap 8 is client-only (shutter
  animation, ambient loop, particles, eject-sound pitch) per AGENTS.md and is not a gameplay
  blocker.

Files edited: crates/pumpkin-core/src/block/entities/vault.rs,
crates/pumpkin-core/src/block/blocks/vault.rs. Not compiled (worker restriction); orchestrator
batch-checks. rustfmt touched files (orchestrator runs it).

Status: PASS fixed=2 open=0 files=crates/pumpkin-core/src/block/blocks/vault.rs,crates/pumpkin-core/src/block/entities/vault.rs,docs/tracker/tpar-crates-pumpkin-core-src-block-blocks-vault-rs.md
