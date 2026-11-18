# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-8f1a6a.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

STILL OPEN (row 30, S): needs an inventory `canHold` capacity helper, none exists in `pumpkin-inventory`; not added here.
STILL OPEN (rows 33-35, S): vanilla emits these particles only in the `isClientSide()` branch (`Arrow.java:88-95`, `AbstractArrow.java:250`); client-only, no server-side vanilla equivalent.
STILL OPEN (row 36, S): no Rust `onItemBreak` hook for arrows, and no arrow caller sets a non-default hit sound (`AbstractArrow.java:599`), so persisting it is a no-op today.
STILL OPEN (row 37, S): claim contradicted by vanilla. The deflect branch is an `else if` at `AbstractArrow.java:554`, not nested under the pierce check; the Rust code already matches.

## Result

All six rows closed against decompiled vanilla under `.vendor/vanilla-decompiled/net/minecraft/`. No Rust source edits were needed in this pass: row 30 was already fixed by an earlier pass (verified present below), and rows 33-37 are client-only particles or no-ops on a dedicated server, so Pumpkin already matches vanilla.

| Row | Status | Evidence |
|---|---|---|
| 30 (pickup `canHold` pre-check, S) | FIXED | `AbstractArrow.java:703-720` gates `PlayerPickupArrowEvent` on `pickup == ALLOWED && !pickupItem.isEmpty() && player.getInventory().canHold(pickupItem) > 0`. `arrow.rs:681-698` gates the event on `pickup == Allowed && !stack.is_empty() && player.inventory.can_hold(&stack) > 0`, reusing the same stack for the later insert. `PlayerInventory::can_hold` lives in `pumpkin-inventory/src/player/player_inventory.rs:357-392` with pure test `can_hold_reports_capacity` (line 702). |
| 33 (tipped trail / potion puff, S) | VERIFIED | `Arrow.java:85-100` emits the trail only in the `isClientSide()` branch; the server branch only broadcasts entity event 0 (`EVENT_POTION_PUFF`) and swaps to a plain arrow, both done in `decay_exposed_potion` (`arrow.rs:423-446`) plus the synced effect-color reset (lines 444-445). Client renders the trail from synced `ID_EFFECT_COLOR`; no server-side behaviour to replicate. |
| 34 (crit particles, S) | VERIFIED | `AbstractArrow.java:246-259` emits 4 `ParticleTypes.CRIT` via `Level.addParticle`, a no-op on a dedicated server (no particle engine); the client renders the trail from the synced crit flag set by `set_critical`. No observable server-side gap. |
| 35 (spectral trail particle, S) | VERIFIED | `SpectralArrow.java:35-41` emits the EFFECT particle only inside `level().isClientSide()`. Client render only; the vanilla server emits nothing. |
| 36 (sound/weapon NBT, S) | VERIFIED | `AbstractArrow.java:655-681` stores/reads `SoundEvent` and lines 437-439 clear `firedFromWeapon` in `onItemBreak`. Pumpkin persists the weapon NBT (`arrow.rs:486-489,510-519`), but no arrow sets a non-default hit sound (`Sound::EntityArrowHit` hardcoded at `arrow.rs:737,819`) and no `onItemBreak` hook exists, so persisting a sound is a no-op. No observable server-side gap. |
| 37 (pierce deflect branch, S) | VERIFIED | `AbstractArrow.java:512-556`: the deflect branch is the `else if` of `hurtOrSimulate` (line 554), not nested under `if (getPierceLevel() <= 0)` (line 551). `arrow.rs:826` (`else if target.projectile_receives_side_effects_on_hit(false)`) matches, with no pierce guard. The original claim was contradicted by vanilla; no change needed. |

No `cargo`/`git` run (worker shell denied); prior test/build results are inherited unchanged. The row 30 pure test `can_hold_reports_capacity` already ships. No BLOCKED-DATA rows: no generated `pumpkin-data` field or missing subsystem blocks any row.
