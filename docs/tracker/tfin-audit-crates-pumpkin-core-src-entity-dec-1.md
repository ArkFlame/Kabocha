# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-entity-dec.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: area_effect_cloud.rs armor_stand.rs splash_potion.rs

All rows below are now closed in the Final Result section at the end of this file.
Editable basenames for this unit: armor_stand.rs, splash_potion.rs. No source edit was
needed in the final pass; the only non-fixable remainder is Row 3's WASM surface, which
is BLOCKED-DATA (WIT cancelled:bool + drops fields require codegen regeneration).

## Result
FINAL PASS per-row results (2026-11-18). Reference:
`.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/ArmorStand.java` (Paper 1.21
decompiled) plus `.../world/level/Explosion.java`. No source edit was needed this pass: every
row fixable in this checkout is already present in armor_stand.rs / splash_potion.rs and matches
the decompiled call chain; re-read each file before verification.

1. Markers still have physics / fall: FIXED, VERIFIED. `get_gravity` returns 0.0 for markers
   (armor_stand.rs:832), so `LivingEntity::get_effective_gravity` never accelerates a marker.
   Vanilla `hasPhysics() = !isMarker() && !isNoGravity()` (ArmorStand.java:129-130) gates
   `travel` (ArmorStand.java:532) and `readAdditionalSaveData` sets `noPhysics = !hasPhysics()`
   (ArmorStand.java:188). The travel/piston-push hooks have no EntityBase seam; markers have
   zero-size dimensions, so collisions/pushing are not observable.
2. Explosion immunity: FIXED, VERIFIED. `ignore_explosion` returns
   `!explosion.should_affect_blocklike_entities() || self.is_immune_to_explosion()`
   (armor_stand.rs:639-640), dispatched from world/explosion.rs; `is_immune_to_explosion`
   returns `is_invisible()` (armor_stand.rs:634-635). Vanilla
   `ignoreExplosion(Explosion) = !explosion.shouldAffectBlocklikeEntities() || isInvisible()`
   (ArmorStand.java:603-604). `should_affect_blocklike_entities` mirrors
   `Explosion.BlockInteraction` (world/explosion.rs:330).
3. Paper plugin events: core FIXED, VERIFIED; WASM surface BLOCKED-DATA.
   `PlayerArmorStandManipulateEvent` fires cancellably in `swap_item` (armor_stand.rs:414-423).
   Core `EntityDeathEvent` is `#[cancellable]` with `cancelled: false` from `new()`
   (crates/pumpkin-core/src/plugin/api/events/entity/entity_death.rs:7-25) and
   `broken_by_anything` returns before equipment removal/drops when cancelled
   (armor_stand.rs:282-293), matching vanilla `postDeathEventTasks` being skipped on a
   cancelled death event (ArmorStand.java:503,513-515).
   BLOCKED-DATA: the WIT records `entity-death-event-data` (pumpkin-plugin-wit/v0.1/
   event.wit:414-417) and `entity-death-event` (pumpkin-plugin-wit/v0.2/event.wit:493-496)
   expose only `entity-id: s32` and `dropped-exp: s32`. The exact missing generated fields are
   `cancelled: bool` and a `list<item-stack>` drops list. Adding them requires WIT edits plus
   `cargo run --locked -p pumpkin-codegen -- wit` regeneration of the SDK/host bindings
   (wit_bindgen::generate!), a PLUGIN_API_VERSION bump, and a codegen run the worker cannot
   perform; generated bindings must not be hand-edited. Wasm plugins therefore still cannot
   cancel the death event or read its drops. Out of this unit's editable basenames
   (armor_stand.rs, splash_potion.rs).
4. skipAttackInteraction / may_interact: FIXED, VERIFIED. The player-attacker branch checks
   `is_in_spawn_protection` + mayBuild (armor_stand.rs:800), matching vanilla
   `skipAttackInteraction` = `source instanceof Player && !level.mayInteract(player, blockPosition())`
   (ArmorStand.java:713-714).
5. Potion immunity: FIXED, VERIFIED. `is_affected_by_potions()` returns false
   (armor_stand.rs:644-645), honoured by splash_potion.rs:356 and area_effect_cloud.rs:409;
   vanilla `isAffectedByPotions()` = false (ArmorStand.java:742).
6. Paper no-tick equipment: FIXED, VERIFIED. `read_can_tick_override` + `tick` skip present
   (armor_stand.rs:605-607,614-618) with pure round-trip unit tests (armor_stand.rs:979-985).
7. Client-only parity: no action needed, as documented in the source tracker.

No new pure unit-test seam exists for the state-dependent fixes; the existing pure tests
`paper_can_tick_override_nbt_round_trip` and
`read_flags_keeps_the_base_plate_unless_no_base_plate_is_true` remain. No source file was
edited this pass. Re-read every relevant diff against the Worker compile pitfalls table: full
paths for `Explosion` and `EntityDeathEvent`, no new struct literals with missing fields,
`const fn` on the pure helper, no unwrap/expect/panic by this unit, no em dashes.

Status: PASS fixed=0 open=1 files=none (Row 3 WASM surface BLOCKED-DATA; needs WIT cancelled:bool
and list<item-stack> drops fields plus pumpkin-codegen regeneration, outside editable files).
