# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-entity-dec.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

vanilla `ignoreExplosion`'s invisible clause. STILL OPEN: `!explosion.shouldAffectBlocklikeEntities()`
shared seam reached from kill/burn/explosion, before the equipment lock is taken. STILL OPEN:
STILL OPEN (out of this unit, one-line reasons):
2. Explosion immunity: STILL OPEN (out of unit). Vanilla `ignoreExplosion(Explosion)` (ArmorStand.java:603-604)

## Result

FINAL PASS 2026-11-18. Re-verified every row against
`.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/ArmorStand.java` and
`.vendor/vanilla-decompiled/net/minecraft/world/level/Explosion.java` (Paper 1.21 decompiled)
and the current Pumpkin sources (armor_stand.rs, entity/mod.rs, world/explosion.rs,
area_effect_cloud.rs, splash_potion.rs, entity_death.rs, both WIT event.wit records).
No source file needed a code change this pass: every row implementable in this checkout is
already present and matches the decompiled call chain. Per-row terminal status:

1. Markers still have physics / fall: FIXED, VERIFIED. `get_gravity` returns 0.0 for markers
   (armor_stand.rs:832-835); vanilla `hasPhysics() = !isMarker() && !isNoGravity()` gates
   `travel` and `readAdditionalSaveData` sets `noPhysics = !hasPhysics()` (ArmorStand.java:129,188,532).
   `isIgnoringBlockTriggers() = isMarker()` (armor_stand.rs:629; ArmorStand.java:613).
2. Explosion immunity: FIXED, VERIFIED. `ignore_explosion` returns
   `!explosion.should_affect_blocklike_entities() || self.is_immune_to_explosion()`
   (armor_stand.rs:639-640), dispatched at world/explosion.rs:644;
   `should_affect_blocklike_entities` mirrors vanilla
   `Explosion.BlockInteraction#shouldAffectBlocklikeEntities` (world/explosion.rs:330-335).
   Vanilla `ignoreExplosion(Explosion) = !explosion.shouldAffectBlocklikeEntities() || isInvisible()`
   (ArmorStand.java:603-604). The default `EntityBase::ignore_explosion` delegates to
   `is_immune_to_explosion` (entity/mod.rs:496-497), so the seven existing implementors keep
   their behaviour; no signature refactor was needed.
3. Paper plugin events: core FIXED, VERIFIED; WASM surface BLOCKED-DATA. Core `EntityDeathEvent`
   is `#[cancellable]` with `cancelled: false` (entity_death.rs:7,23) and `broken_by_anything`
   returns before equipment removal/drops on cancel (armor_stand.rs:282-293), matching vanilla
   where `postDeathEventTasks` is skipped on a cancelled death event; the shared seam is reached
   from kill/burn/explosion before the equipment lock. `PlayerArmorStandManipulateEvent` fires
   cancellably in `swap_item` (armor_stand.rs:411-427). BLOCKED-DATA: the exact missing generated
   fields are `cancelled: bool` and a drops list (`list<item-stack>`) on the WIT records
   `entity-death-event` (pumpkin-plugin-wit/v0.2/event.wit:493) and `entity-death-event-data`
   (pumpkin-plugin-wit/v0.1/event.wit:414), which expose only `entity-id: s32, dropped-exp: s32`.
   The core `EntityDeathEvent` also carries no drops. Wiring them requires WIT edits plus
   regeneration of the `wit_bindgen::generate!` / `bindgen!` bindings and a `PLUGIN_API_VERSION`
   bump: generated data + public API, which cannot be completed by this worker without running
   codegen (shell denied), so wasm plugins still cannot cancel.
4. skipAttackInteraction / may_interact: FIXED, VERIFIED. Player-attacker branch checks
   `is_in_spawn_protection` + mayBuild (armor_stand.rs:794-802), matching vanilla
   `skipAttackInteraction = source instanceof Player && !level.mayInteract(player, blockPosition())`
   (ArmorStand.java:713-714).
5. Potion immunity: FIXED, VERIFIED. `is_affected_by_potions()` returns false (armor_stand.rs:644),
   honoured by splash_potion.rs:356 and area_effect_cloud.rs:409; vanilla `isAffectedByPotions()
   = false` (ArmorStand.java:742).
6. Paper no-tick equipment: FIXED, VERIFIED. `read_can_tick_override` + `tick` skip present
   (armor_stand.rs:605,614-618) with pure round-trip test `paper_can_tick_override_nbt_round_trip`
   (armor_stand.rs:979), mirroring Paper's `Paper.CanTickOverride` behaviour.
7. Client-only parity: no action needed, as documented in the parent tracker.

No source file edited this pass and no new pure test seam exists for the state-dependent fixes;
the existing pure tests remain. Re-read the relevant source and diff against the Worker compile
pitfalls table: full paths used for `Explosion`, no unwrap/expect/panic, no em dashes, no new
struct literals with missing fields.

Status: PASS fixed=0 open=1 files=none (all rows terminal: 1,2,4-7 FIXED/VERIFIED; row 3 core
FIXED with WASM surface BLOCKED-DATA requiring WIT fields `cancelled: bool` + drops `list<item-stack>`
and codegen/version bump, outside worker scope)
