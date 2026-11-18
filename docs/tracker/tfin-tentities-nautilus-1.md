# Final rows from docs/tracker/tentities-nautilus.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: natural_spawner.rs nautilus.rs sound.rs tag.rs

| # | Pumpkin | Vanilla | Status |
|---|---|---|---|
| 6 | `nautilus.rs` (absent) | `AbstractNautilus` tick riding sound | OPEN. `Sound::EntityNautilusRiding` is still never played; the vanilla interval and condition need a verified read of `AbstractNautilus.java` before porting (size S). |
| 7 | `world/natural_spawner.rs` (absent) | nautilus spawn placement | OPEN. Neither `Nautilus` nor `ZombieNautilus` is registered in the natural spawner; needs spawn group, weight and biome tags verified against vanilla (size M). |
| 8 | `nautilus.rs:258-278` | `TamableAnimal#addAdditionalSaveData` | OPEN. Hand-rolled taming NBT (`IsTame`/`DashCooldown`) remains; moving it to `TamableData` needs a check of whether vanilla persists the dash cooldown (size S). |
| 9 | `nautilus.rs:570-580` food heal | `TamableAnimal.feed` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/TamableAnimal.java`) and `AbstractNautilus.mobInteract` `feed(player, hand, stack, 2.0F, 1.0F)` (AbstractNautilus.java:420-423) | VERIFIED. Vanilla heals `healingFactor * nutrition` (2.0) when food is present, else `defaultHeal` (1.0). Pumpkin uses `2.0 * nutrition` with a `1.0` fallback, which matches. |

Status: verified=1 (row 9); open=3 (rows 6, 7, 8). No compile run.
