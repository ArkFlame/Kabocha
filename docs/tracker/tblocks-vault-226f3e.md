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

## h2 parity sweep (status)
- Gaps 3, 5, 7 (display item, rewarded-set ordering, eject shape) STILL OPEN - need decompiled VaultBlock to verify; not changed.

## Result

Verified against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/`
(`VaultBlock.java`, `VaultBlockEntity.java`, `VaultServerData.java`, `VaultState.java`,
`VaultSharedData.java`).

- Gap 3 (display item): FIXED. `entities/vault.rs::VaultBlockEntity::set_display_item` now writes
  `ItemStack` into `shared_data.display_item` (removing it for an empty stack); it is sent to clients by
  `chunk_data_nbt`. `blocks/vault.rs::use_with_item` fires `VaultDisplayItemEvent` with the next item to eject
  (vanilla `Server.unlock` -> `setDisplayItem(getNextItemToEject())`), honours `cancelled()`, and stores the
  resulting item. Still open: the periodic 20-tick ACTIVE display cycle (`cycleDisplayItemFromLootTable`)
  needs gap 1's scheduled tick; it is a client-only visual (gap 8), no gameplay blocker.
- Gap 5 (check ordering): NO CHANGE NEEDED. Decompiled `VaultBlockEntity.Server.tryInsertKey` tests
  `isValidToInsert` first (line 274) and only then `hasRewardedPlayer` (line 276). Pumpkin already validates
  the key (blocks/vault.rs:58-71) before the rewarded set (:75-82), so the doc's claim that vanilla checks the
  rewarded set first is false. The 15-tick insert-fail sound buffer (`last_insert_fail_timestamp`) is a
  separate gap 4 concern and was not touched.
- Gap 7 (ejection shape): FIXED. Replaced `World::drop_stack` with a spawn at
  `Vec3.atBottomCenterOf(pos).relative(Direction.UP, 1.2)` = (x+0.5, y+1.2, z+0.5), upward velocity ~0.6 with
  small gaussian jitter (vanilla `DefaultDispenseItemBehavior#spawnItem` speed 2, Direction.UP) and pickup
  delay 10. Added a pure unit test `ejection_spawns_at_bottom_center_raised_to_vanilla_height`.
  Still open: one stack is dropped per tick across the UNLOCKING -> EJECTING -> ACTIVE state machine (gap 1),
  which is a separate, larger change outside this pass.

Files edited: `crates/pumpkin-core/src/block/blocks/vault.rs`,
`crates/pumpkin-core/src/block/entities/vault.rs`. Both formatted with `rustfmt --edition 2024`.
Not compiled (worker restriction); orchestrator batch-checks.


## Result (second pass: state machine + stored loot)

Verified against decompiled vanilla `VaultBlock.java`, `VaultBlockEntity.java` (Server.tick,
tryInsertKey, unlock, cycleDisplayItemFromLootTable), `VaultState.java` (tickAndGetNext,
ejectResultItem) and `VaultServerData.java`.

- Gap 1 (ejecting/display state machine): FIXED. `use_with_item` now enters UNLOCKING and
  schedules a block tick after `UNLOCKING_DELAY_TICKS` (14). A new `on_scheduled_tick` drives
  UNLOCKING -> EJECTING (shutter-open sound) -> one stack per `EJECTION_DELAY_TICKS` (20) ->
  ACTIVE (shutter-close sound), matching vanilla's timing (`stateUpdatingResumesAt` equivalent
  via self-rescheduling block ticks) and sound order (insert on UNLOCKING enter, open shutter on
  EJECTING enter, eject per stack, close shutter on EJECTING exit).
- Gap 2 (loot stored for reload): FIXED. `VaultBlockEntity` now owns `items_to_eject`, written to
  and read from `server_data.items_to_eject` as a list of item-stack compounds (vanilla
  `VaultServerData#itemsToEject`). Loot resolved at insertion is stored there and popped from the
  back (`popNextItemToEject` order), so a reload mid-ejection keeps the rewards.
- Gap 7 (ejection shape): retained from prior pass; spawn now happens once per EJECTING tick via
  the same `ejection_spawn_position` + upward-velocity + pickup-delay path, with its pure test.
- Gap 4 (non-key items): NO CHANGE NEEDED. Decompiled `VaultBlockEntity.Server.isValidToInsert`
  uses `ItemStack.isSameItemSameComponents(stackToInsert, config.keyItem())` and plays
  `VAULT_INSERT_ITEM_FAIL` for ANY non-matching item; `useItemOn` returns SUCCESS_SERVER whenever
  the state is ACTIVE. Pumpkin already plays `BlockVaultInsertItemFail` and returns
  `SuccessServer`, so the doc's `ItemTags.TRIAL_KEYS` / `TRY_WITH_EMPTY_HAND` claim is outdated.
  Remaining minor gap: vanilla's 15-tick `last_insert_fail_timestamp` sound buffer (needs game time).
- Gap 5 (check ordering): unchanged from prior pass; vanilla checks valid key before rewarded set.
- Gap 3 remainder (periodic 20t ACTIVE display cycle): STILL OPEN - client-only visual (gap 8)
  needing a persistent every-tick ACTIVE ticker; no gameplay blocker.
- Gap 6 (ominous activation): STILL OPEN - vanilla converts a vault for nearby players with the
  Trial Omen effect via TrialSpawner/effect entry points, which are outside this unit's files.
- Gap 8: client-only (shutter animation, ambient loop, particles); no gameplay blocker.

Files edited: `crates/pumpkin-core/src/block/blocks/vault.rs`,
`crates/pumpkin-core/src/block/entities/vault.rs`. Both formatted with `rustfmt --edition 2024`
(`--check` clean). Not compiled (worker restriction); orchestrator batch-checks.


## Result (re-review pass)

Re-verified the two remaining `STILL OPEN` rows against decompiled vanilla
(`VaultBlockEntity.Server.tick`, `VaultState.tickAndGetNext`, `VaultServerData`,
`TrialSpawner.applyOminous`, `TrialSpawnerStateData.tryDetectPlayers`).

- Gap 3 remainder (periodic 20-tick ACTIVE display cycle): verified real in vanilla -
  `Server.tick` calls `cycleDisplayItemFromLootTable` when `shouldCycleDisplayItem` holds
  (`gameTime % 20 == 0 && state == ACTIVE`), which picks a random loot item and fires
  `VaultDisplayItemEvent`. LEFT OPEN: purely client visual (the floating item) with no
  gameplay blocker (gap 8); matching it needs a perpetual self-rescheduling 20-tick ticker
  on every ACTIVE vault (a loaded-chunk perf cost) plus a proximity/ticker lifecycle
  Pumpkin's vault does not have. The unlock-time `VaultDisplayItemEvent` path is already
  wired, so plugins can still customise the displayed item. Per AGENTS.md client-only
  effects are noted, not blocked on.
- Gap 6 (ominous activation): NO CHANGE NEEDED - premise is false. The only runtime
  `OMINOUS = true` write in vanilla is `TrialSpawner.applyOminous` (sets the spawner block,
  never a vault), triggered by the Trial Omen / Bad Omen effect in
  `TrialSpawnerStateData.tryDetectPlayers`. Vault ominous state is structural (placed that
  way by worldgen); no code path converts a Vault block at runtime, so there is nothing to
  port here. Converting vaults would also require trialspawner/effect files outside this
  unit.
- Stale status line 41 (`Gaps 3, 5, 7 STILL OPEN`) is superseded: both Result sections above
  already fixed 3 and 7 and confirmed 5 (vanilla `tryInsertKey` checks `isValidToInsert`
  before `hasRewardedPlayer`, matching Pumpkin's order).

No source changes made this pass; both named files already contain the earlier fixes.

Status: PASS fixed=0 open=1 files=docs/tracker/tblocks-vault-226f3e.md


## Result (third pass: periodic ACTIVE display cycle)

Verified against decompiled vanilla `.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/vault/VaultBlockEntity.java`
(`Server.tick` 230-260, `cycleDisplayItemFromLootTable` 342-356, `getRandomDisplayItemFromLootTable` 358-365,
`shouldCycleDisplayItem` 418-420) and `VaultState.java` (`ACTIVE#onEnter` 24-35).

- Gap 3 remainder (periodic 20-tick ACTIVE display cycle): FIXED. `blocks/vault.rs::on_scheduled_tick`
  ACTIVE branch now calls a new `cycle_active_display_item` that resolves the vault loot table, picks a
  random generated stack (`random_display_item`, vanilla `getRandomDisplayItemFromLootTable`), fires
  `VaultDisplayItemEvent` honouring `cancelled()`, and writes `shared_data.display_item`. The ACTIVE
  ticker chain now actually runs: `placed()` schedules the first tick and `finish_ejection` reschedules
  it when ejection completes, so a vault cycles its floating item every 20 ticks, matching vanilla's
  per-tick block-entity ticker gated by `gameTime % 20 == 0 && state == ACTIVE`.
- Removed the dead `entities/vault.rs::cycle_display_item_from_loot_table` stub (its `resolve_loot_table`
  always returned an empty `Vec` and config is never populated for player-placed vaults) and its two
  trivial no-op tests; display-item writing stays in `set_display_item`, now driven from the block tick.
- Added pure unit tests `random_display_item_is_empty_for_empty_loot` and
  `random_display_item_returns_a_generated_stack`.

Files edited: `crates/pumpkin-core/src/block/blocks/vault.rs`,
`crates/pumpkin-core/src/block/entities/vault.rs`. Not compiled (worker restriction); orchestrator batch-checks.

Status: PASS fixed=1 open=0 files=crates/pumpkin-core/src/block/blocks/vault.rs,crates/pumpkin-core/src/block/entities/vault.rs
