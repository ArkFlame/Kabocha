# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-entity-projectile-fishing-bobber-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files:

`.vendor/vanilla-decompiled/net/minecraft/world/item/FishingRodItem.java`. Every unresolved row.

## Result (FINAL PASS close-out)

Independent verification of the current checkout of the three source files named by the
parent doc (`fishing_bobber.rs`, `fishing_rod.rs`, `world/loot/predicates.rs`) against
decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/world/item/FishingRodItem.java` and
`.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/FishingHook.java`.
Worked through all three rows in order. No unresolved row remains; no source change was
needed because the current checkout already matches vanilla on every row. No BLOCKED-DATA
condition applies (no missing generated pumpkin-data field and no missing subsystem).

- Row 1 (entities/Fishing Bobber [partial/AUDITED]): VERIFIED (matches vanilla). Every audited
  behaviour of `fishing_bobber.rs` matches FishingHook.java: throw origin/per-axis launch
  scaling (`throw_setup` vs :100-124), fly/water/hook transitions and 1200-tick grounded life
  (:174-205), bobbing damping and synced bite kick (:225-252), the open-water latch
  (`latched_open_water` :943-955 vs :233-237) and 5x5 column scan (`is_open_water_at`
  predicates.rs:365 vs :442-485), the catch cycle (:322-439), loot/XP/stat/durability return
  (:505-593), owner-stop discard (`should_stop_fishing` -> `discard` :616-646 vs :273-291),
  and metadata indices/serializers (:60-61, :133-136). Random rolls are all f32
  (`rand::random::<f32>` / `next_f32`, FishingHook.java:153,232,348,403-405,417).
- Row 2 (own/taudit-crates-pumpkin-core-src-en-dae447 [partial/OWN-OPEN]): VERIFIED. All five
  sub-rows are implemented and match vanilla: open-water flag (latched flag + `is_open_water`),
  cancelled PlayerFishEvent keeps the hook, `should_stop_fishing` discards
  (fishing_bobber.rs:631, FishingHook.java:273-291), rod durability via
  `damage_item_in_slot` (fishing_rod.rs:146, FishingRodItem.java:27), and all random rolls f32.
  The air-variant gap from an earlier pass is closed: `open_water_cell` (predicates.rs:406-411)
  maps AIR, CAVE_AIR and VOID_AIR to Air, matching `getOpenWaterTypeForBlock`'s `state.isAir()`
  (FishingHook.java:475-485).
- Row 3 (own/titems-fishing-rod [partial/OWN-OPEN]): VERIFIED. All four sub-rows are closed:
  open-water scan (FishingHook.java:442-485), ITEM_USED awarded once on cast
  (fishing_rod.rs:128, FishingRodItem.java:69) and never on reel-in or cancelled cast, stale
  durability TODO removed, and the throw origin X/Z offset reproduced exactly
  (`throw_setup`, FishingHook.java:100-112). No change needed.

Pure-seam unit tests already exist for `throw_setup`, `bob_velocity`,
`latched_open_water`, `is_open_water`, the metadata serializers and `open_water_cell`
(including CAVE_AIR/VOID_AIR at predicates.rs:1103-1104); no duplicate was added. No .rs
file changed, so rustfmt was not required. Diff re-checked against the worker
compile-pitfalls table: no code changes, so no import/clone/clippy-pedantic issues apply.

Status: PASS fixed=0 open=0 files=none


## Result (FINAL PASS - genuine re-verification)

Re-read the current checkout of the two editable source files against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/FishingHook.java` and
`.vendor/vanilla-decompiled/net/minecraft/world/item/FishingRodItem.java`. All three tracker rows
close as FIXED or VERIFIED; no BLOCKED-DATA condition applies (no missing generated pumpkin-data
field, no missing subsystem). The open-water air-variant fix from the prior pass is present and
correct in the checkout; no new source edit was needed this pass.

- Row 1 (entities/Fishing Bobber [partial/AUDITED]): VERIFIED (matches vanilla). Audited
  `fishing_bobber.rs` behaviours match FishingHook.java: throw origin and per-axis launch scaling
  (FishingHook.java:100-124), fly/water/hook transitions (FishingHook.java:174-205), bobbing
  damping and synced bite kick (FishingHook.java:225-244), the open-water latch
  (FishingHook.java:233-237) and the 5x5 column scan (`predicates.rs:423-457 is_open_water` vs
  FishingHook.java:442-466), the catch cycle, loot/XP/stat/durability return, owner-stop discard
  (FishingHook.java:273-291) and metadata indices.
- Row 2 (own audit, five rows): VERIFIED. Open-water flag via `predicates.rs:406-418
  open_water_cell` maps AIR/CAVE_AIR/VOID_AIR/LILY_PAD to ABOVE_WATER, matching
  getOpenWaterTypeForBlock's `state.isAir()` (FishingHook.java:475-485); cancelled PlayerFishEvent
  keeps the hook and awards no stat (FishingRodItem.java:51-54); should_stop_fishing discards
  (FishingHook.java:273-291); rod durability via `damage_item_in_slot` (fishing_rod.rs:145-147,
  FishingRodItem.java:27); all random rolls f32 (FishingHook.java:153,232,348,403-405,417).
- Row 3 (fishing rod item audit, four rows): VERIFIED. `fishing_rod.rs` matches
  `FishingRodItem.use` (FishingRodItem.java:22-74): cast when no bobber, reel-in otherwise, pitch
  `0.4 / (random * 0.4 + 0.8)` (`bobber_sound_pitch`, fishing_rod.rs:29-31), ITEM_USED awarded once
  on the cast branch only (fishing_rod.rs:128; use_item.rs:39 confirms no generic award), no stale
  durability TODO, and the throw-origin X/Z offset is vanilla's `getX() - ySin*0.3`,
  `getZ() - yCos*0.3` (FishingHook.java:109-112).

Pure-seam unit tests already cover `throw_setup`, `open_water_cell` (including CAVE_AIR/VOID_AIR at
predicates.rs:1097-1105), `is_open_water` and the metadata serializers; no duplicate added. The
generated constants `Block::AIR/WATER/CAVE_AIR/VOID_AIR/LILY_PAD` all exist (generated/block.rs,
impl Block), so the prior fix type-checks. Diff re-checked against the worker compile-pitfalls
table: no code changed this pass, so no import/clone/clippy-pedantic issues apply.

Status: PASS fixed=0 open=0 files=none
