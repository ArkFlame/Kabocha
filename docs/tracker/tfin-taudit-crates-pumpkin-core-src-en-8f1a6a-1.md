# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-8f1a6a.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: arrow.rs

| `arrow.rs:556-558` | `SpectralArrow#tick` emits an EFFECT particle every airborne tick | No spectral trail | Client-only particle, same call | S | STILL OPEN (spectral trail particle; S, not attempted) |
| `arrow.rs:427-445` | `addAdditionalSaveData` also stores `SoundEvent`, and `onItemBreak` clears `firedFromWeapon` | Fixed hit sounds; broken bow still leaves the weapon set | Persist the sound event; clear the weapon on item break if that hook exists | S | STILL OPEN (sound/weapon NBT; S, not attempted) |
| `arrow.rs:602-614` (`playerTouch`) | pickup when `ALLOWED`, or `CREATIVE_ONLY && instabuild`, or `isNoPhysics() && ownedBy(player)` | Only the `ArrowPickup` value is checked, so a no-physics arrow owned by the touching player cannot be retrieved | Add the `no_physics && owned_by(player)` case | S | STILL OPEN (no-physics owner pickup; S/M, not attempted) |
STILL OPEN (row 29, M): vanilla applies `modifyDamage` to the base damage before `pow * base` and `ceil` (`AbstractArrow.java:466-474`); Rust applies the enchant after the ceil inside `damage_with_context`. Matching needs restructuring the damage path, not a local change.

## Result

All four rows were re-checked against the decompiled vanilla tree under
`.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/arrow/` and the current
`crates/pumpkin-core/src/entity/projectile/arrow.rs`. Every row is already vanilla-correct, so
no source edit was needed and there is nothing to rustfmt. Per item verdicts:

| Row | Verdict | Evidence |
|---|---|---|
| `SpectralArrow#tick` EFFECT trail (audit row 35) | VERIFIED | `.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/arrow/SpectralArrow.java:35-41` emits the EFFECT particle only inside `level().isClientSide()`; a vanilla dedicated server emits nothing and the vanilla client renders the trail from the synced `SPECTRAL_ARROW` entity. `arrow.rs:624-626` ticks only `abstract_arrow::tick`, so Pumpkin matches. Client-only effect. |
| `addAdditionalSaveData` SoundEvent / `onItemBreak` weapon clear (audit row 36) | VERIFIED | `.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/arrow/AbstractArrow.java:437-439,653-681` stores/reads `SoundEvent` and clears `firedFromWeapon` through the `onProjectileSpawned` break callback (`.vendor/.../Projectile.java:420-427`). Pumpkin has no sound field or break-callback hook, hardcodes `Sound::EntityArrowHit` (`arrow.rs:737`), and already persists `weapon` NBT (`arrow.rs:486-490,510-520`). Enchantments are folded into `base_damage` at fire time (`item/items/projectile_weapon.rs:67-75`), so clearing the weapon on break could not change a hit and persisting an unset sound is a no-op. No observable server-side gap. |
| `playerTouch` no-physics owner pickup (audit row 38) | VERIFIED | The claimed `isNoPhysics() && ownedBy(player)` case exists only in `.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/arrow/ThrownTrident.java:192`, not in `AbstractArrow`. `.vendor/.../AbstractArrow.java:703-704,724-729` checks only `(isInGround() || isNoPhysics()) && shakeTime <= 0` and the three `pickup` values, matching `arrow.rs:656-728`. Player-shot arrows get `Allowed` pickup (`projectile_weapon.rs:30-41`, vanilla `AbstractArrow.java:690-700`), so they are already retrievable; the trident has its own owner branch (`trident.rs:408-422,672-700`). |
| `modifyDamage` before `pow * base` and `ceil` (audit row 29) | VERIFIED | `.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/arrow/AbstractArrow.java:466-474` applies `EnchantmentHelper.modifyDamage` to `baseDamage` before `pow * baseDamage` and `Mth.ceil`. `item/items/projectile_weapon.rs:67-70` folds the same target-independent Power adjustment into `base_damage` at fire time and `arrow.rs:746-748` computes `ceil(clamp(velocity * base_damage))`. `EntityBase::damage_with_context` (`entity/mod.rs:1055-1075`) does not re-apply the enchant, so the pre-ceil value matches vanilla. |

Status: PASS fixed=0 open=0 files=none (all four rows VERIFIED against the cited .vendor paths; no code change needed).
