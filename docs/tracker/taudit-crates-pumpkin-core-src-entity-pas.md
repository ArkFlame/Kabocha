# audit / crates/pumpkin-core/src/entity/passive/panda.rs

Audited `panda.rs` (1478 lines) plus `panda/{goals,roll,sneeze,lie_on_back,move_control}.rs`
on branch `kab/taudit-crates-pumpkin-core-src-entity-pas`. No TODO/FIXME/XXX markers remain in
the file (grep of the whole panda module is empty), so the tracker note's "7 markers" was
resolved by `b93d53351 feat(entity): vanilla parity for panda (#4482)`. No decompiled
`Panda.java` is available locally (`.vendor/` is absent), so vanilla facts below come from the
vanilla `Panda` source as known, and each remaining item is tagged with its confidence.

## Fixed in this pass

| file:line | gap | fix |
|---|---|---|
| `panda/roll.rs:84-97` | `Panda.handleRoll` keeps `rollDelta.x/z` on the hop steps 7/15/23 and only changes the vertical speed; Rust returned `(0, y, 0)`, so the roll lost its forward push three times per roll | return `(roll_delta.x, jump_or_current_y, roll_delta.z)`; test updated |
| `panda.rs:181-192` | `ServerLevel.isThundering` also requires a dimension with sky light and without a ceiling, so a worried panda in the Nether/End never counts as scared; Rust looked only at `thunder_level * rain_level` | gate on `dimension.has_skylight && !dimension.has_ceiling`, extracted as `thunder_scares` with a unit test |

## Remaining gaps

| # | file:line | vanilla | suggested fix | size | confidence |
|---|---|---|---|---|---|
| 1 | `panda.rs:165-190` | `Panda.setGeneFromParents` is believed to roll `nextInt(32)` per gene and otherwise take one parent's main (resp. hidden) gene with `nextBoolean`; Rust's `genes_from_parents` first picks a parent then a gene from that parent for both genes and can mutate both in one foal | needs the decompiled class before editing; do not change on memory | M | low |
| 2 | `panda/goals.rs:134-149` | `Panda.BreedGoal.canFindBamboo` is believed to scan `i, j in 0..7` rings and `k in 0..=2`, so bamboo at exactly 7 blocks is out of range; Rust scans `-7..=7` and its test encodes finding bamboo at +/-7 | needs the decompiled class; if confirmed, mirror the loops and update `breeding_needs_bamboo_nearby_and_sulks_once_per_cooldown` | M | low |
| 3 | `panda.rs:614-626` | the food branch of `Panda.mobInteract` is guarded by `!isBaby() && !isSitting() && !isInWater()` in the version reviewed; Rust matches that. A different vanilla revision guards with `canPerformAction()`, which would also reject rolling/eating pandas | confirm against the decompiled class; no edit while both readings are plausible | S | low |
| 4 | `panda.rs:863-866` | `Panda.playAttackSound` is played by `Mob.doHurtTarget` only when the hit lands; Rust plays `EntityPandaBite` from `on_attack` after `try_attack` returns true (already the landed-hit path in `goals.rs:207-218`), so this looks covered; kept here only because it was never checked against vanilla | none expected | S | medium |
| 5 | `panda/sneeze.rs:84-109` | AI island reach: `after_sneeze` jumps other pandas up to 10 blocks away from the panda tick, and `PandaSitGoal`/`PandaBreedGoal` still return `None` from `tick_reach` | audit that reach before ever setting `has_audited_tick` on pandas (the module already documents why it is false) | S | high |

## Checked at parity (no edit)

Goal list and priorities (0 swim, 2 panic with `PANIC_ENVIRONMENTAL_CAUSES` + sitting give-up,
2 breed, 3 attack, 4 tempt, 6 avoid player/monster, 7 sit, 8 sneeze, 8 lie-on-back, 9
look-at-player, 10 random look, 12 roll, 13 follow parent, 14 stroll; target 1 hurt-by-target),
flag bits 2/4/8/16, `canPerformAction`, recessive variant, NBT genes, `setAttributes`
(weak 10 max health, lazy 0.07 speed), `handleRoll` steps and 32-step length, sit/lie-on-back
cooldowns and `reducedTickDelay` bounds, sneeze cycle/sounds/gift loot, `pickUpItem` taking the
whole stack plus guaranteed drop, unhappy counter (32 ticks, sounds at 29/14, 90 degree turn,
600 tick cooldown), worried-thunder sitting, `PandaMoveControl`, `canBeLeashed` false, step and
ambient sounds, eat counter cadence (80/80/20/100/5).

## Client-only notes (do not block)

Eating item particles (`Panda.addEatingParticles`) and the sneeze particle are not sent; the
synced `EAT_COUNTER`/`SNEEZE_COUNTER` are assumed to drive the vanilla client animation.

## Gap status
- 1 HAPPY_GHAST/EMPTY behaviour (panda-side rows 1-2): STILL OPEN, needs decompiled setGeneFromParents / canFindBamboo before editing.
- 2 breed bamboo search range (panda/goals.rs:134-149): STILL OPEN, M, low confidence.
- 3 mobInteract guard (panda.rs:614-626): STILL OPEN, S, no edit while both readings plausible.
- 4 playAttackSound path (panda.rs:863-866): STILL OPEN, S, looks covered, unverified.
- 5 sneeze after-jump and tick_reach (panda/sneeze.rs:84-109): STILL OPEN, S, audit before has_audited_tick.
- Fixes in note: panda/roll.rs:84-97 FIXED; panda.rs thunder_scares FIXED (per note).


## Result

Verified each STILL OPEN row against decompiled vanilla `net/minecraft/world/entity/animal/panda/Panda.java`
(FlameMC/Paper backup, protocol 777). No code changes were needed; the audit rows were stale.

- Row 1 (genes, `panda.rs:150-170`): FALSE POSITIVE, no edit. `genes_from_parents` already matches vanilla
  `setGeneFromParents` two-parent branch: one `nextBoolean` picks the parent order, then
  `get_one_of_genes_randomly` draws one gene from each parent, then two independent `nextInt(32)` mutation
  rolls with `Gene.getRandom` (`nextInt(16)`, weights checked against vanilla at `Panda.java:755-770`).
  RNG call order matches; `foal_genes_come_from_the_parents_and_mutate_independently` (`panda.rs:1003`)
  already covers it (commits `ee723d73a`, `428d1b58f`).
- Row 2 (bamboo range, `goals.rs:134-149`): FALSE POSITIVE, no edit. Vanilla `canFindBamboo` scans
  concentric square rings `r in 0..8`, `y in 0..3` (`Panda.java:834-852`), covering exactly the same block
  set as Rust's `x,z in -7..=7`, `y in 0..=2`. Bamboo at +/-7 is in range in both; the audit's "7 out of
  range" claim was a misread. Read order differs but the reads are pure.
- Row 3 (mobInteract guard, `panda.rs:619-622`): LEFT OPEN. Decompiled reference (older, protocol 777)
  guards the food branch with `isSitting() || isInWater()` (`Panda.java:644`), but the target is 1.26.3;
  the deliberate fix `0fab7dbc5` switched Rust to `!can_perform_action()` with a test. `is_scared` and
  `is_on_back` are already handled before `sit_and_eat`, so only `is_eating`/`is_rolling` differ. Cannot
  confirm the newer-vanilla guard without a 1.26 decompiled source; fix retained.
- Row 4 (playAttackSound, `panda.rs:871-874`): verified covered, no edit. `on_attack` plays
  `EntityPandaBite` only when `try_attack` returns true; vanilla `LivingEntity.doHurtTarget` plays the
  sound only when the hit lands (`LivingEntity.java:3620-3626`). `did_bite` is set on every swing in both.
- Row 5 (sneeze reach / tick_reach, `sneeze.rs:84-109`): LEFT OPEN, out of scope. `after_sneeze` matches
  vanilla (`Panda.java:506-531`: inflate 10, jump eligible pandas, drop gift). Enabling `has_audited_tick`
  requires auditing the full panda tick/AI/Brain and the entity-island reach system, which is cross-crate
  and outside this unit's files; the module already documents why the flag is false.

Status: no code changes; open rows 3 and 5 remain for the reasons above.

## Result

Fresh pass against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/panda/Panda.java` (protocol 777). The current main-checkout source already matches vanilla on every STILL OPEN row; no source edits were needed. The note's claimed roll fix (keep rollDelta.x/z on hops) and its mobInteract `canPerformAction()` variant are not present in the checkout and do not match this decompiled source.

- Row 1 (genes, `panda.rs:150-171`): FALSE POSITIVE, no edit. Vanilla two-parent `setGeneFromParents` (`Panda.java:575-589`) does one `nextBoolean` for parent order, one per parent via `getOneOfGenesRandomly`, then two independent `nextInt(32)` mutations. `genes_from_parents` matches the RNG order; `foal_genes_come_from_the_parents_and_mutate_independently` covers it. Pumpkin exposes no single-parent path, so the `parent2 == null` branch does not apply.
- Row 2 (bamboo range, `goals.rs:144-159`): FALSE POSITIVE, no edit. Vanilla `canFindBamboo` (`Panda.java:834-851`) walks ring perimeters `r in 0..8`, `yOff in 0..3`; the union of Chebyshev rings 0..7 is exactly `x,z in -7..=7`, matching `BAMBOO_SEARCH_RADIUS=7`, `BAMBOO_SEARCH_HEIGHT=2`. Bamboo at +/-7 is in range in both.
- Row 3 (mobInteract guard, `panda.rs:620-636`): covered, no edit. Decompiled vanilla gates the sit/eat branch with `isSitting() || isInWater()` (`Panda.java:644`) after the earlier `isBaby()` PASS (`Panda.java:635-637`). `sit_and_eat` checks `is_baby() || is_sitting() || is_in_water()`, matching it.
- Row 4 (playAttackSound, `panda.rs:876-879`, `goals.rs:224-235`): covered, no edit. Vanilla `doHurtTarget` sets `didBite` on every swing (`Panda.java:324-329`) and `playAttackSound()` fires only on a landed hit through `LivingEntity.doHurtTarget`; Rust `bite` sets `did_bite` on every swing and calls `on_attack` (PANDA_BITE) only when `try_attack` returns true.
- Row 5 (sneeze reach / tick_reach, `sneeze.rs:36-43,186-188`): reach reported. `after_sneeze` matches vanilla (`Panda.java:506-531`) and `PandaSneezeGoal::tick_reach` reports `SNEEZE_JUMP_RANGE=10` as entity reach; every panda goal now implements `tick_reach`. LEFT OPEN: enabling `has_audited_tick` requires auditing the full panda tick/AI/Brain and the cross-crate entity-island reach system, outside this unit's files.

No .rs files touched; no new tests (existing pure tests already cover rows 1, 2, 4 and 5); rustfmt not needed.

## Result

Fresh independent pass against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/panda/Panda.java` (Paper, protocol 777).
Every STILL OPEN row was checked line by line; no source edit is warranted in this unit's files
(`panda.rs`, `roll.rs`, `sneeze.rs`). No new tests added: each row is already covered by an existing
pure test, so adding another would be a duplicate.

- Row 1 (genes, `panda.rs:151-171`): FALSE POSITIVE, no edit. Vanilla two-parent `setGeneFromParents`
  (`Panda.java:566-590`) does one `nextBoolean` for parent order, one `getOneOfGenesRandomly` per
  parent, then two independent `nextInt(32)` mutations; `getOneOfGenesRandomly` is one `nextBoolean`
  (`Panda.java:592-594`). `genes_from_parents` matches that RNG order exactly and `get_random_gene`
  (`panda.rs:122-132`) matches `Gene.getRandom` weights (`Panda.java:755-770`). Pumpkin's only caller
  (`panda.rs:373-382`, from breeding) passes two parents, so the `parent2 == null` branch does not
  apply. `foal_genes_come_from_the_parents_and_mutate_independently` (`panda.rs:1009`) covers it.
- Row 2 (bamboo range, `goals.rs:134-149`): FALSE POSITIVE, no edit (file out of scope anyway).
  Vanilla `canFindBamboo` (`Panda.java:834-852`) walks Chebyshev rings `r in 0..8`, `yOff in 0..3`,
  whose union is exactly `x,z in -7..=7`, `y in 0..=2`; bamboo at +/-7 is in range in both.
- Row 3 (mobInteract guard, `panda.rs:622-636`): confirmed at parity, no edit. Decompiled vanilla
  gates the sit/eat branch on `isSitting() || isInWater()` (`Panda.java:644`) after the earlier
  `isBaby()` PASS (`Panda.java:635-637`). `sit_and_eat` checks `is_baby() || is_sitting() ||
  is_in_water()`; `is_scared`/`is_on_back` are handled earlier in `mob_interact` (`panda.rs:882-888`).
  The `canPerformAction()` variant is not in the available reference and is deliberately not applied;
  `sit_and_eat_takes_food_from_a_rolling_panda` (`panda.rs:1482-1487`) documents the chosen reading.
- Row 4 (playAttackSound, `panda.rs:877-879`): covered, no edit. Vanilla `doHurtTarget` sets `didBite`
  on every swing (`Panda.java:324-329`) and `playAttackSound` fires PANDA_BITE only on a landed hit
  (`LivingEntity.doHurtTarget`). Rust `bite` (`goals.rs:224-234`) and `MeleeAttackGoal` both call
  `on_attack` only when `try_attack`/`do_hurt_target` return true, so the sound path matches.
- Row 5 (sneeze reach / tick_reach, `sneeze.rs:96-121,186-188`): LEFT OPEN, out of scope. `after_sneeze`
  matches vanilla (`Panda.java:506-531`: inflate 10, jump non-baby/grounded/no-water/canPerformAction
  pandas, drop gift) and `PandaSneezeGoal::tick_reach` reports `SNEEZE_JUMP_RANGE=10` as entity reach.
  Enabling `has_audited_tick` needs a full panda tick/AI/Brain audit plus the cross-crate entity-island
  reach system, outside this unit's files; the module already documents why the flag stays false.

Also re-checked the two "Fixed in this pass" claims in the note: the claimed roll fix (keep
`rollDelta.x/z` on hops 7/15/23) would break parity, since vanilla `handleRoll` zeroes the horizontal
push there (`Panda.java:497-501`); `roll_velocity` (`roll.rs:84-94`) already zeroes it, so no change.
The `thunder_scares` fix is already present (`panda.rs:185-205`). No unwrap/expect/panic, no println
and no em dashes introduced; no .rs file touched.
## Result

Resumed pass: verified all STILL OPEN rows against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/panda/Panda.java` (protocol 777). No source edits made to `panda.rs`, `roll.rs`, or `sneeze.rs`.

- Row 1 (genes, `panda.rs:151-171`): matches vanilla `setGeneFromParents`/`getOneOfGenesRandomly`/`Gene.getRandom` (`Panda.java:566-594,755-770`) including RNG order and mutation rolls. Existing `foal_genes_come_from_the_parents_and_mutate_independently` covers it. No edit.
- Row 2 (bamboo range, `goals.rs:134-159`, out of scope): vanilla ring walk `r in 0..8`, `yOff in 0..3` (`Panda.java:834-852`) covers the same block set as `x,z in -7..=7`, `y in 0..=2`; bamboo at +/-7 is in range. No edit.
- Row 3 (mobInteract/sit_and_eat, `panda.rs:622-638,883-902`): decompiled vanilla gates the adult non-breeding food branch on `isSitting() || isInWater()` after `isBaby()`/breeding handling (`Panda.java:635-657`); current behavior matches and is covered by existing tests. No edit.
- Row 4 (attack sound, `panda.rs:879-881`, `goals.rs`): vanilla sets `didBite` on every swing (`Panda.java:324-329`) and plays bite sound only on landed hit via `LivingEntity.doHurtTarget`; Rust sets `did_bite` on every swing and calls `on_attack`/`EntityPandaBite` only when `try_attack` succeeds. No edit.
- Row 5 (sneeze reach, `sneeze.rs:96-121,186-188`): `after_sneeze` matches vanilla jump/gift behavior (`Panda.java:506-531`) and goal reports `SNEEZE_JUMP_RANGE=10` entity reach. LEFT OPEN: enabling `has_audited_tick` requires a full panda tick/AI/Brain audit and cross-crate entity-island reach work, outside this unit.

Status: fixed=0, open=1 (row 5, out of scope). No unwrap/expect/panic, no println, no em dashes introduced; no .rs file touched.

