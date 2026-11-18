# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-8f1a6a.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: arrow.rs

| `arrow.rs:556-558` | `SpectralArrow#tick` emits an EFFECT particle every airborne tick | No spectral trail | Client-only particle, same call | S | STILL OPEN (spectral trail particle; S, not attempted) |
| `arrow.rs:427-445` | `addAdditionalSaveData` also stores `SoundEvent`, and `onItemBreak` clears `firedFromWeapon` | Fixed hit sounds; broken bow still leaves the weapon set | Persist the sound event; clear the weapon on item break if that hook exists | S | STILL OPEN (sound/weapon NBT; S, not attempted) |
| `arrow.rs:602-614` (`playerTouch`) | pickup when `ALLOWED`, or `CREATIVE_ONLY && instabuild`, or `isNoPhysics() && ownedBy(player)` | Only the `ArrowPickup` value is checked, so a no-physics arrow owned by the touching player cannot be retrieved | Add the `no_physics && owned_by(player)` case | S | STILL OPEN (no-physics owner pickup; S/M, not attempted) |
STILL OPEN (row 29, M): vanilla applies `modifyDamage` to the base damage before `pow * base` and `ceil` (`AbstractArrow.java:466-474`); Rust applies the enchant after the ceil inside `damage_with_context`. Matching needs restructuring the damage path, not a local change.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
