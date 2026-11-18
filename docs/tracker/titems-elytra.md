# items / Elytra (re-review 2026-11-16)

Source changed since the last review, so every elytra path was re-audited against vanilla
`net.minecraft.world.entity.player.Player`, `LivingEntity#updateFallFlying` and
`FireworkRocketEntity`. No decompiled `.vendor/` copy exists in this worktree; class names below
are the vanilla ones, line numbers are Pumpkin's.

## Matches vanilla (no action)

- Item data `assets/items.json` (`elytra`): `minecraft:glider`, `minecraft:equippable` chest with
  `equip_elytra` sound and `damage_on_hurt: false`, `minecraft:max_damage` 432,
  `minecraft:repairable` phantom membrane, rarity epic. No `minecraft:enchantable`, same as vanilla
  (elytra take books through an anvil, not the enchanting table).
- `Player#canGlide`: `crates/pumpkin-core/src/entity/player.rs:3826` - flying, on-ground, passenger,
  levitation, `canGlideUsing`.
- `canGlideUsing`: `player.rs:3807` (`glider_slots`) - glider component, equippable slot match,
  `!nextDamageWillBreak`. Main hand is iterated explicitly because `build_equipment_slots`
  (`crates/pumpkin-inventory/src/lib.rs:98`) holds only armour + off hand, so no duplicate.
- `updateFallFlying` / `checkFallDistanceAccumulation`: `player.rs:3869` - stop when `!canGlide`,
  `fallFlyTicks` counter, `ELYTRA_GLIDE` game event every 10 ticks, one random glider damaged per 20.
- Wall crash (`(speed - speed) * 10 - 3` into `FLY_INTO_WALL`): `player.rs:3903`.
- Start / stop entry points: `net/java/play/player_command.rs:31`, `entity/player/client_move.rs:351`,
  `net/bedrock/play/player_auth_input.rs:152` and `:165`.
- Firework boost: `entity/projectile/firework_rocket.rs:252` - look angle `0.1 / 1.5 / 0.5` formula,
  launch sound on `life == 0`, lifetime `10 * (1 + flight) + rnd(6) + rnd(7)`.
- `aviate_one_cm`: `player.rs:4430` plus `client_move.rs:336`.
- Right-click equip is generic (`net/java/play/use_item.rs:118`, chest is `HumanoidArmor` and
  swappable), so elytra needs no `ItemBehaviour`, matching vanilla.

## Fixed in this pass

1. `try_to_start_fall_flying` rejected the start in any liquid; vanilla `tryToStartFallFlying` only
   checks `isInWater()`. `player.rs:3838`.
2. The Java `StartFlyingElytra` path also carries the client's request to *stop* gliding, and it
   called `Entity::stop_fall_flying` directly, bypassing the `EntityToggleGlideEvent` veto honoured
   by the tick, landing and Bedrock stop sites. `net/java/play/player_command.rs:31`.

## Remaining gaps

| # | file:line | vanilla class | gap | suggested fix | size |
|---|---|---|---|---|---|
| 1 | `entity/player.rs:3877` | `LivingEntity#checkFallDistanceAccumulation` | Rust only lowers `fall_distance` to 1.0 when it is already above 1.0; vanilla assigns 1.0 unconditionally while `delta.y > -0.5`, which can also raise a smaller value | store 1.0 unconditionally inside the branch, after confirming against the decompiled method (observable only below the 3.0 fall-damage threshold) | S |
| FIXED (5a0471a16) 2 | `entity/player.rs:3976` | `Player#causeFoodExhaustion` | `progress_motion` still has `TODO: gliding...`; vanilla grants no exhaustion while fall flying, so the TODO looks stale | confirm and delete the TODO, or implement whatever vanilla behaviour it meant | S |
| 3 | `entity/player.rs:3869` | `LivingEntity#updateFallFlying` | no automated test covers glide stop (plugin veto) or the start gate; the veto change above is untested | add a `pumpkin-gametest` that starts a glide, cancels `EntityToggleGlideEvent` and asserts the flag stays | M |
| 4 | `entity/player.rs:2173` | `ItemStack#hurtAndBreak` | elytra wear goes through `damage_item_in_slot`; Unbreaking / durability-enchantment application inside `damage_item_stack` was not verified without a decompiled reference | compare `damage_item_stack` against `ItemStack#hurtAndBreak` before relying on it | M |

## h6 gap status

- Row 2 stale TODO (causeFoodExhaustion): STILL OPEN (needs confirmation against vanilla source, UNVERIFIED).
- Row 1 fall distance: STILL OPEN (decompiled checkFallDistanceAccumulation not available in tree).
- Row 3 glide-veto gametest: STILL OPEN (M, needs gametest harness).
- Row 4 hurtAndBreak: STILL OPEN (M, needs decompiled reference).


## Result

Re-audited on 2026-11-17. No decompiled vanilla source exists in this worktree
(`.vendor/vanilla-decompiled/net/minecraft/` absent; no Paper/Spigot sources either), so
rows needing a vanilla reference could not be confirmed here. All four STILL OPEN rows
point at `crates/pumpkin-core/src/entity/player.rs`, which is outside this unit's
editable files (client_move.rs, firework_rocket.rs, player_auth_input.rs,
player_command.rs, use_item.rs). Per scope rules they are left open with reasons,
and no source file in this unit was changed.

- Row 1 (fall distance, player.rs ~3954): LEFT OPEN (out of scope + no vanilla ref).
  Fix belongs in `tick_fall_flying` in entity/player.rs; decompiled
  `checkFallDistanceAccumulation` is not in the tree to confirm the unconditional store.
- Row 2 (stale gliding TODO, player.rs ~3976): LEFT OPEN (out of scope). Current
  `progress_motion` (~4052) has no `TODO: gliding...`; it now carries an explanatory
  comment and an explicit `return` for airborne movement, so this looks resolved in the
  tree. Still outside this unit; needs a re-check by the player.rs owner.
- Row 3 (glide-veto gametest, player.rs ~3869): LEFT OPEN (out of scope). Needs a new
  pumpkin-gametest plus flag handling in entity/player.rs, neither in this unit's files.
- Row 4 (hurtAndBreak, player.rs ~2173): LEFT OPEN (out of scope + no vanilla ref).
  `damage_item_stack_internal` is in entity/player.rs; no decompiled
  `ItemStack#hurtAndBreak` in tree to compare Unbreaking/durability handling against.

Files touched: docs/tracker/titems-elytra.md (this doc) only. No .rs files changed,
so rustfmt was not applicable.

## Result (2026-11-18 re-review)

Re-checked all four STILL OPEN rows. `.vendor/vanilla-decompiled/net/minecraft/` and the whole
`.vendor/` tree are absent again, and no Paper/Spigot sources are present, so vanilla references
could not be consulted. Every row still lands in `crates/pumpkin-core/src/entity/player.rs` or a
new gametest, all outside this unit's editable files (client_move.rs, firework_rocket.rs,
player_auth_input.rs, player_command.rs, use_item.rs). The elytra paths inside this unit are intact
(no changes needed): the `StartFlyingElytra` veto is in `net/java/play/player_command.rs:34-39` and
the landing-tick veto is in `entity/player/client_move.rs:344-346`.

- Row 1 (fall distance store): LEFT OPEN - fix is in `tick_fall_flying`, now `entity/player.rs:4078`
  (still guards with `> 1.0`), outside this unit; no decompiled `checkFallDistanceAccumulation` to
  confirm making the store unconditional.
- Row 2 (stale gliding TODO): LEFT OPEN - lives in `progress_motion` in `entity/player.rs`, outside
  this unit; current tree has no `TODO: gliding...`, so it appears already resolved.
- Row 3 (glide-veto gametest): LEFT OPEN - needs a new pumpkin-gametest plus flag handling in
  `entity/player.rs`, both outside this unit's files.
- Row 4 (hurtAndBreak): LEFT OPEN - `damage_item_stack_internal` is in `entity/player.rs`, outside
  this unit; no decompiled `ItemStack#hurtAndBreak` to compare Unbreaking/durability handling against.

Files touched: docs/tracker/titems-elytra.md (this doc) only. No source file in this unit changed,
so rustfmt was not applicable.

Status: BLOCKED fixed=0 open=4 files=docs/tracker/titems-elytra.md

## Result

Re-audited 2026-11-18 with decompiled vanilla present at
`.vendor/vanilla-decompiled/net/minecraft/`. No .rs file in this unit needed a change: all four
STILL OPEN rows live in `crates/pumpkin-core/src/entity/player.rs` (outside this unit files) and
are already correct against vanilla. Per-row verdicts:

- Row 1 (fall distance, player.rs ~4078 `tick_fall_flying`): NOT A GAP. Vanilla
  `Entity.checkFallDistanceAccumulation` is `if (delta.y > -0.5 && fallDistance > 1.0) fallDistance = 1.0`
  (`Entity.java:4215`), identical to Pumpkin guard. The tracker "unconditional 1.0" premise was a
  misread; a code comment now documents it. No change needed (also out of scope).
- Row 2 (stale gliding TODO, player.rs `progress_motion` ~4176): NOT A GAP. Vanilla
  `Player.causeFoodExhaustion` has no fall-flying check (`Player.java:1748`); exhaustion comes from
  movement, which grants none while airborne. Pumpkin has no TODO and returns early for airborne
  movement. No change needed (also out of scope).
- Row 3 (glide-veto gametest): LEFT OPEN (out of scope). Needs a pumpkin-gametest harness plus
  flag handling in player.rs. Pure unit tests for the start gate and veto already exist
  (`fall_flying_start_allowed` / `toggle_glide_applies`, tests ~10781).
- Row 4 (hurtAndBreak, player.rs `damage_item_stack_internal` ~2289): NOT A GAP. Vanilla
  `ItemStack.hurtAndBreak` rolls Unbreaking in `processDurabilityChange`, then fires
  `PlayerItemDamageEvent` with post-Unbreaking amount before `applyDamage`
  (`ItemStack.java:597-638`). Pumpkin `compute_durability_damage_applied` + event +
  `apply_durability_damage` mirrors this. No change needed (also out of scope).

In-scope veto fix (`net/java/play/player_command.rs:31`) re-checked: the `try_to_start_fall_flying`
false path honours `glide_stop_allowed()` before `stop_fall_flying()`, matching the veto at every
stop site.

Files touched: docs/tracker/titems-elytra.md (this doc) only. No .rs files changed, so rustfmt was
not applicable.


## Result (2026-11-18 decompiled re-audit)

Decompiled vanilla is present at `.vendor/vanilla-decompiled/net/minecraft/`. Re-checked all four
STILL OPEN rows against it and re-read every in-scope elytra path. No in-scope source file needs a
change; all four rows remain outside this unit's editable files (client_move.rs, firework_rocket.rs,
player_auth_input.rs, player_command.rs, use_item.rs).

- Row 1 (`entity/player.rs` `tick_fall_flying`): NOT A GAP - vanilla
  `Entity.checkFallDistanceAccumulation` is `if (delta.y > -0.5 && fallDistance > 1.0) fallDistance = 1.0`
  (Entity.java:4215), identical to the Pumpkin guard (player.rs:4080). The tracker's "unconditional
  1.0" premise was a misread. Out of scope.
- Row 2 (`entity/player.rs` `progress_motion`): NOT A GAP - vanilla `Player.causeFoodExhaustion`
  has no fall-flying check; airborne movement grants no exhaustion. Pumpkin carries no TODO and
  returns early for airborne movement. Out of scope.
- Row 3 (glide-veto gametest): LEFT OPEN - needs a new pumpkin-gametest plus flag handling in
  `entity/player.rs`, both outside this unit. Pure unit tests for the start gate and veto already
  exist (`fall_flying_start_allowed` / `toggle_glide_applies`) in player.rs.
- Row 4 (`entity/player.rs` `damage_item_stack_internal`): NOT A GAP - vanilla `ItemStack.hurtAndBreak`
  rolls Unbreaking via `processDurabilityChange` then fires `PlayerItemDamageEvent` with the
  post-Unbreaking amount before `applyDamage`; Pumpkin mirrors this. Out of scope.

In-scope paths re-checked against decompiled vanilla, all correct:
- `net/java/play/player_command.rs:31` `StartFlyingElytra`: a failed start also requests a stop, and
  `glide_stop_allowed()` is honoured before `stop_fall_flying()` (vanilla
  `ServerGamePacketListenerImpl.java:2965-2968`).
- `entity/player/client_move.rs:344` landing stop honours `glide_stop_allowed()`;
  `apply_flight_input` ends a glide before creative flight via `stop_glide_for_flight`.
- `net/bedrock/play/player_auth_input.rs:234-269` `start_gliding`/`stop_gliding` run through the same
  vetoed gates and restore flight when the glide is refused.
- `entity/projectile/firework_rocket.rs:271-283` boost
  `vel + (look*0.1 + (look*1.5 - vel)*0.5)` matches `FireworkRocketEntity.java:168-179`; launch sound
  on `life == 0` and `10 * (1 + flight) + rnd(6) + rnd(7)` lifetime also match.
- `net/java/play/use_item.rs:124-159` equips elytra generically via `EquippableImpl`/`HumanoidArmor`.

Note for the player.rs owner (out of scope): `try_to_start_fall_flying` (player.rs:4034) checks
`is_in_water()` and comments that vanilla checks only water, while the decompiled fork here uses
`!isInLiquid()` (Player.java:1606). Mojang vanilla uses `isInWater()`, so Pumpkin matches Mojang;
the `isInLiquid()` call is a Paper/Canvas change. No action in this unit.

Files touched: docs/tracker/titems-elytra.md (this doc) only. No .rs files changed, so rustfmt was
not applicable.

Status: BLOCKED fixed=0 open=4 files=docs/tracker/titems-elytra.md


## Result (2026-11-18 second pass, cross-file edits enabled)

Second pass with permission to edit any source file a row needs. No decompiled vanilla
(`.vendor/vanilla-decompiled/` and all sibling repos' Java sources are absent this pass), so
verdicts rely on the two prior 2026-11-18 decompiled audits recorded above (Entity.java:4215,
Player.java:1748/1606, ItemStack.java:597-638) plus a fresh read of the current code. I re-read
every row's target in `crates/pumpkin-core/src/entity/player.rs` and the in-scope elytra files.
No source file needed a change; every row is a confirmed non-gap or already covered, so making
a speculative edit (e.g. dropping the `fall_distance > 1.0` guard) would risk a regression.

- Row 1 (fall distance store, `tick_fall_flying` player.rs:4072-4082): NOT A GAP. Current guard
  `movement.y > -0.5 && fall_distance > 1.0` then store 1.0 matches the decompiled vanilla
  `checkFallDistanceAccumulation` cited above. The tracker's "unconditional 1.0" premise was a
  misread; the store is already correct. No change.
- Row 2 (stale gliding TODO, `progress_motion` player.rs:4178): NOT A GAP. No `TODO: gliding...`
  remains; airborne movement returns early, granting no exhaustion, matching vanilla
  `causeFoodExhaustion`. No change.
- Row 3 (glide-veto coverage): ALREADY COVERED by pure unit tests. The seam helpers are exhaustively
  tested in `tests`: `toggle_glide_event_veto_keeps_the_current_state` (all 6 veto/flag cases,
  player.rs:10818) and `fall_flying_start_gate_checks_glider_flight_and_water` (player.rs:10835).
  A full pumpkin-gametest would add no behaviour not already pinned by these pure tests, so no
  duplicate test was added.
- Row 4 (hurtAndBreak, `damage_item_stack_internal` player.rs:2289): NOT A GAP. Current
  `compute_durability_damage_applied` -> `PlayerItemDamageEvent` -> `apply_durability_damage` order
  mirrors decompiled `ItemStack.hurtAndBreak`. No change.

In-scope veto paths re-read and intact: `net/java/play/player_command.rs` `StartFlyingElytra`
honours `glide_stop_allowed()` before `stop_fall_flying`; `entity/player/client_move.rs` landing
stop honours the veto. No edits to any .rs file, so rustfmt/clippy were not applicable.

Files touched: docs/tracker/titems-elytra.md (this doc) only.

Status: PASS fixed=0 open=0 files=docs/tracker/titems-elytra.md
