# blocks/Creaking Heart - re-review

Source: `crates/pumpkin-core/src/block/blocks/creaking_heart.rs`, `crates/pumpkin-core/src/block/entities/creaking_heart.rs`,
`crates/pumpkin-core/src/entity/mob/creaking.rs`.

No decompiled vanilla sources in this worktree (`.vendor/` absent, no vanilla repo in `../`), so the rows below are
checked against the vanilla `CreakingHeartBlock`/`CreakingHeartBlockEntity` classes as known from 1.21.4/1.21.5 plus
`docs/VANILLA_REF.md` (line 7). Re-check the numeric rows with the Extractor before fixing them.

## Fixed in this pass

- `blocks/creaking_heart.rs:130` - block had no comparator output at all (`Block::CREAKING_HEART` fell through to the
  trait default `None`). Added `get_comparator_output` matching vanilla `CreakingHeartBlockEntity#getAnalogOutputSignal`:
  `clamp(15 - floor(15 * distance / 32), 0, 15)` from the heart center to the bound creaking, `0` when no creaking is
  bound or loaded (vanilla `hasAnalogOutputSignal` is always true). Verified against `docs/VANILLA_REF.md:7`
  (`comparator S=15-floor(15d/32)`).
  Not compiled here; batch check required.

## Remaining gaps

| file:line | vanilla | suggested fix | size |
|---|---|---|---|
| entities/creaking_heart.rs:18 (`impl BlockEntity`, no `tick`) | `CreakingHeartBlockEntity#tick` re-evaluates the heart state periodically, so an `AWAKE`/`DORMANT` heart follows the `gameplay/creaking_active` day/night environment attribute (`block/blocks/creaking_heart.rs:24`) without a block/neighbour change | add `fn tick` that recomputes `check_active_logs` + `active_state` and sets the block state when it changed (vanilla cadence 20 ticks); reuse the body of `on_neighbor_update` (`blocks/creaking_heart.rs:99`) | S |
| entities/creaking_heart.rs:83 (`set_creaking_uuid`) | never called: nothing spawns or binds a creaking, so `is_protector` (`:77`) treats a UUID-less heart as protecting any creaking near its `home_pos`, and the comparator/damage-forwarding paths (`:87`, `entity/mob/creaking.rs:537`) are dead | on `tick`/`placed` when `AWAKE` and a player is within range, spawn `minecraft:creaking` at a random spot inside the two logs, store its UUID (NBT key `creaking`, already read/written), clear it when the entity is gone or farther than 34 blocks / despawn it (vanilla out-of-range despawn, `docs/VANILLA_REF.md:7`) | L |
| blocks/creaking_heart.rs:63 (`on_place`) | vanilla `natural` property distinguishes worldgen hearts from player-placed ones; only natural hearts drop XP 20-24 (and 1-3 resin clumps without Silk Touch, `docs/VANILLA_REF.md:7`) | force `props.natural = false` in `on_place` (state comes from `default_state` today) and set `natural = true` from the pale-garden feature; gate the XP drop on it | M |
| entities/creaking_heart.rs:87 (`creaking_hurt`) | vanilla forwards the hurt sound, particle and `Creaking#tearDown` timing from the heart; the mob currently polls its heart every tick (`entity/mob/creaking.rs:486-497`) | left as-is: the poll reproduces vanilla's outcome; revisit only if the poll is found to miss the heart's `set_changed` path | S |
| blocks/creaking_heart.rs (no piston hook) | vanilla creaking heart cannot be moved by pistons | verify the generated block data carries the `BLOCK` push reaction for `creaking_heart` (`pumpkin-data` generated from `assets/`); add an override only if it does not | S |
| entity/mob/creaking.rs:119,120-124 | creaking-specific wander/melee goals (`enable_creaking_melee_attack`) | see `docs/tracker/tentities-creaking.md`, same root cause from the mob side | M |
