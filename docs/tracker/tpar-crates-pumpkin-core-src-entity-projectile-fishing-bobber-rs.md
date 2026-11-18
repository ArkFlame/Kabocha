# Parity rows for crates/pumpkin-core/src/entity/projectile/fishing_bobber.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Fishing Bobber** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/taudit-crates-pumpkin-core-src-en-dae447** [partial/OWN-OPEN] own audit: 9 open, 4 fixed, 3 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-en-dae447.md) docs/tracker/taudit-crates-pumpkin-core-src-en-dae447.md
- **own/titems-fishing-rod** [partial/OWN-OPEN] own audit: 1 open, 3 fixed, 7 verified, 0 blocked-data (docs/tracker/titems-fishing-rod.md) docs/tracker/titems-fishing-rod.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)


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
