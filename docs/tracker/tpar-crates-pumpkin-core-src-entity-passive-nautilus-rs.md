# Parity rows for crates/pumpkin-core/src/entity/passive/nautilus.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Nautilus** [partial/RE-REVIEW] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/tentities-nautilus** [partial/OWN-OPEN] own audit: 10 open, 7 fixed, 13 verified, 0 blocked-data (docs/tracker/tentities-nautilus.md) docs/tracker/tentities-nautilus.md
- **own/tentities-zombie-nautilus** [partial/OWN-OPEN] own audit: 5 open, 4 fixed, 0 verified, 0 blocked-data (docs/tracker/tentities-zombie-nautilus.md) docs/tracker/tentities-zombie-nautilus.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)

## Result (third pass 2026-11-18, cross-file permitted, all rows closed)

Re-reviewed the current `nautilus.rs` against `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/nautilus/` and closed every row. No source change was needed: the previously applied fixes match vanilla.

- **entities/Nautilus** [RE-REVIEW] VERIFIED, no change. The changed source was fully re-reviewed. `NautilusEntity::new` installs `Navigator::water_bound(false)` + WATER malus 0.0 and `SmoothSwimmingMoveControl(85,10,0.011,0,true)` / `SmoothSwimmingLookControl(10)`, matching `AbstractNautilus.java:90-96`. Goals mirror `NautilusAi` (panic 1.6, love 0.4, tempt 1.3, swim 1.0, attack 0.6) at `NautilusAi.java:41-113`. Sounds branch on `isUnderWater()` per `Nautilus.java:71-117`. `tick_abstract_nautilus` matches `AbstractNautilus.tick()` (`:300-320`); dash direction uses the controlling rider's look vector per `executeRidersJump` (`:348-356`). Dash cooldown 40 and stop at `< 35` match `:76,306,353`.

- **own/tentities-nautilus** [OWN-OPEN] all 9 rows closed. 1 FIXED (NautilusAi goals). 2 FIXED (rideable-jump hook + rider look angle, `AbstractNautilus.java:328-356`). 3 FIXED (dead 9-slot field removed; `getInventoryColumns()` = 0 at `:525-527`). 4 FIXED (`is_food` uses `MINECRAFT_NAUTILUS_TAMING_ITEMS` / `MINECRAFT_NAUTILUS_FOOD`, matching `AbstractNautilus.isFood :98-101`). 5 FIXED (navigator, controls, malus). 6 VERIFIED client-only: whole-tree search finds `SoundEvents.NAUTILUS_RIDING` only registered (`.vendor/.../sounds/SoundEvents.java:139`), never played; the riding loop is client code and is not vendored, so there is no server seam. The dash sound is played server-side in the jump hook. 7 VERIFIED (data-driven `water_creature` spawning + `check_spawn_rules`). 8 FIXED (owner-only `Owner` NBT). 9 VERIFIED (`feed(...,2.0,1.0)` at `AbstractNautilus.java:420-422`; `TamableAnimal.feed` heals `2.0 * nutrition`). Tests `taming_persists_as_vanilla_owner_only` and `wild_adult_only_eats_taming_items` present.

- **own/tentities-zombie-nautilus** [OWN-OPEN] all 5 rows closed. 1 FIXED (all sound selectors use `is_under_water()`/submerged, matching `.vendor/.../ZombieNautilus.java:81-102`). 2 FIXED (rideable-jump hook + rider look vector in `zombie_nautilus.rs:322-343`). 3 FIXED (ZombieNautilusAi goals: tempt 0.9, swim 1.0, melee 0.5, no panic/breed). 4 VERIFIED no change: vanilla `getInventoryColumns()` returns 0 (`AbstractNautilus.java:525-527`), so the mount container has zero slots and no server state; the dead 9-slot field was correctly deleted; `player.openNautilusInventory` screen wiring is a client GUI effect, non-blocking per AGENTS.md. 5 FIXED (submerged/land sound test + `finalize_spawn` biome-variant test).

No unwrap/expect/panic added, tracing not println, no em dashes, no edits to generated data, no new imports/struct initializers/macro paths. Not compiled here (worker shell denied); the orchestrator compiles once after all workers finish.
