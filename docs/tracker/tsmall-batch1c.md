# tsmall batch 1c: audit of batch 1b rows 1-11 and 15

Partial pass. Only rows 5, 6 and 8 were audited against vanilla. Rows 1-4, 7, 9-11 and 15 were not audited in this pass. No code changed; no cargo run.

## Rows

5. `taudit-crates-pumpkin-core-src-en-f50f1a.md:14` axolotl play-dead causing entity: STILL OPEN (divergence found). Axolotl.java:397-405 passes when `source.getEntity() != null || source.getDirectEntity() != null`. Rust `axolotl.rs` `HurtCheck.has_causing_entity` uses only `cause.is_some()`, so a direct entity without an owner does not count. Fix needs the direct source threaded into `on_damage_with_context`; not a one-line change, so left open.
6. `taudit-crates-pumpkin-core-src-en-f50f1a.md:15` axolotl play-dead health before hit: STILL OPEN (divergence found). Vanilla reads `currentHealth` before `super.hurtServer`. Rust `on_damage_with_context` runs after damage is applied and reads `living.health`, so the health ratio and `damage < health` use post-hit health. Its doc comment claims the opposite. Fix needs a pre-damage health snapshot; left open.
8. `taudit-crates-pumpkin-core-src-en-f50f1a.md:24` play-dead regeneration duration: VERIFIED for the play-dead timer. Axolotl.java:405 sets `PLAY_DEAD_TICKS` to 200 and `TOTAL_PLAYDEAD_TIME = 200` (Axolotl.java:75). Rust `axolotl.rs:611` sets `TOTAL_PLAYDEAD_TIME` = 200. The value 120 at `axolotl.rs:1110` is a test fixture only.

Not audited in this pass (STILL OPEN, reason: not reached):
1. `taudit-crates-pumpkin-core-src-en-39d9d7.md:38` mace_smash_knockback push bookkeeping. MaceItem.java:133 uses `nearby.push`; Rust `combat.rs:169-177` uses velocity setters.
2. `taudit-crates-pumpkin-core-src-en-39d9d7.md:42` getMessageForAssistedFall main-hand name. Vanilla reads the main hand at message time; Rust `combat.rs:766` (`get_message_for_assisted_fall`) snapshots at hit time.
3. `taudit-crates-pumpkin-core-src-en-39d9d7.md:44` causeExtraKnockback non-living push. Player.java:1227.
4. `taudit-crates-pumpkin-core-src-en-f12c04.md:22` merged movement test seam. Not a vanilla question.
7. `taudit-crates-pumpkin-core-src-en-f50f1a.md:22` support regen expanded AABB. Axolotl.java:471 uses `body.getBoundingBox().inflate(20.0)`; Rust `PLAYER_REGEN_DETECTION_RANGE` = 20.0 is the range, the box shape was not compared.
9. `taudit-crates-pumpkin-core-src-entity-liv.md:16` living.rs scaffolding descent clamp. LivingEntity.java.
10. `tentities-bamboo-chest-raft.md:39` boat.rs chest contents drop gamerule. Belongs to ChestBoat.java.
11. `tentities-bamboo-chest-raft.md:40` boat.rs double drop with loot table. Same as row 10.
15. `tentities-fishing-bobber-e0f308.md:56` fishing_rod.rs PlayerFishEvent.State.FISHING. FishingHook.java.

## Counts

VERIFIED 1 (row 8). STILL OPEN 10 (rows 1-7, 9-11, 15; rows 5 and 6 have confirmed divergences). FIXED 0.

## Checks

No code changed, so no rustfmt or cargo check was needed. Vanilla line numbers were read with `sed`/`grep` on `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/axolotl/Axolotl.java`.
