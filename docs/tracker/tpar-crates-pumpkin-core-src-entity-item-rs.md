# Parity rows for crates/pumpkin-core/src/entity/item.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Item** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/tentities-item** [partial/OWN-OPEN] own audit: 8 open, 5 fixed, 0 verified, 0 blocked-data (docs/tracker/tentities-item.md) docs/tracker/tentities-item.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)

## Result (2026-11-18 third pass - final)

Independently re-verified both rows against `.vendor/vanilla-decompiled/net/minecraft/` and the
current source. No source change warranted: every gap already matches vanilla. No `.rs` file
touched, so no rustfmt run and no build required.

- `entities/Item` (AUDITED): VERIFIED. All 5 gaps in `docs/tracker/tentities-item.md` re-checked
  below against the decompiled vanilla; none needs a fix.
- `own/tentities-item` (OWN-OPEN): VERIFIED. Every previously open row is already correct in
  `item.rs` / `player.rs`; the only editable-file behaviour (`set_thrower`) matches vanilla.

Gap-by-gap (vanilla reference `net/minecraft/world/entity/item/ItemEntity.java`, 1.21.x Paper/Canvas):

- 1 owner pickup protection (M): VERIFIED - `ItemEntity.playerTouch` (ItemEntity.java:490) gates
  only on `pickupDelay == 0 && (target == null || target.equals(player.getUUID()))`. Pumpkin
  `on_player_collision` (item.rs:822-834) checks `pickup_delay > 0 || never_pickup` and
  `target_allows_pickup` (item.rs:195-198: `target.is_none_or(|owner| owner == player)`), which
  matches. The doc's `owner`/`age > 60` rule is old vanilla and must NOT be added (it would be a
  regression); thrower is only an advancement/NBT field here, not a pickup gate.
- 2 player drops record thrower (M): VERIFIED - `player.rs:7046` calls
  `item_entity.set_thrower(self.gameprofile.id)` on the drop path, matching vanilla `Player.drop`.
- 3 never-despawn items do not merge (S): VERIFIED - vanilla `isMergable` (ItemEntity.java:276-278)
  excludes `pickupDelay == 32767` and `age == -32768`; Pumpkin `can_merge` (item.rs:271-278:
  `!removed && !never_despawn && !never_pickup && count < max`) matches. Doc premise wrong.
- 4 spilled contents pickup delay (S): VERIFIED - vanilla container/bundle spill spawns a plain
  `new ItemEntity` at delay 0 (`Containers.dropItemStack`, `ItemUtils.onContainerDestroyed`); only
  `Block.popResource` sets 10. Pumpkin keeps `set_pickup_delay(0)` (item.rs:473), which is correct.
  Doc premise wrong.
- 5 despawn before merge (S): VERIFIED - vanilla `ItemEntity.tick` merges at ItemEntity.java:203-205
  before `age++` (207-209) and despawn (219). Pumpkin `process_age_and_merge` (item.rs:596-635)
  merges at 604-606 before the age increment (610-614) and discard (616). Matches.

Net: no source edits this pass; both rows VERIFIED against the 1.21.x vanilla decompile.

