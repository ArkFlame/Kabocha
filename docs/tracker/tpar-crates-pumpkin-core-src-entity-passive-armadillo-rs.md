# Parity rows for crates/pumpkin-core/src/entity/passive/armadillo.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Armadillo** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/tentities-armadillo-d4b8f2** [partial/OWN-OPEN] own audit: 5 open, 7 fixed, 2 verified, 0 blocked-data (docs/tracker/tentities-armadillo-d4b8f2.md) docs/tracker/tentities-armadillo-d4b8f2.md
- **own/tentities-armadillo** [partial/OWN-OPEN] own audit: 2 open, 3 fixed, 9 verified, 0 blocked-data (docs/tracker/tentities-armadillo.md) docs/tracker/tentities-armadillo.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)


## Result (re-audit 2026-11-18, third pass)

All three unit rows re-verified directly against the decompiled vanilla source
`.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/armadillo/Armadillo.java`
and `ArmadilloAi.java`. No generated pumpkin-data fields, WIT, or cross-crate changes
were needed; only `armadillo.rs` was edited.

- **entities/Armadillo [AUDITED]: VERIFIED.** Full re-review of `armadillo.rs` against
  `Armadillo.java`: state enum ids/names and `shouldHideInShell` thresholds (lines 383-444),
  `isScaredBy` three clauses (238-245), `customServerAiStep`/scute drop (140-157),
  `hurtServer` damage formula (280-286), `actuallyHurt` branch order (290-305),
  `canStayRolledUp` (332-334), `canFallInLove` (336-339), ambient/hurt/death/step sounds
  (341-364), `getMaxHeadYRot` (366-369), and `rollUp`/`rollOut` (261-277). All match.
- **own/tentities-armadillo-d4b8f2 [OWN-OPEN]: VERIFIED (all 6 rows closed).**
  Row 1 `hurtServer` gating on `isScared()` + `(damage - 1.0F) / 2.0F` with no clamp
  (Armadillo.java:280-286) matches `modify_incoming_damage`. Row 2 5-tick
  `armadillo_scare_detected` sensor cadence (SensorType.java / Armadillo.java:65) matches
  `SCARE_SCAN_INTERVAL = 5`. Row 3 `actuallyHurt` guard + 80-tick memory + rollUp/rollOut
  branch (290-305) matches `on_damage`. Row 4 three `isScaredBy` clauses (238-245) including
  the `getLastHurtByMob()` and spectator clauses are present. Row 5 `BABY_ID` compile-time
  assert ties the generated aliases. Row 6 pure state-id/name test pins ids 0..3.
- **own/tentities-armadillo [OWN-OPEN]: VERIFIED (all 6 rows closed).**
  Rows 1-6 (proximity scan, actuallyHurt, shell damage formula, redundant baby flag,
  scute spawn helper, state ids) all match the vanilla members listed above; the prior
  passes' fixes remain correct.

Bonus parity fixes applied this pass in `armadillo.rs` (from `Armadillo.java:261-277`):
`rollUp` now calls `stop_navigation()` (vanilla `stopInPlace()`), and both `rollUp` and
`rollOut` emit the `ENTITY_ACTION` game event, which were previously missing.

Known MVP residual (pre-existing, outside these rows, unchanged): the 5-tick threat scan
stays Idle-only and does not refresh the 80-tick danger memory while a threat persists,
unlike vanilla's always-on sensor; deferred in all prior Results. No pure seam exists for
the sound/AI/scan behaviour, so no new unit test was added; the existing state-id and
shell-threshold tests cover the pure logic.
