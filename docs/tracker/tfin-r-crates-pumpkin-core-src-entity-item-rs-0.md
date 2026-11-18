# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-entity-item-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

CLOSED. Every row VERIFIED against the decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/`. No source change warranted; no `.rs` file touched,
so no rustfmt and no build required.

## Result

Rows inherited from `docs/tracker/tpar-crates-pumpkin-core-src-entity-item-rs.md` and detailed in
`docs/tracker/tentities-item.md`.

- `entities/Item` (AUDITED): VERIFIED (matches vanilla).
- `own/tentities-item` (OWN-OPEN): VERIFIED (matches vanilla); all 5 sub-gaps closed below.

Gap-by-gap, vanilla reference
`.vendor/vanilla-decompiled/net/minecraft/world/entity/item/ItemEntity.java` (1.21.x Paper/Canvas):

- 1 owner pickup protection (M): VERIFIED. `ItemEntity.playerTouch` (ItemEntity.java:490) gates
  only on `pickupDelay == 0 && (target == null || target.equals(player.getUUID()))`. Pumpkin
  `on_player_collision` (item.rs:822-834) checks `pickup_delay` / `never_pickup` and
  `target_allows_pickup` (item.rs:196-198: `target.is_none_or(|owner| owner == player)`), matching
  vanilla. The tracker's `owner`/`age > 60` rule is older vanilla and must NOT be added (it would
  be a regression); thrower is only an advancement/NBT field here, not a pickup gate. Test:
  `target_gate_matches_vanilla_player_touch` (item.rs:1112).
- 2 player drops record thrower (M): VERIFIED. `player.rs:7046` calls
  `item_entity.set_thrower(self.gameprofile.id)` on the `Player.drop` path, matching vanilla.
- 3 never-despawn items do not merge (S): VERIFIED. Vanilla `isMergable` (ItemEntity.java:278)
  excludes `pickupDelay == 32767` and `age == -32768`; Pumpkin `can_merge` (item.rs:271-278:
  `!removed && !never_despawn && !never_pickup && count < max`) matches. Doc premise wrong. Test:
  `never_pickup_or_never_despawn_items_never_merge` (item.rs:1131).
- 4 spilled contents pickup delay (S): VERIFIED. Vanilla container/bundle spill spawns a plain
  `new ItemEntity` at delay 0 (`Containers.dropItemStack` Containers.java:42,
  `ItemUtils.onContainerDestroyed` ItemUtils.java:50; field default `pickupDelay = 0`,
  ItemEntity.java:49-51); only `Block.popResource` sets 10 (Block.java:483). Pumpkin keeps
  `set_pickup_delay(0)` (item.rs:473), which is correct. Doc premise (default 10) wrong.
- 5 despawn before merge (S): VERIFIED. Vanilla `ItemEntity.tick` merges at ItemEntity.java:203-205
  before `age++` (207-209) and the despawn discard (219). Pumpkin `process_age_and_merge`
  (item.rs:596-635) merges at 604-606 before age increment (610-614) and discard (616). Matches.

Net: no source edits this pass; both rows and all 5 gaps VERIFIED against the 1.21.x vanilla
decompile.
