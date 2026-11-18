# Final rows from docs/tracker/tblocks-vault-226f3e.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- Gaps 3, 5, 7 (display item, rewarded-set ordering, eject shape) STILL OPEN - need decompiled VaultBlock to verify; not changed.
- Gap 3 remainder (periodic 20t ACTIVE display cycle): STILL OPEN - client-only visual (gap 8)
- Gap 6 (ominous activation): STILL OPEN - vanilla converts a vault for nearby players with the
Re-verified the two remaining `STILL OPEN` rows against decompiled vanilla

## Result
FINAL PASS. Verified against decompiled vanilla under `.vendor/vanilla-decompiled/net/minecraft/`.

- Row 1a, Gap 3 (display item): FIXED. Vanilla `VaultSharedData#setDisplayItem` (`world/level/block/entity/vault/VaultSharedData.java:48`) and `VaultBlockEntity.Server#unlock` (`VaultBlockEntity.java:392` `sharedData.setDisplayItem(serverData.getNextItemToEject())`). Pumpkin `entities/vault.rs::set_display_item` writes/removes the `ItemStack` under `shared_data.display_item`, sent to clients by `chunk_data_nbt`; `blocks/vault.rs::use_with_item` fires `VaultDisplayItemEvent`, honours `cancelled()`, and stores the resulting item.
- Row 1b, Gap 5 (check ordering): VERIFIED, no change needed. Vanilla `VaultBlockEntity.Server.tryInsertKey` tests `isValidToInsert` (`VaultBlockEntity.java:274`) before `hasRewardedPlayer` (:276). Pumpkin `blocks/vault.rs` validates the key (:85) before the rewarded set (:99), matching order. The doc's claim that vanilla checks the rewarded set first is false.
- Row 1c, Gap 7 (eject shape): FIXED. Vanilla `VaultState#ejectResultItem` (`world/level/block/entity/vault/VaultState.java:139-140`) = `DefaultDispenseItemBehavior.spawnItem(level, stack, 2, Direction.UP, Vec3.atBottomCenterOf(pos).relative(Direction.UP, 1.2))`. Pumpkin `blocks/vault.rs::ejection_spawn_position` = (x+0.5, y+1.2, z+0.5) with upward velocity ~0.6 plus small jitter and pickup delay 10; pure test `ejection_spawns_at_bottom_center_raised_to_vanilla_height`.
- Row 2, Gap 3 remainder (periodic 20t ACTIVE display cycle): FIXED. Vanilla `shouldCycleDisplayItem` (`VaultBlockEntity.java:418-420`: `gameTime % 20 == 0 && state == ACTIVE`) and `cycleDisplayItemFromLootTable` (:342-356). Pumpkin `blocks/vault.rs::on_scheduled_tick` ACTIVE branch calls `cycle_active_display_item` (random loot stack, `VaultDisplayItemEvent`, honours cancelled, writes `shared_data.display_item`); the ticker chain starts in `placed()` and reschedules in `finish_ejection`. Tests `random_display_item_is_empty_for_empty_loot` and `random_display_item_returns_a_generated_stack`. Though the floating item is a client visual, the server-side behaviour is implemented, so no gameplay blocker remains.
- Row 3, Gap 6 (ominous activation): VERIFIED, no change needed (premise false). Vanilla `VaultBlock.java` only declares/defaults `OMINOUS` (:31, :35, :72), with no runtime write or vault-conversion path. The only runtime `OMINOUS = true` write is `TrialSpawner.applyOminous` (`world/level/block/entity/trialspawner/TrialSpawner.java:104-106`), which sets `TrialSpawnerBlock.OMINOUS` at the spawner position, never a vault. Vault ominous state is structural (worldgen), so there is no vanilla runtime behaviour to port here; conversion would also need files outside this unit.

All rows closed: no BLOCKED-DATA (no missing generated field). Source files `crates/pumpkin-core/src/block/blocks/vault.rs` and `crates/pumpkin-core/src/block/entities/vault.rs` already contain the fixes from earlier passes; no new source edits were needed this pass. NOT compiled (worker restriction); orchestrator batch-checks.
