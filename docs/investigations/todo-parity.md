# Audit: docs/TODO_2.md and PARITY.tsv RE-REVIEW / UNTRACKED rows

Counts: part 1 rows=25 (RE-REVIEW=10, RE-REVIEW-UNTRACKED=3, UNCOVERED=4, COVERED=8); verdicts RESOLVED=11 STILL-OPEN=6 NO-NOTE=8. Part 2 items=42; verdicts DONE=26 OPEN=11 UNKNOWN=5. Method: read-only; PARITY.tsv `covered` column filtered to the four named values, each row matched to `docs/tracker/t<category>-<name>.md` (or the nearest audit note found by name grep), and the cited Rust file checked. Part 2 evidence taken only from docs/DONE.md, docs/GAPS_FIXED.md and docs/tracker/*.md; no git, no cargo.

## Part 1 - PARITY.tsv rows with covered in {RE-REVIEW, RE-REVIEW-UNTRACKED, UNCOVERED, COVERED}

| category | name | covered | Verdict | Evidence |
|---|---|---|---|---|
| entities | Bogged | RE-REVIEW | RESOLVED | `tverify-mobs9.md` Bogged table: attack interval 70/50, poison arrow 100t, shear/drop and all sounds OK, "no S/M divergence found; nothing changed"; `bogged.rs` has the interval and `get_arrow_projectile` overrides. `tverify-mobs7.md` found no divergence. |
| entities | Breeze | RE-REVIEW | STILL-OPEN | `tverify-mobs9.md`: head-rotation speed (no Mob hook), pathfinding malus, and full `BreezeAi` Brain port (Fight/Idle activities, Shoot/LongJump/Slide, 100-tick attackable memory) marked STILL OPEN; `tverify-mobs7.md` "blocked on a full Brain audit". |
| entities | Creaking | RE-REVIEW | RESOLVED | `tentities-creaking.md` Result (re-review): gaze cone fixed via `GAZE_CONE_SIZE`, disguise gating restored in `check_can_move`, attack-animation counter + event 4 added; all in-scope rows FIXED/no-gap. Remaining items are client-only eye glow and generated loot tables (both explicitly out of unit, no server-observable divergence). |
| entities | Guardian | RE-REVIEW | RESOLVED | `tentities-guardian.md` Result 2026-11-18: `GuardianAttackGoal` (prio 3, charge 80/60, INDIRECT_MAGIC, metadata clear) verified in code, move-control 0.125 lerp and `wanted_position` verified, gaps 2/4/5/6 already fixed; "No STILL OPEN rows remain". |
| entities | Hoglin | COVERED | STILL-OPEN | `tverify-mobs4.md:7-8`: crimson-nylium walk value returns 10.0 but vanilla repellent `-1.0` needs `NEAREST_REPELLENT` brain memory the Rust mob lacks; `HoglinAi` brain not ported. Matches the PARITY note. |
| entities | Illusioner | UNCOVERED | NO-NOTE | No tracker audit note for the bow/spell gap; only incidental equipment lines (`taudit-...-en-921e2f.md:14,33`). `docs/GAPS_FIXED.md:3489` "STILL OPEN T2-005 Illusioner (L)"; `illusioner.rs` still has no RangedBowAttackGoal/spell handlers. |
| entities | Parched | COVERED | RESOLVED | `parched.rs:27-39` overrides `get_arrow` adding WEAKNESS 600t (vanilla `Parched#getArrow`) and returns 50/70 attack intervals; `taudit-...-en-aadca5.md` Result verifies interval + per-species arrow payload, `tverify-mobs.md:7` FIXED. PARITY note ("plain ARROW") is stale. |
| entities | Piglin | RE-REVIEW | STILL-OPEN | `tentities-piglin.md` Remaining gaps: baby flee goal set, `avoidZombified` 6-block, `angerNearbyPiglins` 600t, hunt cooldown on lost target, spawn-rule verification. No Result section closing them; issue 3376 still open. |
| entities | Piglin Brute | COVERED | NO-NOTE | No audit note; `tworld-sweep3.md:47` lists it as "not reviewed this pass (budget)". The stored home / return-to-home behaviour in the PARITY note was never audited. |
| entities | Ravager | COVERED | NO-NOTE | `thostile-sweep.md:30` "Not re-verified: ... ravager"; `tverify-mobs2.md:16-17` only covered walk speed and client-only stun particles. No audit of the stun/roar state in the PARITY note. |
| entities | Silverfish | COVERED | NO-NOTE | No audit note for the countdown / infested-block scan / stone-merging gap; `tblocks-infested-blocks-f4f39a.md:5` only records the entity-type mapping and `thostile-sweep.md:30` lists it as not re-verified. |
| entities | Skeleton | COVERED | RESOLVED | `skeleton/mod.rs:32-33,134-140` returns 40 normal / 20 Hard via `get_min_attack_interval`, consumed by `bow_attack.rs:271-273`; `tverify-mobs.md:14` VERIFIED against `AbstractSkeleton`, `taudit-...-en-aadca5.md` Result found no gap. PARITY note ("fixed at 20 ticks") is stale. |
| entities | Warden | COVERED | STILL-OPEN | `tverify-mobs2.md:21` verifies darkness numbers but records GAP: `WardenAi` brain not ported (dig cooldown 1200, sonic-boom cooldown); `docs/GAPS_FIXED.md:622` issue #4123 SKIPPED. |
| entities | Ender Dragon | COVERED | NO-NOTE | `thostile-sweep.md:21` "withers, ender dragon | skipped (size) | not audited" and `:30` "Not re-verified: ... ender dragon". Part registration was addressed by PRs #3407/#3661 (DONE in GAPS_FIXED) but the cited part-based damage/contact semantics were never audited. |
| entities | Wither | RE-REVIEW | RESOLVED | `tentities-wither.md` Result verified all 9 rows against `WitherBoss.java`: lift, spawn-goal guard, hovering, bossbar, `break_block` drops, idle offsets, peaceful despawn are NOT A GAP; rows 4/6/9 FIXED with a unit test for powered immunity. |
| entities | Copper Golem | RE-REVIEW | RESOLVED | `tentities-copper-golem.md` Result (re-review 2026-11-18): statue conversion, chest transport, NBT `next_weather_age`/`weather_state` and `>=` weathering boundary all ALREADY FIXED/NO BUG against `CopperGolem.java`. |
| entities | Goat | RE-REVIEW | STILL-OPEN | `tentities-goat.md` Result: ram activity (`RamTargetGoal` constants differ, `PrepareRamNearestTarget` absent), long jump, horn drop and brain-memory init are STILL OPEN pending out-of-unit goal files; only the spawn rule is already fixed. |
| entities | Nautilus | RE-REVIEW | RESOLVED | `tentities-nautilus.md` Result: 7 of 9 rows FIXED (AI goals, jump impulse, dead inventory removed, food tags, navigation/move controls, NBT/taming) and the rest VERIFIED; the one LEFT OPEN item is the client-only riding sound (client source not vendored), no server divergence. |
| entities | Ocelot | RE-REVIEW | RESOLVED | `tentities-ocelot.md` Result: `mobInteract` dist_sqr < 9.0, plain offspring and `isSteppingCarefully` all verified matching `Ocelot.java`; ambient/hurt/death sounds FIXED. Every tracker premise that claimed a gap was wrong. |
| entities | Spawner Minecart | RE-REVIEW-UNTRACKED | STILL-OPEN | `tentities-spawner-minecart.md` Remaining gaps: weighted random `SpawnPotentials`, `SpawnData` NBT (`entityToSpawn` with name/equipment/passengers/NoAI), and custom spawn rules; only nearby-count and `spawnAnim` are marked FIXED. No Result section. |
| entities | Ominous Item Spawner | RE-REVIEW-UNTRACKED | RESOLVED | `tentities-ominous-item-spawner.md` Result: launch sound and spawn origin FIXED in pass; rows 2 (dispense config 1.375/3.0), 3 (particle flag 1) and 5 (explosion immunity) CHECKED no change against decompiled vanilla. |
| blocks | Lodestone | RE-REVIEW-UNTRACKED | RESOLVED | `tblocks-lodestone.md` Result (re-review 2026-11-18): duplicate POI hooks already removed (`impl BlockBehaviour` empty, engine-driven `update_poi`), rows 1/3/6 confirmed no-gap, rows 4/5 out of unit; `lodestone.rs` matches. |
| blocks | Ender Chest | UNCOVERED | NO-NOTE | `tcontainers-sweep.md:27` "Viewer counts, sounds, ender chest, jukebox: not read in this pass"; `taudit-...-co-20e7ce.md:30` only checks inventory mapping. The guarded-block interaction / piglin anger call (T2-001) was never audited. |
| blocks | Sculk Catalyst | UNCOVERED | NO-NOTE | No audit note for the death listener / sculk-spreading block entity; `taudit-...-en-6f030b.md:22,43` only re-verifies the soul-charge experience reward and `tverify-blocks.md` only records commit 2716bcff4 (bloom volume/pitch). |
| blocks | Moving Piston | UNCOVERED | NO-NOTE | Only an incidental mention at `pass2/0291.md:16` (structure block / piston list). No audit of `piston_extension.rs:25-26` losing the breaking-player context (T2-002). |

## Part 2 - docs/TODO_2.md items

| item | Verdict | Evidence |
|---|---|---|
| T2-001 Ender Chest (guarded block anger) | OPEN | No fix in DONE.md/GAPS_FIXED.md; `tcontainers-sweep.md:27` not read; PARITY row still UNCOVERED / NO-NOTE. |
| T2-002 Moving Piston (break player context) | UNKNOWN | `GAPS_FIXED.md:1152` records `tblocks-pistons` "FIXED `piston_extension.rs:15-30`" but only for the creative-only `playerWillDestroy` guard; no evidence the `break_block` player argument was restored. PR #4047 (moving-piston desync) DONE is a different bug. |
| T2-003 Infested blocks (generic entity) | OPEN | `GAPS_FIXED.md:3487` still lists T2-003 verbatim in the remaining-gaps section, despite `tblocks-infested-blocks-f4f39a.md` "no gap, FIXED (no change needed)" for a different row. |
| T2-004 Sculk Catalyst (death listener + spreader) | OPEN | Only bloom volume/pitch commit 2716bcff4 in `tverify-blocks.md`; part 1 NO-NOTE, no listener/spreader fix. |
| T2-005 Illusioner (bow + spells) | OPEN | `GAPS_FIXED.md:3489` "STILL OPEN T2-005 Illusioner (L)"; no tracker audit note. |
| Dropped: Dirt Path | DONE | `DONE.md:540` issue #3580 DONE (`fe1dfe320`), `DONE.md:332` #3585 skipped as already identical; `GAPS_FIXED.md:1487`. |
| Dropped: Farmland | DONE | Same issue #3580 DONE; `DONE.md:112/332` skipped as already on master; `GAPS_FIXED.md:1488`. |
| Dropped: Sculk Sensor | DONE | `GAPS_FIXED.md` #3107 (Sculk Shrieker) DONE (`e1ed99c1b`); shared root, so no separate slice. |
| T2-C01 Blaze | DONE | `GAPS_FIXED.md:262` #4113 and `:339` #4505 DONE, `:150` #3790 skipped; `DONE.md:25` #4562 DONE. PARITY AUDITED note still records a restricted-area movement detail. |
| T2-C02 Cave Spider | DONE | `GAPS_FIXED.md:298` #4368 "vanilla parity for spider and cave spider" DONE (`ed6d588bf`); umbrella #4228 SKIPPED. |
| T2-C03 Hoglin | OPEN | Umbrella #4259 and #3974 DONE (`GAPS_FIXED.md:602,687`) but repellent/brain gap still open (part 1, `tverify-mobs4.md:7-8`). |
| T2-C04 Parched | DONE | `GAPS_FIXED.md:689` #4261 DONE (`336b588d5`); `parched.rs` arrow/interval verified (part 1 RESOLVED). |
| T2-C05 Piglin Brute | UNKNOWN | `GAPS_FIXED.md:690` #4263 DONE and `:100` #3630 brain move DONE, but no audit of the cited home/return-to-home behaviour (part 1 NO-NOTE). |
| T2-C06 Ravager | DONE | `GAPS_FIXED.md:692` #4265 DONE (`98db6d375`); walk-speed fix in `tverify-mobs2.md:16`. Roar/stun state unaudited (part 1 NO-NOTE). |
| T2-C07 Silverfish | DONE | `GAPS_FIXED.md:696` #4269 DONE (`fb361086a`), `:185` #3886 "complete the silverfish goal set" DONE. |
| T2-C08 Skeleton | DONE | `GAPS_FIXED.md:694` #4267 DONE (`99905c6f6`), `:214` #3988 bogged/skeleton parity DONE; code interval verified (part 1 RESOLVED). |
| T2-C09 Spider | DONE | `GAPS_FIXED.md:298` #4368 DONE; #4241 SKIPPED (already covered by the same parity work). |
| T2-C10 Stray | OPEN | `GAPS_FIXED.md:3488` still lists T2-C10 "arrow payload still open"; #4270 DONE but the slowness arrow wrapper was not fixed. |
| T2-C11 Vex | DONE | `GAPS_FIXED.md:698` #4271 DONE (`ad0637262`); not in the remaining-gaps list, though `tentities-vex-8c3541` still describes goals. |
| T2-C12 Vindicator | DONE | `GAPS_FIXED.md:699` #4272 DONE (`ca7eec503`). |
| T2-C13 Warden | OPEN | #4122 sonic boom DONE (`GAPS_FIXED.md:621`) but #4123 SKIPPED (`:622`) and the WardenAi brain gap remains (part 1 STILL-OPEN). |
| T2-C14 Witch | DONE | `GAPS_FIXED.md:701` #4274 DONE (`6a8ea06a6`), `:313` #4393 vanilla-parity PR DONE. |
| T2-C15 Zoglin | DONE | `GAPS_FIXED.md:602` #3974 (hoglin and zoglin throw guards) DONE (`2fdcbe601`). |
| T2-C16 Ender Dragon | DONE | `GAPS_FIXED.md:50` #3407 (make dragon hittable) DONE, `:116` #3661 (register hitbox parts) DONE; remaining part-based damage semantics still un-audited (part 1 NO-NOTE). |
| T2-C17 Bee | DONE | `GAPS_FIXED.md:664` #4227 DONE (`44a260098`). |
| T2-C18 Dirt Path | DONE | `GAPS_FIXED.md:540` #3580 DONE; push_entities_up present. |
| T2-C19 Farmland | DONE | Same #3580 DONE; `DONE.md:332` #3585 skipped as empty. |
| T2-C20 Sculk Shrieker | DONE | `GAPS_FIXED.md:488` #3107 DONE (`e1ed99c1b`). |
| T2-C21 Sculk Sensor | DONE | Root cause shared with #3107 DONE; no separate slice was planned. |
| T2-C22 Tripwire | DONE | `GAPS_FIXED.md:94` #3584 DONE (`2680883ce`), `:107` #3640 DONE (`07c4d7861`); `DONE.md:142,325`. |
| Section 4 (87 RE-REVIEW audit rows) | OPEN | PARITY.tsv now has only 25 rows left in the four target states; 120 tracker files carry a re-review/Result section, so most of the 87 were audited. This audit resolves 17 of the 25 (11 RESOLVED + 6 STILL-OPEN) but 8 still have NO audit note (part 1), so the "one worker per row" plan is not complete. |
| INFRA-1 parity ledger script `scripts/parity.sh` | DONE | File exists at `scripts/parity.sh`; `scripts/sync-upstream.sh` also present for the hourly refresh. |
| INFRA-2 one re-review worker per section-4 row | OPEN | 8 of the 25 remaining PARITY rows have NO-NOTE (part 1); `GAPS_FIXED.md:3490` still lists re-review rows as "not audited this pass". |
| INFRA-3 math parity harness (StrictMath pow/log10) | UNKNOWN | No mention in DONE.md/GAPS_FIXED.md and no harness found; cannot confirm from allowed read-only sources. |
| INFRA-4 block-equality benchmark vs vanilla dumps | UNKNOWN | No mention in DONE.md/GAPS_FIXED.md; no benchmark evidence in the allowed sources. |
| INFRA-5 vanilla-bug registry `docs/VANILLA_BUGS.md` | DONE | File exists at `docs/VANILLA_BUGS.md`. |
| INFRA-6 1.8 combat mode design note | DONE | `docs/COMBAT_1_8.md` exists. |
| INFRA-7 refresh stale Dirt Path / Farmland tracker notes | UNKNOWN | Note-only item; `DONE.md:332` records the upstream fix, but no evidence the tracker notes were refreshed. |
| H9 Name Tag fix | DONE | `TODO_2.md:189` records FIXED d4ac58548; PARITY.tsv `Name Tag` is now AUDITED. |
| H9 Illusioner (T2-005) | OPEN | `TODO_2.md:190` STILL OPEN; `GAPS_FIXED.md:3489` unchanged; too large for the pass. |
| H9 remaining unaudited re-review rows | OPEN | `TODO_2.md:191` lists them; part 1 confirms 6 STILL-OPEN and 8 NO-NOTE. |
| H9 out-of-scope deferrals (Hoglin, Wolf, boat, minecart, raid, wasm-host, BLOCKED-DATA) | DONE | `TODO_2.md:193` explicitly defers them by brief; no work owed in this plan. |

## PARITY.tsv rows whose `covered` value should change

All 17 rows below have a completed re-review audit (tracker Result section or equivalent audit table). The `RE-REVIEW` / `RE-REVIEW-UNTRACKED` / `COVERED` label is stale because the audit is done; `AUDITED` is the value used elsewhere in PARITY.tsv for audited rows (e.g. Blaze, Creeper). Change to `AUDITED`; for the six STILL-OPEN rows keep the gap tracked in TODO_2/GAPS_FIXED (consider `OWN-OPEN` if you prefer encoding "gap filed" in this column, since no own-issue row exists for them today). The eight NO-NOTE rows should keep their current value until somebody actually audits them.

| category | name | current covered | change to | why |
|---|---|---|---|---|
| entities | Bogged | RE-REVIEW | AUDITED | RESOLVED; `tverify-mobs9.md` no divergence. |
| entities | Breeze | RE-REVIEW | AUDITED | Audited; gaps recorded in `tverify-mobs9.md` (Brain port open). |
| entities | Creaking | RE-REVIEW | AUDITED | RESOLVED; `tentities-creaking.md` re-review complete. |
| entities | Guardian | RE-REVIEW | AUDITED | RESOLVED; "No STILL OPEN rows remain". |
| entities | Hoglin | COVERED | AUDITED | Audited; repellent/brain gap recorded in `tverify-mobs4.md:7-8`. |
| entities | Parched | COVERED | AUDITED | RESOLVED; arrow payload + interval implemented and verified. |
| entities | Piglin | RE-REVIEW | AUDITED | Audited; remaining gaps table in `tentities-piglin.md`. |
| entities | Skeleton | COVERED | AUDITED | RESOLVED; 40/20 interval verified in `skeleton/mod.rs`. |
| entities | Warden | COVERED | AUDITED | Audited; brain gap recorded in `tverify-mobs2.md:21`. |
| entities | Wither | RE-REVIEW | AUDITED | RESOLVED; all 9 rows verified in `tentities-wither.md`. |
| entities | Copper Golem | RE-REVIEW | AUDITED | RESOLVED; all 4 rows verified in `tentities-copper-golem.md`. |
| entities | Goat | RE-REVIEW | AUDITED | Audited; open goal-file gaps in `tentities-goat.md`. |
| entities | Nautilus | RE-REVIEW | AUDITED | RESOLVED; only a client-only sound left open. |
| entities | Ocelot | RE-REVIEW | AUDITED | RESOLVED; all premises disproved in `tentities-ocelot.md`. |
| entities | Spawner Minecart | RE-REVIEW-UNTRACKED | AUDITED | Audited; remaining gaps in `tentities-spawner-minecart.md`. |
| entities | Ominous Item Spawner | RE-REVIEW-UNTRACKED | AUDITED | RESOLVED; `tentities-ominous-item-spawner.md` Result. |
| blocks | Lodestone | RE-REVIEW-UNTRACKED | AUDITED | RESOLVED; `tblocks-lodestone.md` re-review 2026-11-18. |

Bottom line for the community member: the AI agents did work through most of TODO_2.md - 26 of 42 planned items are DONE and 17 of the 25 lingering PARITY rows have a real audit behind them. But 8 rows were never audited, 6 audits found genuine remaining gaps, and 3 infra items plus 5 uncovered/concrete gaps are still unverified or open, so "everything was done" is not accurate.
