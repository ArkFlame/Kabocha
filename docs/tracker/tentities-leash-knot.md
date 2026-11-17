# entities / Leash Knot (RE-REVIEW)

Source audited: `crates/pumpkin-core/src/entity/decoration/leash_knot.rs` (whole file) plus the
leash plumbing it relies on (`entity/mod.rs`: `leash_to`, `leash_to_if_held_by`, `unleash`,
`drop_leash`, `tick_leash`; `item/items/lead.rs`; `entity/type.rs`).  Vanilla reference is
`net.minecraft.world.entity.decoration.LeashFenceKnotEntity` (extends `BlockAttachedEntity`) and
`LeadItem`.  No decompiled class is available in this worktree (`.vendor/` absent, no local
`LeashFenceKnotEntity.java`), so each item below is tagged with its confidence.

Verdict: the structure matches vanilla (get_or_create/get_knot over a 2 block box, 0.375 y offset,
fence-tag survival check, right-click transfer, damage destroys the knot).  Three small divergences
were fixed; five remain open and need the decompiled class before editing (WORKER.md rule 12).

## Fixed in this audit
- `leash_knot.rs:63` (`create_knot`): played `item.lead.tied` itself, so `item/items/lead.rs:96`
  played it a second time when tying to a fence and `restore_leash_from_save` (`entity/mod.rs:5556`)
  replayed it every time a chunk with a saved leash loaded. Vanilla `LeashFenceKnotEntity.getOrCreateKnot`
  is silent; `LeadItem.bindPlayerMobs` / `playPlacementSound` owns the sound. Removed.
- `leash_knot.rs:141` (`interact`): searched with `EntityDimensions { width: 32, height: 32 }`, which
  `BoundingBox::new_from_pos` turns into +/-16 blocks horizontally and `y .. y+32` vertically, so a
  player could tie mobs from 16 blocks away and never from below the knot. Vanilla builds
  `new AABB(x-7, y-7, z-7, x+7, y+7, z+7)`. Replaced with the 7 block cube.
- `leash_knot.rs:172` (`interact`): set `attached_mob = true` even when `leash_to` returned false
  (a plugin cancelling `PlayerLeashEntityEvent`), so success was reported and the placement sound
  played with no leash moved. Now `attached_mob |= mob.leash_to(..)`.

## Remaining gaps

| # | file:line | vanilla | suggested fix | size |
|---|---|---|---|---|
| 1 | `leash_knot.rs:169-183` (`interact`) | `LeashFenceKnotEntity.interact`: when no mob was moved off the player (`!bl`) the knot is `discard()`-ed and the sound is played; mobs still leashed to it drop their leads on their next `tickLeash`. | Replace the `leash_to_if_held_by(knot_id, player)` "give mobs back to the player" branch with `self.entity.remove()` when `!attached_mob`, then verify against decompiled `LeashFenceKnotEntity.java` (Pumpkin may have added the re-home path deliberately). High confidence on vanilla, still needs the class. | M |
| 2 | `leash_knot.rs:107-131` (`tick` survival) | `BlockAttachedEntity.tick` discards the knot as soon as `survives()` is false (block not in `BlockTags.FENCES`); each mob then releases itself with `dropLeash(true, true)`, dropping a lead. | Rust instead keeps the knot alive while any entity within the 32 block scan is still leashed to it, so a knot on a broken fence can persist after its mobs leave the scan. Remove the `retained_leash` early return once the mob-side `tick_leash` release path is confirmed to cover the same range. | M |
| 3 | `leash_knot.rs:110-116` (`tick` scan box) | The release scan is Pumpkin-only (vanilla releases mob-side), but as written the box is `x+/-16, z+/-16, y .. y+32`: mobs below the knot are never released. | Make the vertical extent symmetric (or reuse one shared scan helper with `lead.rs:48`, which already centres its box with `center.y - 16.0`). | S |
| 4 | `leash_knot.rs:73`, `leash_knot.rs:196` (sounds) | Vanilla uses `SoundEvents.LEASH_KNOT_PLACE` (`block.leash_knot.place`) for placement and `LEASH_KNOT_BREAK` (`block.leash_knot.break`) when the knot is hit; `item.lead.tied`/`item.lead.untied` are the player-side tie/untie sounds. | `crates/pumpkin-data/src/generated/sound.rs:941` only knows `ItemLeadTied`/`ItemLeadUntied`; add `block.leash_knot.place`/`block.leash_knot.break` to `assets/` and regenerate with `cargo run --locked -p pumpkin-codegen`, then use them in `play_placement_sound`/`damage_with_context`. | M |
| 5 | `leash_knot.rs:189-215` (`damage_with_context`) | `LeashFenceKnotEntity.hurt` is `isInvulnerableTo` -> (`!isClientSide && !isRemoved`) { `kill()`; `markHurt()`; play break sound } and returns true; there is no mob-griefing branch there. | Drop the `mob_griefing` gate (a mob hitting the knot should still destroy it) and call `self.entity.mark_hurt()`; confirm the gate is not inherited from `BlockAttachedEntity` in this version first. | S |

## Notes (not gaps)
- `get_knot` (`leash_knot.rs:27-51`) uses a 2 block box plus `block_pos` equality, the same predicate
  vanilla uses; the asymmetric y extent cannot mis-match because of the block-pos comparison.
- `can_hit` -> `isPickable`, the `OFFSET_Y = 0.375` spawn offset and spawning the knot through
  `world.spawn_entity` match `LeashFenceKnotEntity`/BlockAttachedEntity.
- `play_placement_sound` mirrors vanilla `playPlacementSound`; only its sound constant is wrong (gap 4).
- Leash persistence (`write_leash_data`/`read_leash_data`) stores knots as a block position, matching
  vanilla's `leash` int array, so knots are not saved as standalone entities; no gap.
- Client-side rendering distance (`shouldRenderAtSqrDistance`) is client only.
