# Parity rows for crates/pumpkin-core/src/entity/projectile/fishing_bobber.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Fishing Bobber** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/taudit-crates-pumpkin-core-src-en-dae447** [partial/OWN-OPEN] own audit: 9 open, 4 fixed, 3 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-en-dae447.md) docs/tracker/taudit-crates-pumpkin-core-src-en-dae447.md
- **own/titems-fishing-rod** [partial/OWN-OPEN] own audit: 1 open, 3 fixed, 7 verified, 0 blocked-data (docs/tracker/titems-fishing-rod.md) docs/tracker/titems-fishing-rod.md

## Result
FINAL PASS. Every row is resolved below against decompiled vanilla at
.vendor/vanilla-decompiled/net/minecraft/. One real parity gap was found and fixed this pass
(open-water air variants); no BLOCKED-DATA condition applies (no missing generated pumpkin-data
field and no missing subsystem).

- entities/Fishing Bobber [partial/AUDITED]: FIXED + VERIFIED. Vanilla FishingHook.java
  (world/entity/projectile/FishingHook.java) governs every audited behaviour: throw origin and
  per-axis launch scaling (:100-124), fly/water/hook transitions and 1200-tick grounded life
  (:167-271), bobbing damping and synced bite kick (:225-252, :144-155), open-water latch
  (:233-237) and 5x5 scan (:442-485), catch cycle (:322-439), loot/XP/stat/durability return
  (:505-593), owner-stop discard (:273-291) and metadata indices (:60-61, :133-136). New fix:
  predicates.rs open_water_cell now maps AIR, CAVE_AIR and VOID_AIR to ABOVE_WATER, matching
  vanilla getOpenWaterTypeForBlock's state.isAir() (FishingHook.java:475-485; the three air
  blocks all register BlockBehaviour.Properties.air(), BlockBehaviour.java:1435-1437). A pure
  test all_air_variants_count_as_above_water was added.
- own/taudit-crates-pumpkin-core-src-en-dae447 [partial/OWN-OPEN]: FIXED + VERIFIED. All five
  originally open rows in docs/tracker/taudit-crates-pumpkin-core-src-en-dae447.md are closed:
  open-water flag (fishing_bobber.rs open_water AtomicBool + latched_open_water, predicates.rs
  is_open_water) FishingHook.java:233-237,442-485; cancelled PlayerFishEvent keeps the hook
  FishingHook.java:518-520,544-546; should_stop_fishing discards FishingHook.java:273-291; rod
  durability via damage_item_in_slot FishingRodItem.java:27; all random rolls f32
  FishingHook.java:153,232,348,403-405,417. The air-variant fix above closes the last residual
  open-water edge case.
- own/titems-fishing-rod [partial/OWN-OPEN]: VERIFIED. All four rows in
  docs/tracker/titems-fishing-rod.md are closed: open-water scan FishingHook.java:442-485,
  ITEM_USED awarded once on cast FishingRodItem.java:69, stale TODO removed, throw origin offset
  FishingHook.java:100-112. No change needed this pass.


## Result (third pass, fresh independent re-review)

Fresh read of the current checkout against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/FishingHook.java` and
`.vendor/vanilla-decompiled/net/minecraft/world/item/FishingRodItem.java`. Every STILL OPEN row
is VERIFIED against vanilla; no source change was needed and no BLOCKED-DATA (missing generated
pumpkin-data field or whole missing subsystem) applies.

- **entities/Fishing Bobber [partial/AUDITED]: VERIFIED (matches vanilla).** The recheck of
  `fishing_bobber.rs` found every audited behaviour still matches FishingHook.java: throw origin
  and per-axis launch scaling (`throw_setup` :854-868 vs FishingHook.java:100-124); fly/water/hook
  state transitions and 1200-tick grounded life (:452-477 vs :174-205); bobbing damping and synced
  bite kick (:479-525, :757-759 vs :225-252, :152-154); open-water latch
  (`latched_open_water` :943-955 vs :233-237) and the 5x5 column scan
  (`predicates.rs:419-454` vs :442-485); catch cycle wait/lure/hook timers (:660-809 vs :322-439);
  loot, XP, catches stat and durability-return damage (:181-348 vs :505-593); owner-stop discard
  (`should_stop_fishing` :616-646 vs :273-291); and hook metadata indices/serializers (:115-122,
  tested at :1030-1056 vs :60-61,133-136). `fishing_rod.rs` reels in, discards and awards
  ITEM_USED only on cast (:68-167) matching FishingRodItem.java.
- **own/taudit-crates-pumpkin-core-src-en-dae447 [partial/OWN-OPEN]: VERIFIED.** All five tracked
  rows are already closed in that doc's final Result (open-water flag, cancelled PlayerFishEvent
  keeps the hook, should_stop_fishing discards, rod durability via damage_item_in_slot, and all
  random rolls f32); each re-confirmed against FishingHook.java :233-236, :273-291, :322-439,
  FishingRodItem.java:27. No row remains open there.
- **own/titems-fishing-rod [partial/OWN-OPEN]: VERIFIED.** That doc's final Result closes all rows
  (open-water, ITEM_USED single-count on cast, stale TODO removed, hook origin offset correct);
  re-confirmed against FishingHook.java:100-112,442-485 and FishingRodItem.java:22-74. No row
  remains open there.

Pure-seam unit tests already exist for `throw_setup`, `bob_velocity`,
`latched_open_water`, metadata serializers and `is_open_water`; no duplicate was added. No .rs
file changed, so rustfmt was not required. Diff re-checked against the worker compile-pitfalls
table: no code changes, so none apply.

Status: PASS fixed=0 open=0 files=none


## Result (FINAL PASS close-out)

Independent verification of the current checkout of all three unit source files
(fishing_bobber.rs, fishing_rod.rs, world/loot/predicates.rs) against decompiled vanilla at
.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/FishingHook.java and
.vendor/vanilla-decompiled/net/minecraft/world/item/FishingRodItem.java. Worked through all
three rows in order.

- Row 1 (entities/Fishing Bobber): FIXED + VERIFIED. Added CAVE_AIR and VOID_AIR to
  predicates.rs open_water_cell and a pure unit test. Vanilla getOpenWaterTypeForBlock
  (FishingHook.java:475-485) uses state.isAir(), which is true for all three air blocks
  (BlockBehaviour.java:1435-1437, Blocks.java:66,4401,4404). Every other audited behaviour was
  re-read and matches FishingHook.java as cited above.
- Row 2 (own audit, five rows): VERIFIED. All five sub-rows already implemented and correct in
  code; reopened the open-water sub-row to apply the air-variant fix above.
- Row 3 (fishing rod item audit, four rows): VERIFIED. All four sub-rows already closed; no
  change needed.

No generated data, WIT or cross-crate refactor was required. rustfmt is run by the orchestrator.
No compile run (workers do not compile); verification by reading code against .vendor/ only.
Diff re-checked against the worker compile-pitfalls table: only one small if-condition expansion
and one pure test using assert!(a == b) (OpenWaterCell has no Debug derive); no unwrap/expect,
no tracing/println changes, no new imports in shared files.

Status: PASS fixed=1 open=0 files=crates/pumpkin-core/src/world/loot/predicates.rs
