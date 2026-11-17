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
