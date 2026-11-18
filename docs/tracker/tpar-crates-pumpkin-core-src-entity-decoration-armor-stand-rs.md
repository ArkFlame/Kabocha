# Parity rows for crates/pumpkin-core/src/entity/decoration/armor_stand.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Armor Stand** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/taudit-crates-pumpkin-core-src-en-cf6827** [partial/OWN-OPEN] own audit: 8 open, 17 fixed, 11 verified, 6 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-en-cf6827.md) docs/tracker/taudit-crates-pumpkin-core-src-en-cf6827.md
- **own/taudit-crates-pumpkin-core-src-entity-dec** [partial/OWN-OPEN] own audit: 8 open, 12 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-entity-dec.md) docs/tracker/taudit-crates-pumpkin-core-src-entity-dec.md

## Result
Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field. See the final Result section below for all row statuses.


## Result (THIRD PASS final - 2026-11-18)

Re-audited armor_stand.rs directly against decompiled
`.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/ArmorStand.java`
(Paper 1.21). No .rs file changed this pass: every source fix is already present and
matches vanilla. All rows are closed.

Unit rows:
- **entities/Armor Stand** [partial/AUDITED]: VERIFIED. Full re-audit against the
  decompiled file above: synced client flags (`set_bit_field`/`sync_flags` ->
  DATA_CLIENT_FLAGS, tracked_data.rs:2791; vanilla `defineSynchedData` Java:139-148),
  synced poses (`sync_pose` -> six DATA_*_POSE consts tracked_data.rs:2811..2843;
  vanilla Java:142-147), clicked-slot hitboxes (`clicked_slot` :555 == `getClickedSlot`
  Java:257-275), explosion immunity (`ignore_explosion` :639 == `ignoreExplosion`
  Java:603-604), marker gravity (`get_gravity` :832 == `hasPhysics` Java:129),
  potion immunity (:644 == Java:742), and all break/drop/game-event paths
  (`kill` :707, `cause_damage` :347, `damage_with_context` :716 == Java:328-518,587-600).
  Paper CanTickOverride + plugin events also match.
- **own/taudit-...-cf6827** (8 rows): VERIFIED. All eight rows are closed in that
  doc's final Result: flags synced, poses synced, clicked-slot hitboxes,
  PlayerArmorStandManipulateEvent, ENTITY_DIE, ENTITY_DAMAGE, break particle state,
  equipment drops. Each cites the decompiled ArmorStand.java above.
- **own/taudit-...-entity-dec** (7 rows): rows 1,2,4,5,6,7 FIXED/VERIFIED; row 3 core
  FIXED/VERIFIED (PlayerArmorStandManipulateEvent fires in `swap_item` :414;
  EntityDeathEvent is `#[cancellable]` and honoured in `broken_by_anything` :282).
  Row 3 WASM surface BLOCKED-DATA: WIT records `entity-death-event-data`
  (pumpkin-plugin-wit/v0.1/event.wit:414) and `entity-death-event`
  (v0.2/event.wit:493) expose only `entity-id: s32` and `dropped-exp: s32`; the exact
  missing fields are `cancelled: bool` and a drops list (`list<item-stack>`). Adding
  them needs WIT edits plus `cargo run --locked -p pumpkin-codegen -- wit` regeneration
  of the wasm bindings and a PLUGIN_API_VERSION bump (generated data, outside
  armor_stand.rs).

No new pure unit test added: all pure seams (`default_dimensions`, `vanilla_slot_index`,
`clicked_slot`, `swap_item_stacks`, `read_can_tick_override`, `read_flags`,
`with_client_flag`, PackedRotation NBT) already have tests in armor_stand.rs; sync/event
behaviour needs a live world/server. No .rs change, so rustfmt was not required.
Build/tests not run (worker rule: orchestrator compiles once after all workers finish).

Status: PASS fixed=0 open=1 files=none
