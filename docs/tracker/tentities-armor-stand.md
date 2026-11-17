# entities/Armor Stand - re-review

Source: `crates/pumpkin-core/src/entity/decoration/armor_stand.rs` (+ `item/items/armor_stand.rs`).
No decompiled vanilla sources in this worktree (`.vendor/` absent, nothing under `../`), so the rows below are
judged against the vanilla `ArmorStand`/`ArmorStandItem` classes as known from 1.21.x. Rows marked *verify* must be
re-checked against the Extractor output before changing behaviour.

## Fixed in this pass

- `armor_stand.rs:501-503` - `read_custom_nbt` inverted `NoBasePlate`: the flag was set when the key was `false`
  *and* when it was absent, so every stand loaded from disk (vanilla always writes `NoBasePlate: 0b`) lost its base
  plate, and stands without the key did too. Vanilla `NoBasePlate` is true only when the plate is hidden.
- `armor_stand.rs:246-255` - `broken_by_anything` skipped `EquipmentSlot::MAIN_HAND` when dropping equipment
  (vanilla iterates every `EquipmentSlot`), so an item in a ShowArms stand's main hand was voided on break.
- `armor_stand.rs:630-636` - the wobble hit sound used the default 1.0 volume; vanilla plays
  `ARMOR_STAND_HIT` at 0.3.

No automated test: both fixes sit behind `EntityBase::{read_custom_nbt, damage_with_context}`, which need a live
world/entity fixture; the existing `clicked_slot`/`swap_item_stacks` unit tests do not cover them.

## Remaining gaps

| file:line | vanilla | suggested fix | size |
|---|---|---|---|
| armor_stand.rs:374-382 (`slot_bit`) | `ArmorStand.getDisabledSlotsMask(slot) = 1 << slot.getIndex()` uses the `EquipmentSlot` enum order (mainhand 0, offhand 1, feet 2, legs 3, chest 4, head 5); Pumpkin's `get_slot_index()` is the *filter* order (mainhand 0, feet 1, legs 2, chest 3, head 4, offhand 5) | store/read `DisabledSlots` through a vanilla-order bit helper, otherwise loaded and `/data merge` values disable the wrong slots | M (*verify* with .vendor) |
| armor_stand.rs:283-291 (`cause_damage`) | `ArmorStand.causeDamage` calls `brokenByAnything` (sound + equipment) and `discard`; it does **not** run `breakAndDropItem`, so fire/`IGNITES_ARMOR_STANDS` destruction does not drop the stand item itself | drop only equipment in `cause_damage`; keep `break_and_drop_items` for the `CAN_BREAK_ARMOR_STAND` path | S (*verify*) |
| armor_stand.rs:575-579 (explosion branch) | same as above: vanilla `DamagetypeTags.IS_EXPLOSION` -> `brokenByAnything` + `discard`, no stand-item drop | same helper split as the row above | S (*verify*) |
| armor_stand.rs:245 (`drop_pos`) | `new ItemEntity(level, getX(), getY() + 1.0, getZ(), stack)` | drop at `entity.pos + (0, 1.0, 0)` instead of `block_pos.up()`; differs whenever the stand sits off the block floor | S |
| armor_stand.rs:343-349 | `ArmorStand.interactAt` returns `SUCCESS` for spectators before any slot work | add an `is_spectator()` early return (check the caller `net/java/play/interact.rs` for a global guard first; there is none today) | S |
| armor_stand.rs:314-318 (`get_clicked_slot`) | `getClickedSlot` compares the hit `vec.y` directly against the small-stand thresholds (0.1+0.8, 0.9+0.3, ...); Rust also divides `y` by 0.5 for small stands, double-scaling the hit areas | drop the `scale` division and pass `position.y` through, then extend `clicked_slot` tests with small-stand values | S (*verify* small-stand branch) |
| armor_stand.rs:443-445 (`clicked_slot` offhand fallback) | vanilla `getClickedSlot` has no offhand fallback; default is `MAINHAND` | confirm against .vendor; if absent, delete the branch and its unit test | S (*verify*) |
| armor_stand.rs:613-621 (creative break) | vanilla's `CAN_BREAK_ARMOR_STAND` path still runs `breakAndDropItem` + `discard` (stand item dropped, break sound played) | route creative through `break_and_drop_items()` instead of a bare `entity.remove()` | S |
| armor_stand.rs:529-533 (`kill`) | `ArmorStand.kill` -> `breakAndDropItem` + `brokenByAnything`, so `/kill` and void damage drop the stand plus equipment; `GameEvent.ENTITY_DIE` is emitted | call `break_and_drop_items()` and emit the death game event (resolves the existing TODO) | S |
| armor_stand.rs:283-291 + 583-590 | burning only calls `cause_damage`, so `last_damage_taken`/hurt animation and the `entity.is_removed()` bookkeeping are never updated for the fire path | set `last_damage_taken` in `cause_damage` once the vanilla behaviour split above is settled | S |
