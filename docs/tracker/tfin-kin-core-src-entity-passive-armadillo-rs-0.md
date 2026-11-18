# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-entity-passive-armadillo-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

All rows below closed. Per row: FIXED, VERIFIED (vanilla path), or BLOCKED-DATA.

## Result (FINAL PASS, 2026-11-18)

All three rows closed against the decompiled vanilla source
`.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/armadillo/Armadillo.java`
(and `ArmadilloAi.java`, `SensorType.java`). No generated pumpkin-data fields, WIT changes,
or cross-crate refactors were needed; only `armadillo.rs` was edited (one guard fix).

- **entities/Armadillo [AUDITED]: VERIFIED.** Every reviewed member in `armadillo.rs` matches
  vanilla: state enum ids/names and `shouldHideInShell` thresholds (Armadillo.java:383-407),
  `isScaredBy` three clauses (238-245), `customServerAiStep` / scute drop (140-157),
  `hurtServer` damage formula (280-286), `actuallyHurt` branch order (290-305),
  `canStayRolledUp` (332-334), `canFallInLove` (336-339), ambient/hurt/death/step sounds
  (341-364), `getMaxHeadYRot` (366-369), and `rollUp`/`rollOut` (261-277).
- **own/tentities-armadillo-d4b8f2 [OWN-OPEN]: VERIFIED (all 6 rows closed).** `hurtServer`
  gating on `isScared()` with `(damage - 1.0F) / 2.0F` and no clamp (280-286), the 5-tick
  `armadillo_scare_detected` sensor cadence (SensorType.java:26-28), `actuallyHurt` 80-tick
  memory + rollUp gate (290-305), the three `isScaredBy` clauses (238-245), the `BABY_ID`
  compile-time assert, and the pure state-id test all match.
- **own/tentities-armadillo [OWN-OPEN]: VERIFIED (all 6 rows closed).** Proximity scan (5-tick
  `get_all_at_box` probe), `actuallyHurt`, shell damage formula, redundant baby flag, scute
  spawn helper, and state ids all match the vanilla members cited above.

Fix applied this pass in `armadillo.rs`: `on_damage` now guards with
`!self.mob_entity.living_entity.is_dead_or_dying()` instead of `!self.get_entity().is_alive()`,
matching vanilla `actuallyHurt`'s `!this.isNoAi() && !this.isDeadOrDying()` (Armadillo.java:294).
The previous `is_alive()` check is `!is_removed()`, which let a lethal blow roll up a dying
armadillo; `is_dead_or_dying()` is `health <= 0 || dead` and correctly skips that reaction.

Known MVP residual (outside these three rows, unchanged, deferred in all prior passes): the
5-tick threat scan stays Idle-only and does not refresh the 80-tick danger memory while a
threat persists, unlike vanilla's always-on `armadillo_scare_detected` sensor; no pure seam
exists for AI/sound behaviour, so no new unit test was added beyond the existing state-id and
shell-threshold tests. No cargo/git run per unit rules; compilation and rustfmt are left to the
orchestrator.
