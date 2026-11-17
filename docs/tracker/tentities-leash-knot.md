# entities / Leash Knot (RE-REVIEW)

Source audited: `crates/pumpkin-core/src/entity/decoration/leash_knot.rs` (whole file, at commit
5175e3efd "match vanilla leash knot sound and interact range") plus the leash plumbing it relies on
(`entity/mod.rs`: `leash_to`, `leash_to_if_held_by`, `unleash`, `drop_leash`, `tick_leash`;
`item/items/lead.rs`; `entity/type.rs`).  Vanilla reference is
`net.minecraft.world.entity.decoration.LeashFenceKnotEntity` (extends `BlockAttachedEntity`) and
`LeadItem`.  No decompiled class is available (`.vendor/` absent, nothing under `../`), so items
that depend on an exact vanilla branch are marked UNCERTAIN.

Verdict: structure still matches vanilla (get_or_create/get_knot, 0.375 y offset, fence-tag survival
check, right-click transfer, damage destroys the knot).  Two further divergences are fixed below;
the rest need the decompiled class or assets/codegen.

## Fixed in this review
- `leash_knot.rs:92-96` (`tick`, release scan): used `BoundingBox::new_from_pos(pos, dim)`, which
  starts the box at the knot own y and only grows upwards (`boundingbox.rs:75-81`: `min.y = y`).
  Mobs more than about two blocks below a broken knot were never released. Now a symmetric
  +/-16 box around the knot, matching the intent of the release scan and
  `drop_all_leash_connections` (`entity/mod.rs:5457-5459`).
- `leash_knot.rs:99`, `115-122` (`tick`): breaking the fence spawned a lead for every released mob
  regardless of `doEntityDrops`, while the mob-side path (`tick_leash`, `entity/mod.rs:5591-5600`)
  honours `entity_drops`. Lead refunds now use the same game rule.

## Remaining gaps

| # | file:line | vanilla | suggested fix | size |
|---|---|---|---|---|
| 1 | `leash_knot.rs:173-190` (`interact`) | UNCERTAIN. `LeashFenceKnotEntity.interact`: mobs leashed to the player are moved onto the knot; if none was moved (`!bl`) the knot is discarded, one lead is dropped when `doEntityDrops` allows, and each mob still attached drops its own lead on its next `tickLeash`. | Pumpkin instead re-homes the knot mobs onto the player with `leash_to_if_held_by`. If the vanilla branch is confirmed, replace that block with `self.entity.remove()` plus one `LEAD` drop gated on `entity_drops`, and return true. Needs the decompiled class before editing (WORKER.md rule 12). | M |
| 2 | `leash_knot.rs:68`, `131`, `223` (sounds) | `SoundEvents.LEASH_KNOT_PLACE` (`block.leash_knot.place`) on placement and `LEASH_KNOT_BREAK` (`block.leash_knot.break`) on hit and fence break; `item.lead.tied`/`item.lead.untied` are the player-side tie/untie sounds, not the knot ones. | `pumpkin-data/src/generated/sound.rs:941-942` only knows `ItemLeadTied`/`ItemLeadUntied` (subtitles exist already: `translation.rs:17529-17531`). Add both `block.leash_knot.*` sounds to `assets/` and regenerate with `cargo run --locked -p pumpkin-codegen`, then use them in `play_placement_sound`, `tick` and `damage_with_context`. | M |
| 3 | `leash_knot.rs:214-220` (`damage_with_context`) | `LeashFenceKnotEntity.hurtServer` is `isInvulnerableTo` -> if not removed { `kill()`; `markHurt()`; break sound } and returns true; no `mob_griefing` gate (that rule guards mob world edits, not hurting a knot). | Drop the `mob_griefing` branch and call `self.entity.mark_hurt()` (`entity/mod.rs:2401`); confirm against the decompiled `hurtServer` first, the gate is harmless for player attacks. | S |
| 4 | `leash_knot.rs:128-131` (`tick`) | `LeashFenceKnotEntity.tick` discards the knot as soon as `survives()` is false; attached mobs then release themselves through `tick_leash` (dead-holder path, `entity/mod.rs:5591-5600`), which already drops the lead. | The `retained_leash` early return keeps a knot alive indefinitely when its mobs are outside the scan (cancelled `EntityUnleashEvent` or a plugin-held leash). Once the mob-side release is trusted, remove the scan and just `remove()` the knot. | M |
| 5 | `leash_knot.rs:42` (`get_knot`) | `getOrCreateKnot` searches `AABB(i-1, j-1, k-1, i+1, j+1, k+1)`, +/-1 around the block on every axis. | The box here is `y .. y+2`, so a knot whose entity sits below the fence block (summoned or plugin-placed) is not found and a duplicate is created. Use `new_from_pos(center.x, center.y - 1.0, center.z, ..)`. | S |
| 6 | `leash_knot.rs:143-155` (`interact`) | Vanilla measures the 7 block transfer area from the integer block coordinates, so the box reaches one block wider on the positive side than a box centred on the entity. | Cosmetic 0.5 block asymmetry from centring on the entity position; build the box from `pos.0` block coordinates if exact parity matters. | S |

## Notes (not gaps)
- `get_or_create`/`create_knot` (`leash_knot.rs:25-64`) are silent, matching vanilla; the placement
  sound belongs to `LeadItem` (`item/items/lead.rs:94`) and to `play_placement_sound`.
- `interact` uses `attached_mob |= mob.leash_to(..)` so a cancelled `PlayerLeashEntityEvent` no
  longer reports success.
- The 7 block transfer cube, `can_hit` -> `isPickable`, the `OFFSET_Y = 0.375` spawn offset and
  spawning through `world.spawn_entity` match `LeashFenceKnotEntity`/`BlockAttachedEntity`.
- Leash persistence stores knots as a block position (`restore_leash_from_save`,
  `entity/mod.rs:5556`), matching the vanilla leash int array; knots are not saved as entities.
- Client-only: `shouldRenderAtSqrDistance` (1024) rendering distance.
