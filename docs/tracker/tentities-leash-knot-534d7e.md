# entities / Leash Knot (RE-REVIEW after source change)

Audited `crates/pumpkin-core/src/entity/decoration/leash_knot.rs` in full plus the leash plumbing it
uses (`entity/mod.rs`: `leash_to` 5235, `leash_to_if_held_by` 5246, `unleash` 5351,
`remove_leash` 5467 / `drop_leash` 5479, `tick_leash` 5624; `item/items/lead.rs` 40-100).
No decompiled class in this worktree (`.vendor/` absent, no `LeashFenceKnotEntity.java` locally), so
items below carry a confidence tag. Vanilla reference: `LeashFenceKnotEntity` (extends
`BlockAttachedEntity`), `LeadItem.bindPlayerMobs`.

Verdict: shape matches vanilla (2 block lookup box + block-pos equality, `OFFSET_Y = 0.375`,
fence-tag survival, right-click transfer, hit destroys). One further bug fixed below; five gaps stay
open, three of them need the decompiled class or a `pumpkin-data` regeneration.

## Fixed in this audit
- `leash_knot.rs:93-100` (`tick` release scan): built with `BoundingBox::new_from_pos(pos, width 32,
  height 32)`, which is `x/z +/-16` but `y .. y+32`, so a mob leashed below a broken-fence knot was
  never in the scan: it was neither unleashed (lead dropped) nor counted in `retained_leash`.
  Vanilla `Leashable.leashableLeashedTo` centres its 32 block box. Now
  `BoundingBox::of_size(pos, 32.0, 32.0, 32.0)`; same helper `shear_off_all_leash_connections` uses.

## Remaining gaps

| # | file:line | vanilla | suggested fix | size |
|---|---|---|---|---|
| 1 | `leash_knot.rs:171-190` (`interact`) | `LeashFenceKnotEntity.interact`: after moving the player's mobs onto the knot, if none moved (`!flag`) the knot calls `discard()`; the mobs still leashed to it then `dropLeash(true, true)` on their next tick (lead item spawned), no sound. | Replace the `leash_to_if_held_by(knot_id, player)` / `player_holder` re-home loop with: `unleash()` each entity still held by the knot and spawn a lead at it, then `self.entity.remove()`. Must land with gap 5 or mobs lose the lead silently. Confidence: high; still confirm against the decompiled class. | M |
| 2 | `leash_knot.rs:214-222` (`damage_with_context`) | `LeashFenceKnotEntity.hurtServer` checks `isInvulnerableTo` only, then destroys the knot; no `mob_griefing` gate exists there. The gate came in with the #3800 port and is not inherited from `BlockAttachedEntity`. | Delete the `game_rules.mob_griefing` branch so a mob hit still removes the knot. Confidence: high. | S |
| 3 | `leash_knot.rs:66-69`, `134`, `225-229` (sounds) | Vanilla `playPlacementSound` plays `SoundEvents.LEASH_KNOT_PLACE` and `hurtServer` plays `LEASH_KNOT_BREAK`; `item.lead.tied`/`item.lead.untied` are the player-side tie/untie sounds (also used when a leash snaps). | `crates/pumpkin-data/src/generated/sound.rs:941` only has `ItemLeadTied`/`ItemLeadUntied`. Add `block.leash_knot.place`/`block.leash_knot.break` to `assets/` and regenerate with `cargo run --locked -p pumpkin-codegen`, then use them in `play_placement_sound` and `damage_with_context`. | M (blocked on assets) |
| 4 | `leash_knot.rs:93-136` (`tick`) | `BlockAttachedEntity.tick` re-checks `survives()` every 100 ticks and discards + `dropItem()`; there is no per-tick 32 block mob scan and no `retained_leash` early return. | Remove `retained_leash` (the knot should die as soon as the fence is gone) and let mob-side `tick_leash` release them; depends on gap 5 so the lead item is still dropped. Confidence: medium; needs the decompiled `BlockAttachedEntity`. | M |
| 5 | `entity/mod.rs:5624-5645` (`tick_leash`) | Vanilla `Entity.tickLeash` drops the leash when the holder is removed/dead (`dropLeash(true, true)` spawns the lead); Pumpkin returns `LeashTick::NotLeashed` when the `Weak` no longer upgrades and never checks `holder.get_entity().is_removed()` on a successful upgrade. | After upgrading the holder, if it `is_removed()`, `unleash()` then spawn a lead at the entity. This is the general primitive gaps 1 and 4 rely on. | S-M |

## Notes (not gaps)
- `get_knot` (27-50) uses a 2 block box plus `block_pos` equality; the asymmetric y extent cannot
  produce a false positive because of the block-pos check, and a knot at that block always intersects.
- `notify_leashee_removed` (87-91) matches vanilla `LeashFenceKnotEntity.notifyLeasheeRemoved`
  (discard when the last leashee goes).
- `create_knot` is silent: vanilla `getOrCreateKnot` only spawns; `LeadItem.bindPlayerMobs` owns the
  tie sound and `block.attach` game event (`lead.rs:96-98`).
- `interact` search box (147-161) is the vanilla `AABB(x-7, y-7, z-7, x+7, y+7, z+7)`; `can_hit` and
  the 0.375 spawn offset match.
- Leash persistence stores a block position for knots, matching vanilla's leash int array.
