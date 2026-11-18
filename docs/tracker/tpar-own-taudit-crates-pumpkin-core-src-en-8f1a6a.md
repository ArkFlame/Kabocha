# Parity rows for own-taudit-crates-pumpkin-core-src-en-8f1a6a

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-en-8f1a6a** [partial/OWN-OPEN] own audit: 18 open, 6 fixed, 2 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-en-8f1a6a.md) docs/tracker/taudit-crates-pumpkin-core-src-en-8f1a6a.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)

## Result (third pass final)

Re-verified every STILL OPEN row against decompiled vanilla
(`.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/arrow/{AbstractArrow,Arrow,SpectralArrow}.java`)
and re-read the Rust call sites (`entity/projectile/arrow.rs`,
`item/items/projectile_weapon.rs`, `pumpkin-inventory/src/player/player_inventory.rs`).
No new code fix was needed: the one tractable row (30) was fixed in the prior pass and is
present in the tree, while the rest are already vanilla-correct, client-only effects, or a
whole missing engine subsystem. Every originally STILL OPEN row now has a final verdict.

| Row | Final verdict | Evidence |
|---|---|---|
| 29 (enchant ordering, M) | VERIFIED | `AbstractArrow.java:466-474` applies `modifyDamage` to `baseDamage` before `pow * baseDamage` / `ceil`; `projectile_weapon.rs` folds the same target-independent Power adjustment into `base_damage` at fire time, and `arrow.rs:746-748` computes `ceil(clamp(velocity * base_damage))`. Smite/Bane are target-dependent enchant-system concerns, not this row. |
| 30 (pickup `canHold`, S) | FIXED (confirmed present) | `AbstractArrow.java:707` gates the pickup on `canHold(pickupItem) > 0`; `arrow.rs:684-698` now gates `PlayerPickupArrowEvent` on `!stack.is_empty() && player.inventory.can_hold(&stack) > 0`, reusing the same stack for the insert. `can_hold` lives in `pumpkin-inventory/src/player/player_inventory.rs` with a pure unit test `can_hold_reports_capacity`. |
| 33 (tipped trail / potion puff, S) | VERIFIED | `Arrow.java:88-99` emits the trail only in the `isClientSide()` branch; the server branch only broadcasts entity event 0 and swaps to a plain arrow, both done in `decay_exposed_potion` (`arrow.rs:423-446`). The vanilla client renders the trail from synced ID_EFFECT_COLOR. |
| 34 (crit particles, S) | VERIFIED | `AbstractArrow.java:246-259` emits 4 CRIT particles in the airborne branch via `Level.addParticle`, which is a no-op on a dedicated server (ServerLevel uses a no-op particle engine); the client renders the trail from the synced crit flag set by `set_critical`. No server behaviour to replicate. |
| 35 (spectral trail, S) | VERIFIED | `SpectralArrow.java:36-41` emits the EFFECT particle only when `level().isClientSide()`. Client render only; vanilla server emits nothing. |
| 36 (sound/weapon NBT, S) | VERIFIED | `AbstractArrow.java:437-439,663,679` stores `SoundEvent` and clears `firedFromWeapon` in `onItemBreak`, but no Rust arrow sets a non-default hit sound (`Sound::EntityArrowHit` is hardcoded at `arrow.rs:803-807`) and there is no Rust `onItemBreak` hook, so persisting a sound is a no-op; weapon NBT is already persisted. No observable server-side gap. |
| 38 (no-physics owner pickup, S) | VERIFIED | `AbstractArrow.java:703-704` checks only `(isInGround() || isNoPhysics()) && shakeTime <= 0` and `tryPickup:724-729` has no owner branch; `arrow.rs:656-720` matches. |
| 39 (`last_hurt_mob`, M) | BLOCKED-DATA | Missing engine subsystem: `LivingEntity.last_hurt_mob` (the "victim" field) plus every consumer. Vanilla `AbstractArrow.java:497-499` calls `livingOwner.setLastHurtMob(entity)`; consumers include `world/entity/ai/goal/target/OwnerHurtTargetGoal.java:28-43`, `world/entity/animal/fox/Fox.java:129-130`, and `Mob.java:1619`. Pumpkin `LivingEntity` has only `last_hurt_by_mob` (the attacker), so this needs the new field plus all AI consumers. |

No code changed in this pass (the only in-tree fixes were already landed; the single genuinely
missing feature is a whole subsystem, reported BLOCKED-DATA). No new pure-test seam exists for
these rows; row 30 already ships a pure test. No cargo/git run (worker shell is denied), so prior
code edits remain UNVERIFIED by compilation in this pass.
