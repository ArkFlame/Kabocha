# blocks/Creaking Heart - re-review

Source: `crates/pumpkin-core/src/block/blocks/creaking_heart.rs`,
`crates/pumpkin-core/src/block/entities/creaking_heart.rs`,
`crates/pumpkin-core/src/entity/mob/creaking.rs`.

No decompiled vanilla sources in this worktree (`.vendor/` absent, no vanilla repo next to it), so rows are checked
against the vanilla `CreakingHeartBlock`/`CreakingHeartBlockEntity` classes as known from 1.21.4/1.21.5 plus
`docs/VANILLA_REF.md` (line 7). Re-check numeric rows with the Extractor before changing them.

## Verified already correct (previous review was stale)

- Heart state tick: `entities/creaking_heart.rs:62-69` calls `update_state` every 20 ticks, so the day/night
  `gameplay/creaking_active` flip drives `AWAKE`/`DORMANT` without a neighbour change.
- Comparator: `blocks/creaking_heart.rs:116-140` (`15 - floor(15d/32)`, 0 when unbound/unloaded), matching
  `docs/VANILLA_REF.md:7`. The exact vanilla rounding (cast vs floor) is unverified without decompiled source.
- Placement `natural`: generated default state (`pumpkin-data/.../block.rs:553594`, state offset 7) is
  `natural = false`; the pale oak feature sets `natural = true` (`pumpkin-world/.../decorator/creaking_heart.rs:50`),
  matching vanilla `getPlacementState` + feature override.
- Drops: generated loot table `BLOCKS_CREAKING_HEART` (`pumpkin-data/.../loot_table.rs:21998`) already encodes
  `creaking_heart` with Silk Touch and 1-3 `resin_clump` without it.
- Break sound (`blocks/creaking_heart.rs:143`) and neighbour/place state recomputation (`:106-114`).

## Fixed in this pass

- `blocks/creaking_heart.rs:148` - `broken` played the break sound but never granted vanilla's 20-24 experience for
  world-gen hearts (`CreakingHeartBlock#onStacksDropped` -> `dropExperienceWhenMined`). Added the `natural` guard,
  the Silk Touch / creative early-outs and `ExperienceOrbEntity::spawn(20 + rand(0..5))`. Not compiled here;
  batch check required.

## Remaining gaps

| file:line | vanilla | suggested fix | size |
|---|---|---|---|
| `entities/creaking_heart.rs:93` (`set_creaking_uuid`, no caller) | `CreakingHeartBlockEntity#tick` spawns its creaking when `AWAKE` with a player nearby, stores its UUID in `creaking`, and despawns/re-binds when the entity is gone or farther than 34 blocks | add the spawn/despawn loop in `tick`: spawn `minecraft:creaking` inside the two logs, `set_creaking_uuid(Some(uuid))`, clear on death or >34 blocks | L |
| `entities/creaking_heart.rs:87-91` (`is_protector`) | vanilla `CreakingHeartBlockEntity#isProtector` requires the stored UUID to equal the creaking's UUID (an unbound heart protects nothing) | once binding exists, return `self.creaking_uuid.load() == Some(creaking_uuid)`; the current `is_none_or` fallback is only a stand-in for the missing spawn path | S |
| `entity/mob/creaking.rs:486-503` (heart check) | vanilla creaking despawns/tears down when farther than 34 blocks from its heart | add the 34-block distance check to the existing home-heart block and start the tear-down | M |
| `entity/mob/creaking.rs:119,120-124` | creaking-specific wander/melee goals (`enable_creaking_melee_attack`) | see `docs/tracker/tentities-creaking.md`, same root cause from the mob side | M |
| `entities/creaking_heart.rs:97` (`creaking_hurt`) | vanilla heart also emits its hurt particle and `setChanged` when the creaking is hit | left as-is: sound + sway already reproduce the observable effect | S |
| piston behaviour | vanilla creaking heart cannot be pushed | not actionable in this crate: `pumpkin-data`'s `Block` has no push-reaction field and no piston push code consults one, so there is nothing per-block to fix here | M |

## h6 gap status

- Row 2 (isProtector UUID match): STILL OPEN (S, blocked on the missing creaking spawn/bind path in tick, L).
- Row 1, 3, 4, 6: STILL OPEN (L/M, need spawn loop and creaking AI teardown; piston row needs push-reaction data).
- Row 5 hurt particle: STILL OPEN (S, left as-is by design).
