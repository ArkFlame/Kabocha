# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-entity-passive-axolotl-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: axolotl.rs

records all six STILL OPEN rows resolved: causing-entity check (row 14), pre-hit health (row 15),
No STILL OPEN rows remain. Source file `axolotl.rs` was already correct in all three cases; no
No STILL OPEN rows, no BLOCKED-DATA fields, no code edits required.

## Result

FINAL PASS - unit complete. Every row is closed as FIXED or VERIFIED against the decompiled
vanilla tree at `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/axolotl/`.
No source change was needed this pass: `axolotl.rs` already matches vanilla in every audited
behaviour, so there was nothing to rustfmt. Cargo is forbidden for workers, so compilation is
left to the orchestrator's single batch check.

- **entities/Axolotl [partial/AUDITED] - VERIFIED (matches vanilla).** `Axolotl.java:396-408`
  snapshots `getHealth()` before `super.hurtServer` and checks `damage < currentHealth`, RNG
  draws, `source.getEntity() != null || getDirectEntity() != null`, in water, not already
  playing dead; `plays_dead_when_hurt` (`axolotl.rs:158-166`) mirrors the condition order and
  RNG draws, and `mob_hurt_server`/`try_play_dead` (`:615-654`, `:353-382`) capture pre-hit
  health/amount and consume them from both `post_damage_with_context` and
  `on_damage_with_context` behind `play_dead_roll_pending`. `Axolotl.java:386-387` returns
  `AmphibiousPathNavigation` -> `Navigator::amphibious(false)` (`axolotl.rs:238`), with the
  `PathType.WATER` malus 0.0 (`Axolotl.java:118`, `axolotl.rs:240-244`). Bucket NBT
  (`Axolotl.java:427-450` -> `axolotl.rs:412-452`) writes/erases Age, AgeLocked and gated
  HuntingCooldown. `onStopAttacking` (`Axolotl.java:467-471`) uses the expanded AABB
  `getBoundingBox().inflate(20.0)` -> `expand_all(PLAYER_REGEN_DETECTION_RANGE)`
  (`axolotl.rs:334-348`). `applySupportingEffects` (`Axolotl.java:479-484`) uses 100 + previous,
  cap 2400, amplifier 0 -> `supporting_regeneration_duration` (`axolotl.rs:174-200`).
  `canBeSeenAsEnemy` -> `mob_can_be_seen_as_enemy` (`axolotl.rs:656-659`).

- **own/taudit-crates-pumpkin-core-src-en-f50f1a [partial/OWN-OPEN] - VERIFIED.** All six rows
  are closed: row 14 causing-entity check (`Axolotl.java:403`; `axolotl.rs:374`,
  `source.is_some() || cause.is_some()`), row 15 pre-hit health (`Axolotl.java:397`;
  `axolotl.rs:626-629`), row 22 support regen range (`Axolotl.java:471` inflate(20.0);
  `axolotl.rs:339` expand_all(20.0), a distance-sphere change would diverge from vanilla),
  row 23 blocked-hit play-dead roll (FIXED earlier via `try_play_dead` called from
  `post_damage_with_context`, test `a_blocked_hit_still_rolls_for_play_dead`), row 24 regen
  duration (`PlayDead.java:32` 200/0, `TOTAL_PLAYDEAD_TIME = 200`), plus the duplicated doc
  line removal.

- **own/tentities-axolotl-4ff0f4 [partial/OWN-OPEN] - VERIFIED.** All four rows and the navigator
  fix are closed: row 1 play-dead regen (`PlayDead.java:32` REGENERATION 200, 0; `ai.rs:40`,
  `ai.rs:357-359`), row 2 bucket NBT keys (`Axolotl.java:427-450`; `axolotl.rs:412-452`),
  row 3 roll timing/health (`Axolotl.java:396-408`; `axolotl.rs:615-654`, tests
  `play_dead_roll_uses_pre_hit_health_like_vanilla` and `a_blocked_hit_still_rolls_for_play_dead`),
  row 4 hunting cooldown (`AxolotlAi.java:150-159` 2400 on leaving FIGHT; `ai.rs:245-260`
  HUNTING_COOLDOWN = 2400 and `axolotl_attackables.rs:27-29` guard), and the navigator
  (`Axolotl.java:386-387`; `axolotl.rs:238`, test `navigation_is_amphibious_like_vanilla`).

No STILL OPEN rows remain. No BLOCKED-DATA fields. No source edits or rustfmt were needed this pass.
