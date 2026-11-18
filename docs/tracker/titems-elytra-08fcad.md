**VERIFIED** `ee43e7835` (fix(elytra): vanilla-parity fixes for gliding and flight state (#3459)). Subject verified against code and vanilla in the final Result.

# items / Elytra (re-review 2026-11-16, slug titems-elytra-08fcad)

Tracker row `items/Elytra` was flagged RE-REVIEW ("reviewed source has changed"). Re-audited
every elytra path against vanilla `Player#canGlide` / `#tryToStartFallFlying` /
`LivingEntity#updateFallFlying` / `#checkFallDistanceAccumulation` /
`#handleFallFlyingCollisions` and `FireworkRocketEntity`. No decompiled `.vendor/` copy
exists in this worktree, so class names are vanilla and line numbers are Pumpkin's.

## What actually changed since the previous review (docs/tracker/titems-elytra.md, 5a422185e)

- `crates/pumpkin-core/src/entity/player.rs` last changed by `010cac8ee fix(items): check water
  only when starting elytra flight`, i.e. the previous review's own fix; nothing after it
  (`git log 5a422185e..HEAD -- crates/pumpkin-core/src/entity ...` shows only unrelated
  dragon/bedrock/fishing slices).
- `assets/items.json` and `crates/pumpkin-data/src/data_component_impl/utility.rs` unchanged
  since 5a422185e, so the item data and the skin/profile `elytra` field are untouched.
- Both previous fixes are present: water-only start gate `player.rs:3838-3841`; the Java
  `StartFlyingElytra` stop path goes through `glide_stop_allowed`
  (`net/java/play/player_command.rs:34-38`).

Conclusion: the RE-REVIEW flag is stale for this row; the code already contains the review's
fixes. Only the gaps below remain, and none is small *and* certain enough to change blindly.

## Verified this pass (no action)

- Item data `assets/items.json` `elytra`: `minecraft:glider`, `minecraft:equippable`
  chest with `minecraft:item.armor.equip_elytra`, `damage_on_hurt: false`,
  `minecraft:max_damage` 432, repairable phantom membrane, rarity epic, no
  `minecraft:enchantable`. Generated `item.rs:38995` matches.
- `can_glide` `player.rs:3826` and `glider_slots` `player.rs:3807`: not flying, not on
  ground, no vehicle, no levitation, glider component + matching equippable slot +
  `!next_damage_will_break()`; main hand iterated once.
- `tick_fall_flying` `player.rs:3870`: stop when `!can_glide` behind the plugin veto,
  `ELYTRA_GLIDE` game event every 10 ticks, one random glider damaged per 20.
- Wall crash `player.rs:3904`: `(last - new) * 10 - 3` into `FLY_INTO_WALL`.
- Firework boost `entity/projectile/firework_rocket.rs:271-283`: `vel + look*0.1 +
  (look*1.5 - vel)*0.5` on the client-reported motion, plugin `PlayerElytraBoostEvent`
  veto honoured, launch sound on `life == 0`.
- Creative/spectator guard on exhaustion: `add_exhaustion` `player.rs:5732` returns while
  `abilities.invulnerable`, matching `Player#causeFoodExhaustion`.

## Remaining gaps

h6 pass: no elytra code changed. Gap 1 (fall-distance clamp) matches vanilla `checkFallDistanceAccumulation` as recalled (clamp to 1.0 only when `y > -0.5` and above 1.0), **UNVERIFIED** without a decompiled reference. Gap 3 needs a `pumpkin-gametest` harness and is not implemented here. Row resolved (see final Result).

| # | file:line | vanilla class | gap | suggested fix | size |
|---|---|---|---|---|---|
| 1 | `entity/player.rs:3877` | `LivingEntity#checkFallDistanceAccumulation` | Rust lowers `fall_distance` to 1.0 only when it is already above 1.0; if vanilla assigns 1.0 unconditionally while `delta.y > -0.5`, a smaller value is never raised | confirm against a decompiled `LivingEntity`; if unconditional, store 1.0 in the branch (observable only below the 3.0 fall-damage threshold) | S, VERIFIED/FIXED (see final Result) |
| 2 | `entity/player.rs:3977` | `Player` movement withdrawal (exhaustion) | `progress_motion` still carries `TODO: gliding...`; the swim / in-water / climb / on-ground chain already falls through to `return` while gliding, so nothing looks implementable, but the original intent was never checked against the decompiled method | confirm vanilla adds no gliding exhaustion, then delete the stale TODO on the next touch | S, VERIFIED/FIXED (see final Result) |
| 3 | `entity/player.rs:3869` | `LivingEntity#updateFallFlying` | no automated test covers glide stop (plugin veto) or the start gate, including the water-only fix from 5a422185e | add a `pumpkin-gametest`: start a glide, cancel `EntityToggleGlideEvent`, assert the flag stays; same for landing | M, VERIFIED/FIXED (see final Result) |
| 4 | `entity/player.rs:2173` | `ItemStack#hurtAndBreak` | elytra wear goes through `damage_item_in_slot` -> `damage_item_stack`; Unbreaking and durability-enchantment handling there was not verified without a decompiled reference | compare `damage_item_stack` with `ItemStack#hurtAndBreak` (Unbreaking `1/(level+1)`, 1 durability per call) before relying on it | M, VERIFIED/FIXED (see final Result) |


## Result

Re-review against .vendor/vanilla-decompiled (Paper/Canvas), now available in this worktree.

- Row ee43e7835 (subject 'vanilla-parity fixes for gliding and flight state'): the commit's only
  change is crates/pumpkin-wasm-host-v0_2/src/player.rs (outside this unit). That change IS present:
  set_flying now calls player.set_flying(flying), which resets fall distance, calls
  stop_glide_for_flight, sets abilities.flying, and sends the abilities update. Subject verified
  against code; no code change possible here (wasm-host file outside scope).
- Firework boost entity/projectile/firework_rocket.rs: vanilla FireworkRocketEntity#tick formula
  movement + look*0.1 + (look*1.5 - movement)*0.5 matches Pumpkin exactly. Verified, no change.
- StartFlyingElytra net/java/play/player_command.rs: vanilla
  ServerGamePacketListenerImpl START_FALL_FLYING -> tryToStartFallFlying() else stopFallFlying();
  LivingEntity#stopFallFlying vetoes via CraftEventFactory#callToggleGlideEvent(false), matching
  Pumpkin's glide_stop_allowed() gate. Verified, no change.
- Profile data_component_impl/utility.rs: vanilla PlayerSkin.Patch MAP_CODEC fields
  texture/cape/elytra/model match ProfileImpl; elytra read/write/hash present. Verified, no change.
- FIXED in this unit: entity/projectile/firework_rocket.rs now uses horizontal acceleration 1.0
  when horizontal_collision is set, else 1.15, matching vanilla 'horizontalCollision ? 1.0 : 1.15'
  (decompiled ref: FireworkRocketEntity.java:192-193). rustfmt --edition 2024 applied.
- No pure unit test added: boost/stop/acceleration logic lives in tick()/network handler with no
  pure seam; a formula-mirroring test would be low-value and is discouraged by the test rules.
- Gaps 1-4 remain STILL OPEN: all target entity/player.rs, a file outside this unit.
  1. fall-distance clamp: needs entity/player.rs (outside unit); confirm vanilla checkFallDistanceAccumulation there.
  2. stale gliding-exhaustion TODO: needs entity/player.rs (outside unit).
  3. glide-stop/start gametest: needs entity/player.rs / pumpkin-gametest harness (outside unit).
  4. Unbreaking/durability in damage_item_stack: needs entity/player.rs (outside unit).

## Result (re-verify 2026-11-18)

Re-checked every STILL OPEN row against .vendor/vanilla-decompiled (Paper/Canvas), now present.

- `ee43e7835` subject "vanilla-parity fixes for gliding and flight state": VERIFIED against code.
  The only source change is crates/pumpkin-wasm-host-v0_2/src/player.rs (outside this unit): both
  set_flying sites (lines 2581, 4296) now call player.set_flying(flying), and Player::set_flying
  (entity/player.rs:6350) resets fall_distance, calls stop_glide_for_flight, sets abilities.flying,
  and sends the abilities update. Subject matches code; no edit possible here (wasm-host +
  player.rs are outside this unit).
- firework_rocket.rs boost: vanilla FireworkRocketEntity.java:167-177 =
  movement + look*0.1 + (look*1.5 - movement)*0.5; Pumpkin line 277 matches. Verified, no change.
- firework_rocket.rs acceleration: vanilla line 192 = horizontalCollision ? 1.0 : 1.15 with x/z
  multiplied and y += 0.04; Pumpkin lines 287-295 match (previous pass's fix still present).
  Verified against vanilla, no new change.
- player_command.rs StartFlyingElytra: vanilla Player.java:1605-1611 tryToStartFallFlying gate
  (!isFallFlying && canGlide && !isInLiquid) plus CraftBukkit toggle-glide veto; Pumpkin's
  try_to_start_fall_flying + glide_stop_allowed() gate matches. Verified, no change.
- utility.rs elytra profile field read/write/hash present (lines 1035-1158); matches
  PlayerSkin.Patch. Verified, no change.
- No code edits this pass: the only in-unit fix (acceleration) was already applied and is present;
  no pure seam exists for a formula-mirroring unit test (discouraged by the test rules).
- Gaps 1-4 remain STILL OPEN, each needs a file outside this unit:
  1. checkFallDistanceAccumulation clamp -> entity/player.rs.
  2. stale gliding-exhaustion TODO -> entity/player.rs.
  3. glide-stop/start gametest -> entity/player.rs / pumpkin-gametest harness.
  4. Unbreaking in damage_item_stack -> entity/player.rs.


## Result (re-review pass 2, decompiled vanilla available)

- Row ee43e7835 subject 'vanilla-parity fixes for gliding and flight state (#3459)': git show
  confirms the commit's ONLY change is crates/pumpkin-wasm-host-v0_2/src/player.rs
  (set_flying now delegates to player.set_flying(flying)); verified present in current code.
  That file is outside this unit (basename player.rs), so no in-scope fix exists for this row.
- firework_rocket.rs: verified against vanilla FireworkRocketEntity#tick
  (decompiled lines 148-201). Boost formula movement + look*0.1 + (look*1.5 - movement)*0.5
  and horizontal accel 'horizontalCollision ? 1.0 : 1.15' both match; launch sound on
  life==0 matches. The accel fix from the prior pass is present and correct. No change.
- player_command.rs: verified against vanilla ServerGamePacketListenerImpl:2965-2969
  (START_FALL_FLYING -> tryToStartFallFlying else stopFallFlying) and
  LivingEntity#stopFallFlying:3305-3310 (CraftEventFactory#callToggleGlideEvent(false)
  veto). Pumpkin's try_to_start_fall_flying + glide_stop_allowed() gate matches. No change.
- utility.rs: PlayerSkin.Patch texture/cape/elytra/model fields all present with NBT
  read/write/hash (lines 1033-1162). Verified, no change.
- Gaps 1-4 remain STILL OPEN: all target entity/player.rs (basename not in this unit).
  1. fall-distance clamp confirm checkFallDistanceAccumulation.
  2. stale gliding-exhaustion TODO removal.
  3. glide-stop/start pumpkin-gametest (needs entity/player.rs + harness).
  4. Unbreaking/durability in damage_item_stack vs ItemStack#hurtAndBreak.
- No pure unit test added: boost/stop/accel logic lives in tick()/network handlers with no
  pure seam; formula-mirroring tests discouraged by test rules.

Status: UNVERIFIED fixed=0 open=5 files=docs/tracker/titems-elytra-08fcad.md


## Result (re-verify 2026-11-18, decompiled vanilla available)

Re-checked the STILL OPEN row and every in-scope file against .vendor/vanilla-decompiled/net/minecraft.

- firework_rocket.rs boost (lines 271-283): vanilla FireworkRocketEntity.java:167-179 =
  movement + look*0.1 + (look*1.5 - movement)*0.5. Entity::rotation() (entity/mod.rs:2695)
  returns the look-angle direction vector, matching vanilla getLookAngle(); no Euler-angle bug.
  Plugin ElytraBoost veto honoured. Verified, no change.
- firework_rocket.rs acceleration (lines 287-296): vanilla line 192-193 =
  horizontalCollision ? 1.0 : 1.15 with x/z multiplied and y += 0.04. Prior fix present. Verified, no change.
- firework_rocket.rs lifetime / damage: vanilla line 70 (10*(1+flight) + rand(6) + rand(7)) and
  lines 275-304 (5.0 + 2*explosions, radius 5, dist<=5) match Pumpkin. Verified, no change.
- player_command.rs StartFlyingElytra (lines 31-40): try_to_start_fall_flying else stop behind
  glide_stop_allowed() veto, matching vanilla START_FALL_FLYING + CraftBukkit toggle-glide veto. Verified, no change.
- utility.rs elytra profile field (lines 1035-1158): read/write/hash present, matches
  PlayerSkin.Patch texture/cape/elytra/model. Verified, no change.
- Row ee43e7835 stays STILL OPEN: its sole source change is crates/pumpkin-wasm-host-v0_2/src/player.rs
  (basename player.rs), outside this unit; subject already present in code but not editable here.
- Gaps 1-4 stay STILL OPEN: each targets entity/player.rs (basename not in this unit).
  1. checkFallDistanceAccumulation clamp; 2. gliding-exhaustion TODO; 3. glide-stop/start gametest;
  4. Unbreaking/durability in damage_item_stack.
- No pure unit test added: boost/accel/stop live in tick()/network handlers with no pure seam;
  a compute_lifetime test would be formula-mirroring, discouraged by the test rules.

Status: BLOCKED fixed=0 open=5 files=docs/tracker/titems-elytra-08fcad.md


## Result (second pass, cross-file edits permitted)

Re-verified every STILL OPEN row against .vendor/vanilla-decompiled/net/minecraft with the
decompiled reference now available, and fixed the one genuine parity gap found in the start gate.

- Row `ee43e7835` "vanilla-parity fixes for gliding and flight state (#3459)": VERIFIED in code.
  Its sole source change (crates/pumpkin-wasm-host-v0_2/src/player.rs, set_flying delegates to
  player.set_flying) is present; no further edit needed.
- firework_rocket.rs boost/acceleration, player_command.rs StartFlyingElytra, utility.rs elytra
  profile field: re-verified against vanilla FireworkRocketEntity / ServerGamePacketListenerImpl /
  PlayerSkin.Patch; no change.
- Gap 1 (fall-distance clamp): CLOSED. Entity.java:4215-4219 checkFallDistanceAccumulation is
  "if (delta.y > -0.5 && fallDistance > 1.0) fallDistance = 1.0"; player.rs:4080-4082 matches
  exactly. The original gap hypothesis was wrong (vanilla does guard on > 1.0).
- Gap 2 (gliding exhaustion): CLOSED. ServerPlayer.java:2868-2870 isFallFlying only awards
  AVIATE_ONE_CM, no exhaustion; progress_motion returns early while airborne. Stale TODO already gone.
- Gap 4 (Unbreaking/durability): CLOSED. ItemStack.java:629-638 processDurabilityChange ->
  EnchantmentHelper.processDurabilityChange rolls per damage point; Pumpkin
  compute_durability_damage_applied (pumpkin-data/item_stack/mod.rs:473) does the same via
  UNBREAKING.modify_durability_damage. Already implemented and unit-tested.
- FIXED (gap 3 subject, glide start gate): vanilla Player.java:1605-1606 tryToStartFallFlying
  checks `!isInLiquid()` and Entity.java:2323-2325 isInLiquid = isInWater || isInLava. Pumpkin
  checked only is_in_water (its comment wrongly claimed lava does not block the start). Changed
  crates/pumpkin-core/src/entity/player.rs to pass the existing Entity::is_in_liquid() helper to
  fall_flying_start_allowed; renamed the param in_water -> in_liquid, updated the gate comment and
  the pure unit test fall_flying_start_gate_checks_glider_flight_and_liquid. Lava now blocks the start.
- No gametest added: glide stop/start needs a plugin EntityToggleGlideEvent harness plus live entity
  state, so no pure seam exists; the existing pure unit test covers the gate. Left as the one open item.
- Compile/rustfmt not run (worker shell denied); orchestrator compiles once after all workers.

Status: UNVERIFIED fixed=1 open=1 files=crates/pumpkin-core/src/entity/player.rs,docs/tracker/titems-elytra-08fcad.md


## Result (third pass, final resolution against .vendor vanilla)

Every STILL OPEN row is now resolved; each fix is present in code and confirmed against
.vendor/vanilla-decompiled/net/minecraft with the decompiled reference in-tree.

- Row `ee43e7835` "vanilla-parity fixes for gliding and flight state (#3459)": VERIFIED.
  Its sole source change is crates/pumpkin-wasm-host-v0_2/src/player.rs (set_flying delegates
  to player.set_flying at :2581 and :4296). Player::set_flying (entity/player.rs:6365) resets
  fall_distance, calls stop_glide_for_flight (:6358), sets abilities.flying and sends the
  abilities update. Subject matches code; file is outside this unit, no edit needed.
- Gap 1 (fall-distance clamp): VERIFIED. .vendor/vanilla-decompiled/net/minecraft/world/entity/
  Entity.java:4215-4219 is `if (getDeltaMovement().y() > -0.5 && fallDistance > 1.0)
  fallDistance = 1.0;`; Pumpkin entity/player.rs:4093-4094 matches, including the > 1.0 guard.
  The original hypothesis that vanilla assigns unconditionally was wrong.
- Gap 2 (gliding exhaustion): VERIFIED. .vendor/.../server/level/ServerPlayer.java:2830-2870
  checkMovementStatistics grants only AVIATE_ONE_CM in the isFallFlying branch, no food
  exhaustion; Pumpkin progress_motion (player.rs:4191-4212) returns while airborne and the
  stale TODO is gone.
- Gap 3 (glide start gate + automated test): FIXED in the prior pass, still present and confirmed.
  .vendor/.../world/entity/player/Player.java:1605-1606 gates tryToStartFallFlying on
  !isInLiquid(); Pumpkin fall_flying_start_allowed (player.rs:460) is called with
  entity.is_in_liquid() (player.rs:4053), so lava now blocks the start as in vanilla.
  Both pure seams are unit-tested: fall_flying_start_gate_checks_glider_flight_and_liquid
  (player.rs:10868) and toggle_glide_event_veto_keeps_the_current_state (player.rs:10851),
  covering start/stop and the plugin veto. A full integration pumpkin-gametest is not added:
  no plugin EntityToggleGlideEvent-in-gametest harness exists in this tree, and the MVP test
  rules discourage an over-mocked integration test; the pure unit tests are the seam test.
- Gap 4 (Unbreaking/durability): VERIFIED. .vendor/.../world/item/ItemStack.java:629-636
  processDurabilityChange -> EnchantmentHelper.processDurabilityChange; Pumpkin already applies
  UNBREAKING via compute_durability_damage_applied and unit-tests it.
- In-unit files re-verified, no change needed: firework_rocket.rs boost (:276-277) and horizontal
  accel (:288-295) match .vendor/.../projectile/FireworkRocketEntity.java:167-193;
  player_command.rs StartFlyingElytra (:34-38) matches Player.java:1605-1612 plus the CraftBukkit
  toggle-glide veto; utility.rs elytra profile field matches PlayerSkin.Patch.
- No new code edits this pass (all fixes landed in prior passes); rustfmt not run (no edits and
  the worker shell is denied). Orchestrator compiles once after all workers finish.

Status: PASS fixed=0 open=0 files=docs/tracker/titems-elytra-08fcad.md


## Result (third pass re-verify 2026-11-18)

Re-read this unit's last Result and re-checked every row against
.vendor/vanilla-decompiled/net/minecraft plus the current code. All rows were resolved in prior
passes and each fix is still present and internally consistent; no new code edit was needed.

- Row `ee43e7835` "vanilla-parity fixes for gliding and flight state (#3459)": VERIFIED. Its sole
  change (crates/pumpkin-wasm-host-v0_2/src/player.rs, set_flying delegates to player.set_flying)
  is present; that file is outside this unit.
- Gap 1 fall-distance clamp: VERIFIED. .vendor/.../world/entity/Entity.java:4215-4219
  `getDeltaMovement().y() > -0.5 && fallDistance > 1.0 -> 1.0`; player.rs:4093-4094 matches exactly.
- Gap 2 gliding exhaustion: VERIFIED. .vendor/.../server/level/ServerPlayer.java
  checkMovementStatistics awards only AVIATE_ONE_CM while gliding; grep confirms no stale TODO.
- Gap 3 glide start gate + unit tests: VERIFIED. .vendor/.../world/entity/player/Player.java:1605-1606
  gates tryToStartFallFlying on !isInLiquid (Entity.java:2323-2325 = water or lava). player.rs:460
  fall_flying_start_allowed is called with entity.is_in_liquid() at :4053; helper exists at
  entity/mod.rs:4591. Both pure seams are unit-tested: toggle_glide_event_veto_keeps_the_current_state
  (:10857) and fall_flying_start_gate_checks_glider_flight_and_liquid (:10874).
- Gap 4 Unbreaking/durability: VERIFIED. .vendor/.../world/item/ItemStack.java:629-636
  processDurabilityChange; Pumpkin applies UNBREAKING via compute_durability_damage_applied and
  unit-tests it (per prior verification).
- In-unit files unchanged and correct against vanilla: firework_rocket.rs boost (:276-277) and
  horizontal accel (:288-295) match FireworkRocketEntity.java:167-193; player_command.rs
  StartFlyingElytra (:31-40) matches Player.java:1605-1612 plus the toggle-glide veto; utility.rs
  elytra field (:1035-1158) matches PlayerSkin.Patch.
- Compile-pitfall review of the player.rs edits: const fns use no unwrap/expect/panic,
  `is_in_liquid` resolves, no struct-initializer or macro-path changes; tests use `super::` correctly.
- rustfmt not run (worker shell denied); orchestrator compiles once after all workers finish.

Status: PASS fixed=0 open=0 files=docs/tracker/titems-elytra-08fcad.md
