# Parity rows for crates/pumpkin-core/src/entity/passive/axolotl.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Axolotl** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/taudit-crates-pumpkin-core-src-en-f50f1a** [partial/OWN-OPEN] own audit: 6 open, 1 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-en-f50f1a.md) docs/tracker/taudit-crates-pumpkin-core-src-en-f50f1a.md
- **own/tentities-axolotl-4ff0f4** [partial/OWN-OPEN] own audit: 4 open, 4 fixed, 0 verified, 0 blocked-data (docs/tracker/tentities-axolotl-4ff0f4.md) docs/tracker/tentities-axolotl-4ff0f4.md

## Result

THIRD PASS (re-review). All three rows were re-verified directly against the decompiled vanilla
tree at `.vendor/vanilla-decompiled/net/minecraft/`. The reviewed source has not regressed:
every audited behaviour still matches vanilla, so no source change was required and there is
nothing to rustfmt.

- **entities/Axolotl [partial/AUDITED] - VERIFIED (matches vanilla).** `Axolotl.java:396-408`
  snapshots `getHealth()` before `super.hurtServer` and checks `damage < currentHealth`, RNG draws,
  `source.getEntity() != null || source.getDirectEntity() != null`, in water, not already playing
  dead; `axolotl.rs:158-166` (`plays_dead_when_hurt`) mirrors it and `axolotl.rs:615-654` captures
  pre-hit health/amount in `mob_hurt_server` and consumes them from `post_damage_with_context` and
  `on_damage_with_context` behind `play_dead_roll_pending`. `Axolotl.java:386-387` returns
  `AmphibiousPathNavigation` -> `Navigator::amphibious(false)` (`axolotl.rs:238`), with
  `PathType.WATER` malus 0.0 (`:118`, `axolotl.rs:240-244`). Bucket NBT (`:427-449` -> `axolotl.rs:412-452`)
  writes/erases Age, AgeLocked and gated HuntingCooldown. `onStopAttacking` (`:467-477`) uses the
  expanded AABB `inflate(20.0)` -> `expand_all(PLAYER_REGEN_DETECTION_RANGE)` (`axolotl.rs:334-348`).
  `applySupportingEffects` (`:479-484`) uses 100 + previous, cap 2400, amplifier 0 ->
  `supporting_regeneration_duration` (`axolotl.rs:174-200`). `canBeSeenAsEnemy` (`:463-465`) ->
  `mob_can_be_seen_as_enemy` (`axolotl.rs:656-659`).

- **own/taudit-crates-pumpkin-core-src-en-f50f1a [partial/OWN-OPEN] - VERIFIED.** Its own Result
  records all six STILL OPEN rows resolved: causing-entity check (row 14), pre-hit health (row 15),
  support regen range (row 22), blocked-hit play-dead roll (row 23) and regen duration (row 24),
  all matching the vanilla paths cited above and in that doc. No new gap found on re-review.

- **own/tentities-axolotl-4ff0f4 [partial/OWN-OPEN] - VERIFIED.** Its own Result (sixth pass) records
  all four re-review rows and the navigator fix resolved: regen 200/0
  (`axolotl/axolotl/PlayDead.java:32`), bucket NBT keys (`Axolotl.java:427-450`), roll timing/health
  (`Axolotl.java:396-408`), hunting cooldown 2400 (`AxolotlAi.java:150-159`), and the amphibious
  navigator (`Axolotl.java:386-387`, `AmphibiousPathNavigation.java:17`). No new gap found.

No STILL OPEN rows remain. Source file `axolotl.rs` was already correct in all three cases; no
code edit was needed, so no rustfmt run was required.

