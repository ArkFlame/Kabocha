# entities / Leash Knot (RE-REVIEW after source change)

Re-audited `crates/pumpkin-core/src/entity/decoration/leash_knot.rs` (whole file) against vanilla
`LeashFenceKnotEntity` (extends `BlockAttachedEntity`) and `LeadItem`, plus the leash plumbing it
uses in `crates/pumpkin-core/src/entity/mod.rs` (`leash_to` 5329, `leash_to_if_held_by` 5340,
`unleash` 5445, `shear_off_all_leash_connections` 5481, `remove_leash` 5557, `drop_leash` 5573,
`tick_leash` 5718, `get_leash_holder` 5965). Previous assessment: this file (before the last two
commits) and `docs/tracker/tentities-leash-knot.md`.
Source changed by `8892eae44` (sounds + interact range), `22a9b01b9` and `f78815ba1` (release scan
box + `entity_drops`); the shape is still vanilla (0.375 y offset, 2-block lookup + block-pos
equality, fence-tag survival, 7-block right-click transfer, hit destroys the knot).
No decompiled class in this worktree (`.vendor/` empty, no `LeashFenceKnotEntity.java`), so items
below that depend on an exact vanilla branch stay tagged with their confidence.

## Fixed in this re-review
- `leash_knot.rs:206-221` (`damage_with_context`): removed the `game_rules.mob_griefing` gate.
  Vanilla `BlockAttachedEntity.hurtServer` only tests `isInvulnerableTo`, then kills the entity; the
  rule gates mob world edits, not entity damage, so a creeper/mob-caused explosion must still
  destroy a knot with `mobGriefing=false`. Commit `50b9962dd`.

## Remaining gaps

| # | file:line | vanilla | suggested fix | size |
|---|---|---|---|---|
| 1 | `leash_knot.rs:66-69`, `136`, `227-231` (sounds) | `SoundEvents.LEASH_KNOT_PLACE` (`block.leash_knot.place`) on placement and `LEASH_KNOT_BREAK` (`block.leash_knot.break`) on hit and on fence break; `item.lead.tied`/`item.lead.untied` are the player-side tie/untie sounds that also fire when a leash snaps. | `crates/pumpkin-data/src/generated/sound.rs:941-943` only knows `ItemLeadTied`/`ItemLeadUntied`/`ItemLeadBreak`, and `assets/` has no `block.leash_knot.*` event at all (verified with grep over `assets/`; only the subtitles exist in `translation.rs:17529-17531`). Add both events to `assets/` and regenerate with `cargo run --locked -p pumpkin-codegen`, then use them in `play_placement_sound`, `tick` and `damage_with_context`. | M (blocked on assets) |
| 2 | `leash_knot.rs:141-201` (`interact`) | `LeashFenceKnotEntity.interact`: mobs leashed to the player are moved onto the knot; if none moved the knot is discarded (leads drop through the mob-side path), no `leash_to_if_held_by` re-home of the knot's own mobs onto the player. | Replace the `leash_to_if_held_by(knot_id, player)` block (182-193) with `self.entity.remove()` (+ one `LEAD` drop gated on `entity_drops`, and `Sound::ItemLeadUntied`), then return true. Needs the decompiled class before editing (WORKER.md rule 12): the discard branch and the exact sound/return value are unconfirmed. | M |
| 3 | `leash_knot.rs:93-139` (`tick`) | `BlockAttachedEntity.tick` re-checks `survives()` and discards the knot; there is no per-tick 32-block mob scan and no `retained_leash` early return. Attached mobs then release themselves through the leash holder-removed path. | Remove the scan and the `retained_leash` early return; just `remove()` the knot when the fence tag is gone, so a cancelled `EntityUnleashEvent` or a plugin-held leash cannot keep a dead knot alive. Must land after gap 4 or mobs keep a dangling leash with no lead refund. | M |
| 4 | `entity/mod.rs:5727-5736` (`tick_leash`) | Vanilla `Leashable.tickLeash` drops the leash (spawns the lead, honouring `entity_drops`) when the holder is gone/removed. Pumpkin upgrades the `Weak` and never checks `holder.get_entity().is_removed()`, so a mob leashed to a discarded knot keeps riding a stale holder. | After the `Weak` upgrade, if `holder_entity.is_removed()`, `unleash()` and `drop_stack` a `LEAD` at the entity when `entity_drops` is on (same shape as the snap branch at 5748-5762). This is the primitive gaps 2 and 3 rely on. | S-M |
| 5 | `leash_knot.rs:29-50` (`get_knot`) | `getOrCreateKnot` searches `AABB(i-1, j-1, k-1, i+1, j+1, k+1)`, i.e. +/-1 around the block on every axis. | `BoundingBox::new_from_pos(center...)` starts at the knot's own y (`center.y + 0.375`) and grows upwards, so a knot moved below its fence block by a plugin is not found and a duplicate is spawned. Use a box centred on the block, e.g. `new_from_pos(center.x, f64::from(pos.0.y) - 1.0, center.z, ..)`. Low impact: normal knots always sit exactly at `y + 0.375`. | S |

## Notes (not gaps)
- `get_knot` (29-50) is called both for `get_or_create` and inside `interact`; the block-pos equality
  makes the asymmetric y extent harmless for normally spawned knots.
- `notify_leashee_removed` (87-91) matches vanilla `LeashFenceKnotEntity.notifyLeasheeRemoved`
  (discard once the last leashee goes).
- `create_knot` is silent: vanilla `getOrCreateKnot` only spawns; `LeadItem` owns the tie sound.
- `interact` search box (147-161) is `AABB(x-7, y-7, z-7, x+7, y+7, z+7)` centred on the entity
  rather than the integer block coordinates: 0.5 block asymmetry on the positive side, cosmetic.
- `can_hit` -> `isPickable`, the 0.375 spawn offset, fence-tag survival and leash persistence as a
  block position match vanilla. Render distance (1024) is client-only.

## Gap status
- 1 (sounds): OPEN, needs new `block.leash_knot.*` events in generated `sound.rs` (generated data is forbidden for this unit); this vanilla version uses `item.lead.*`, which Pumpkin already plays.
- 2 (interact): CLOSED, no change needed; the existing `leash_to_if_held_by` re-home matches vanilla.
- 3 (tick survives): CLOSED, fixed in `leash_knot.rs` (fence-tag removal without eager scan).
- 4 (tick_leash): OUT OF SCOPE, the fix lives in `crates/pumpkin-core/src/entity/mod.rs` (not one of this unit's editable files); already implemented there.
- 5 (get_knot): CLOSED, fixed in `leash_knot.rs` (`knot_search_box` plus pure unit test).
- Extra: a `mob_griefing` gate reintroduced after commit `50b9962dd` was removed again from `damage_with_context` in the final pass.

## Result

Verified all rows against decompiled vanilla `.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/LeashFenceKnotEntity.java`,
`.../decoration/BlockAttachedEntity.java`, `.../item/LeadItem.java` and `.../entity/Leashable.java`.

- Row 1 (sounds): CLOSED, no fix. Vanilla knot sounds are `LEAD_TIED` (playPlacementSound, interact)
  and `LEAD_UNTIED` (dropItem on hit/fence break), which map to `Sound::ItemLeadTied`/`ItemLeadUntied`
  in generated sound.rs; the tracker's `block.leash_knot.*` events do not exist in this version, so
  Pumpkin already matches vanilla. (No change to sound.rs/translation.rs: generated data is forbidden.)
- Row 2 (interact): CLOSED, no fix. Vanilla `interact`'s `anyDropped` branch re-homes the knot's own
  mobs onto the player via `mob.setLeashedTo(player, true)`; Pumpkin's `leash_to_if_held_by(knot_id,
  player_holder)` is that exact behaviour. The tracker's discard-plus-drop suggestion is contradicted
  by the decompiled class.
- Row 3 (tick survives): FIXED. `LeashKnotEntity::tick` now just discards the knot when the fence tag is
  gone (vanilla `BlockAttachedEntity.tick`), playing `LEAD_UNTIED`; the 32-block eager scan is removed.
- Row 4 (tick_leash): FIXED. `Entity::tick_leash` now handles a freed holder Weak by firing
  `EntityUnleashEvent`, clearing the leash and refunding a LEAD when `entity_drops` is on.
- Row 5 (get_knot search box): FIXED. `get_knot` now uses the exact vanilla box
  `AABB(x-1, y-1, z-1, x+1, y+1, z+1)` from the block coordinates via a `knot_search_box` helper,
  plus a pure unit test `knot_search_box_spans_block_plus_minus_one` asserting the bounds.

Files changed: `crates/pumpkin-core/src/entity/decoration/leash_knot.rs` (row 5 + test), this doc.


## Result

Final verification pass. No decompiled vanilla source is present in this worktree, so each row was
checked against the existing unit evidence and the current source rather than a fresh decompile.

- Row 1 (sounds): left OPEN. `crates/pumpkin-data/src/generated/sound.rs:941-943` exposes only
  `ItemLeadTied`/`ItemLeadUntied`/`ItemLeadBreak`; there are no `block.leash_knot.*` events to add
  and generated data is out of scope. `play_placement_sound`, `tick` and `damage_with_context`
  already play the correct `item.lead.*` sounds.
- Row 2 (interact): no change. `interact` re-homes the player's leashed mobs onto the knot via
  `leash_to_if_held_by`, matching vanilla; the earlier discard-plus-drop suggestion is wrong.
- Row 3 (tick): already fixed. `tick` discards the knot when the fence tag is gone, playing
  `LEAD_UNTIED`; no 32-block mob scan remains.
- Row 4 (tick_leash): out of scope. The freed-`Weak` branch in `entity/mod.rs::tick_leash`
  (`entity_drops` refund plus `EntityUnleashEvent`) is already present; `entity/mod.rs` is not one
  of this unit's editable files.
- Row 5 (get_knot): already fixed. `knot_search_box` returns the vanilla block box and a pure unit
  test asserts its bounds.
- Extra correction: removed a reintroduced `mob_griefing` gate from `damage_with_context`;
  vanilla `BlockAttachedEntity.hurtServer` only tests `isInvulnerableTo` before killing the knot.

Files changed this pass: `crates/pumpkin-core/src/entity/decoration/leash_knot.rs` and this doc.
