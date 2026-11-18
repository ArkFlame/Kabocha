# entities / Axolotl

Row: `entities	Axolotl	partial	RE-REVIEW` in `docs/PARITY.tsv`.
Source re-reviewed at `386e27d03` (`feat(entity): vanilla parity for axolotl`, #4502).
Reference: decompiled vanilla `net.minecraft.world.entity.animal.axolotl.Axolotl` (no `.vendor/` in this worktree, method signatures checked against the 1.21 / 1.21.6 mappings index).

## Fixed in this pass

- `travel_in_water` now applies vanilla `Axolotl.travel`'s no-target sink (`deltaMovement.y -= 0.005` after the `* 0.9` drag). `crates/pumpkin-core/src/entity/passive/axolotl.rs:541`.

## Remaining gaps (unverified, need the decompiled body)

1. **No effective-AI guard in travel** — `crates/pumpkin-core/src/entity/passive/axolotl.rs:541`
   Vanilla `Axolotl.travel` only uses the axolotl water movement `if (isEffectiveAi() && isInWater())`, otherwise `super.travel`. Here `travel_in_water` (called through `mob_travel_in_fluid`, see `crates/pumpkin-core/src/entity/living.rs:2351`) has no `is_effective_ai()` check, and the unused `gravity`/`falling` parameters are exactly what a `LivingEntity::travel_in_water` fallback needs.
   Fix: `if !self.is_effective_ai() { self.mob_entity.living_entity.travel_in_water(caller, gravity, falling); return; }`. Size S.

2. **Play-dead regeneration constants** — `crates/pumpkin-core/src/entity/passive/axolotl/ai.rs:39-40,353-364`
   Port grants `REGENERATION` for 200 ticks at amplifier 0. Vanilla `Axolotl$PlayDead.start` is believed to use `new MobEffectInstance(MobEffects.REGENERATION, 100, 1)` (100 ticks, Regeneration II). Confirm against 1.21.x source; if so change duration/amplifier. Size S.

3. **No-AI air path** — `crates/pumpkin-core/src/entity/passive/axolotl.rs:520-529`
   `base_tick` sends no-AI axolotls to `LivingEntity::tick_air_supply` and AI axolotls to `handle_air_supply`, on the claim that vanilla wraps `handleAirSupply` in `!isNoAi()`. Verify `Axolotl.baseTick` / `handleAirSupply(ServerLevel, int)` (1.21.6 signature) and the resulting land refill `. Size S.

4. **Hurt play-dead condition** — `crates/pumpkin-core/src/entity/passive/axolotl.rs:158-166`
   Uses `(roll/3 < damage || health/max < 0.5) && damage < health && in_water && has_attacker`. Confirm the OR between the damage roll and the half-health check, and the `damage < health` ordering, against `Axolotl.hurtServer`. Size S.

5. **Extra bucket NBT keys** — `crates/pumpkin-core/src/entity/passive/axolotl.rs:363-401`
   `save_to_bucket_tag` writes `Age`, `AgeLocked` and `HuntingCooldown`; vanilla `saveToBucketTag` is `super.saveToBucketTag` plus the `axolotl/variant` component. Check whether these keys are really in `Bucketable.saveDefaultDataToBucketTag` (1.21.x) or were added speculatively; if speculative, drop keys 2-3. Size S.

6. **Duplicated doc comment** — `crates/pumpkin-core/src/entity/passive/axolotl.rs:183-184`
   `apply_supporting_effects` doc line is repeated (second copy lacks the final period). Cosmetic. Size S.

## Checked and matching

Variant pools and rare 1/1200 roll, group-data spawning/baby rule, air supply 6000/1800 and dry-out damage 2, bucket round trip, breeding/food tag, play-dead/attack/fight brain activities, hunting cooldown memory, `onStopAttacking` killer-in-20-blocks regen, supporting-effects duration math (100 per axolotl, cap 2400), head rotation limits, hurt/death/ambient/swim sounds (from `entity_type` data plus overrides), leash (axolotls are leashable), `removeWhenFarAway`, `requiresCustomPersistence` (call site ORs `persistence_required`), `canBeSeenAsEnemy`.

## h2 parity sweep (status)
- FIXED f090717d1 - `travel_in_water` falls back to `LivingEntity::travel_in_water` when `is_no_ai()` (vanilla `Axolotl.travel` gates the swim path on `isEffectiveAi() && isInWater()`). UNVERIFIED (no build).
- Play-dead regen constants, no-AI air path, hurt condition: VERIFIED against .vendor decompiled vanilla (PlayDead.java:32, Axolotl.java:184-189/256-266, Axolotl.java:396-408); already correct, no change.

## Result

Verified the rows marked STILL OPEN against the decompiled vanilla source under .vendor/vanilla-decompiled/net/minecraft/.

- Row 2 (play-dead regeneration): decompiled world/entity/animal/axolotl/PlayDead.java:32 uses new MobEffectInstance(MobEffects.REGENERATION, 200, 0). Pumpkin PLAY_DEAD_REGENERATION_DURATION = 200 with amplifier 0 already matches; the doc's guessed (100, 1) was wrong. No change.
- Row 3 (no-AI air path): verified Axolotl.baseTick (Axolotl.java:184-189), handleAirSupply (Axolotl.java:256-266) and the inline standard air rules in LivingEntity.baseTick (LivingEntity.java:507-528). Vanilla always runs the standard air rules and runs the axolotl dry-out handler only with AI. Pumpkin's no-AI branch runs tick_air_supply (standard rules) and its AI branch runs handle_air_supply (dry-out), which sets air supply in both of its branches, so the net air and dry-out damage match vanilla. No change.
- Row 4 (hurt play-dead condition): the roll/half-health OR, damage < health ordering and water/attacker checks in plays_dead_when_hurt already match Axolotl.hurtServer (Axolotl.java:396-408). Found and fixed one real mismatch: vanilla checks (source.getEntity() != null || source.getDirectEntity() != null), but on_damage_with_context only checked the causing entity (cause.is_some()) and ignored the direct entity. Changed it to source.is_some() || cause.is_some() and updated the only_source repro assertion to Some(200). The source/cause mapping (direct entity vs causing entity) is confirmed by the lightning and arrow damage call sites.

Outcome: fixed 1 row (row 4); rows 2 and 3 verified already correct. Row 1 (travel effective-AI guard) was already fixed in a prior pass; row 6 (duplicated doc comment) is already a single line. Row 5 (bucket NBT keys) left open: needs checking against generated bucket data and is outside the explicit STILL OPEN set.

Only axolotl.rs was edited; rustfmt --edition 2024 on it exited 0. No cargo/git run (per task rules), so compilation and tests are UNVERIFIED.

## Result (recheck)

Re-checked every STILL OPEN row against the current code in the main checkout. No decompiled vanilla source is present in this worktree (.vendor/ is absent and git clone is not permitted), so verification used the prior pass's decompiled-source notes plus direct code inspection.

- Row 2 (play-dead regen): axolotl/ai.rs:40 sets PLAY_DEAD_REGENERATION_DURATION = 200 with amplifier 0 (ai.rs:359), matching vanilla Regeneration 200/0. That file is outside this unit; no edit.
- Row 3 (no-AI air path): axolotl.rs:574-580 keeps the standard air rules for no-AI axolotls and the dry-out handler for AI axolotls, as previously verified against vanilla; no mismatch found. No change.
- Row 4 (hurt play-dead condition): axolotl.rs:158-166 and :374 already implement the two-roll OR and the source||cause check; tests at :1358-1359 assert only_source and only_cause both play dead. No change.
- Row 1 (travel effective-AI guard) and row 6 (duplicated doc comment) were already fixed in earlier passes (axolotl.rs:592-599 and :183).
- Row 5 (extra bucket NBT keys): left open because confirming whether Age/AgeLocked/HuntingCooldown are vanilla requires generated bucket data outside this unit (axolotl.rs:412-435).

No .rs file was changed, so rustfmt was not needed. No cargo or git command was run per task rules; compile and tests remain UNVERIFIED.

## Result (decompiled-source verification, .vendor present)

The decompiled vanilla tree is now available under .vendor/vanilla-decompiled/net/minecraft/, so every STILL OPEN row was checked against real method bodies this pass. No code change was needed: all rows already match vanilla.

- Row 1 (effective-AI travel guard): axolotl.rs:592-599 falls back to `LivingEntity::travel_in_water` on `is_no_ai()`, matching vanilla `Axolotl.travel` gating the swim path on `isEffectiveAi() && isInWater()`. Already fixed.
- Row 2 (play-dead regen): `PlayDead.start` (PlayDead.java:32) is `new MobEffectInstance(MobEffects.REGENERATION, 200, 0)`. ai.rs:40,358-359 use duration 200, amplifier 0. The doc's guessed (100, 1) was wrong. No change.
- Row 3 (no-AI air path): vanilla `Axolotl.baseTick` (Axolotl.java:184-189) always runs the standard air rules via `super.baseTick()` (LivingEntity.java:507-528) and runs the dry-out handler only with AI; `Axolotl.handleAirSupply` (256-266) is exactly `water_animal::handle_air_supply` (water_animal.rs:52-60). axolotl.rs:574-580 routes no-AI to `tick_air_supply` and AI to `handle_air_supply`, and the dry-out helper sets air in both branches, so net air/damage match. No change.
- Row 4 (hurt play-dead): `Axolotl.hurtServer` (Axolotl.java:396-408) confirmed the roll/half-health OR, `damage < health`, and `(source.getEntity() != null || source.getDirectEntity() != null)`. axolotl.rs:158-166,374 (source.is_some() || cause.is_some()) and tests at :1358-1359 match. Already fixed.
- Row 5 (bucket NBT keys): RESOLVED. Vanilla `Axolotl.saveToBucketTag` (Axolotl.java:427-437) explicitly writes `Age`, `AgeLocked` and `HuntingCooldown` (gated on HAS_HUNTING_COOLDOWN memory value) into bucket entity data, and `loadFromBucketTag` (441-449) restores them. axolotl.rs:410-452 matches; the keys are vanilla, not speculative. Prior passes left this open only because vanilla source was absent. No change.
- Row 6 (duplicated doc comment): axolotl.rs:183 is a single doc line. Already fixed.

Outcome: fixed 0 rows this pass (all six already correct); row 5 newly verified against vanilla, closing the last open item. No pure unit test added: existing tests already cover hurt source/cause (:1358-1359) and bucket round trip (:1494-1545). No .rs file changed, so rustfmt was not applicable. No cargo/git run (per task rules); compile and tests UNVERIFIED.

## Result (final)

All six rows are closed. Each was re-verified this pass against the decompiled vanilla tree under .vendor/vanilla-decompiled/net/minecraft/; no source change was needed.

- Row 1 (effective-AI travel guard): VERIFIED. axolotl.rs:592-599 falls back to LivingEntity::travel_in_water on is_no_ai(). The vanilla water swim path is Axolotl.travelInWater (.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/axolotl/Axolotl.java:531-535), dispatched from LivingEntity.travelInFluid (LivingEntity.java:3208-3219); the no-AI fallback matches vanilla. No change.
- Row 2 (play-dead regen constants): VERIFIED. PlayDead.start (.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/axolotl/PlayDead.java:32) applies new MobEffectInstance(MobEffects.REGENERATION, 200, 0). Pumpkin uses duration 200, amplifier 0 (ai.rs:40,358-359); the doc's old 100/1 guess was wrong. No change.
- Row 3 (no-AI air path): VERIFIED. Vanilla Axolotl.baseTick (Axolotl.java:184-189) runs standard air rules via super.baseTick() and the dry-out handler only with AI; handleAirSupply (Axolotl.java:256-266) equals water_animal::handle_air_supply (water_animal.rs:41-61), which sets air in both branches. Pumpkin routes no-AI to tick_air_supply and AI to handle_air_supply (axolotl.rs:574-580), so net air and dry-out damage match. No change.
- Row 4 (hurt play-dead): VERIFIED. Vanilla Axolotl.hurtServer (Axolotl.java:396-408) uses the roll/half-health OR, damage < health, and (source.getEntity() != null || source.getDirectEntity() != null). Pumpkin's plays_dead_when_hurt and source.is_some() || cause.is_some() (axolotl.rs:158-166,374) match. No change.
- Row 5 (bucket NBT keys): VERIFIED. Vanilla Axolotl.saveToBucketTag / loadFromBucketTag (Axolotl.java:427-449) write and restore Age, AgeLocked and the gated HuntingCooldown; Pumpkin (axolotl.rs:410-452) matches, including erasing the memory when the tag has no HuntingCooldown (no axolotl behavior requires that memory as VALUE_ABSENT, AxolotlAi.java:156). No change.
- Row 6 (duplicated doc comment): VERIFIED. The apply_supporting_effects doc is a single line (axolotl.rs:183). No change.

Outcome: 0 rows fixed this pass (all six already match vanilla); the stale open status line in the h2 parity sweep section was corrected to VERIFIED. No pure unit test added: existing tests already cover the hurt source/cause (axolotl.rs:1358-1359) and bucket round trip (axolotl.rs:1494-1545) seams, and travel/bucket have no smaller pure seam. No .rs file changed, so rustfmt was not applicable. No cargo/git command was run per task rules; compile and tests are UNVERIFIED.
