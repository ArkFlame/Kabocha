# Parity rows for crates/pumpkin-core/src/entity/passive/panda.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Panda** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/taudit-crates-pumpkin-core-src-en-7bf7cf** [partial/OWN-OPEN] own audit: 11 open, 0 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-en-7bf7cf.md) docs/tracker/taudit-crates-pumpkin-core-src-en-7bf7cf.md
- **own/taudit-crates-pumpkin-core-src-entity-pas** [partial/OWN-OPEN] own audit: 9 open, 1 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-entity-pas.md) docs/tracker/taudit-crates-pumpkin-core-src-entity-pas.md
- **own/tentities-panda-714331** [partial/OWN-OPEN] own audit: 3 open, 0 fixed, 0 verified, 0 blocked-data (docs/tracker/tentities-panda-714331.md) docs/tracker/tentities-panda-714331.md
- **own/tentities-panda** [partial/OWN-OPEN] own audit: 5 open, 5 fixed, 0 verified, 0 blocked-data (docs/tracker/tentities-panda.md) docs/tracker/tentities-panda.md

## Result

Final (third) pass. Every row below is closed as VERIFIED against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/panda/Panda.java` (plus `Mob.java`,
`Level.java`, `MeleeAttackGoal.java`). No source change was needed: the current `panda.rs` matches
vanilla on every row, and each seam already has a covering test. No generated data, WIT or
cross-crate change is required.

| row | result | vanilla evidence |
|---|---|---|
| entities/Panda (re-review) | VERIFIED | Re-read the whole `Panda.java`. `panda.rs` matches on genes (`setGeneFromParents` 566-590), gene weights (`Gene.getRandom` 755-770), mobInteract food gate (644), bamboo box (`canFindBamboo` 834-852), `afterSneeze` (506-531), `handleRoll` (485-504) and bite sound (`playAttackSound` 333-335 via `Mob.doHurtTarget`). |
| own/taudit-crates-pumpkin-core-src-en-7bf7cf | VERIFIED | All 4 rows closed in its own doc: independent gene mutation (Panda.java:583,587 -> panda.rs:164,167), sit-and-eat gate (644 -> panda.rs:627), bamboo box (834-852), afterSneeze island note at parity with `has_audited_tick` false. |
| own/taudit-crates-pumpkin-core-src-entity-pas | VERIFIED | All 5 rows closed in its own doc: genes, bamboo range, mobInteract, attack sound and sneeze reach each match vanilla. |
| own/tentities-panda-714331 | VERIFIED | All 6 rows closed in its own doc: the remembered-vanilla claims for rows 2-4 were contradicted by the decompiled class; current source is at parity. |
| own/tentities-panda | VERIFIED | All 7 rows closed in its own doc: genes, roll hops, bamboo, mobInteract, thunder, sneeze reach and bite all match vanilla. |

Confirmations re-read directly this pass from
`.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/panda/Panda.java`:
- `setGeneFromParents` (566-590): two independent `if (nextInt(32) == 0)` mutation rolls (583, 587)
  -> Rust `genes_from_parents` two independent `if next_int(MUTATION_CHANCE) == 0` (panda.rs:164,167),
  `GENE_RANDOM_BOUND = 16`.
- `Gene.getRandom` (755-770): 0 Lazy, 1 Worried, 2 Playful, 4 Aggressive, `<9` Weak, `<11` Brown,
  else Normal -> Rust `get_random_gene` (panda.rs:122-131), including `3 | 5..=8` Weak and `9 | 10` Brown.
- `mobInteract` food gate (644): `isSitting() || isInWater()` after the baby pass -> Rust
  `sit_and_eat` `is_baby() || is_sitting() || is_in_water()` (panda.rs:627).
- `canFindBamboo` (834-852): yOff 0..3, square rings r 0..8 -> x,z in [-7,7], y in [0,2], the same
  box as Rust `BAMBOO_SEARCH_RADIUS = 7` / `HEIGHT = 2`.
- `afterSneeze` (506-531): SNEEZE particle is a server no-op, PANDA_SNEEZE at 1.0/1.0, jump for
  every non-baby, grounded, out-of-water, `canPerformAction` panda in `inflate(10)`, gift only under
  MOB_DROPS -> Rust `after_sneeze` matches; pandas keep `has_audited_tick` false so the conservative
  `unaudited_reach` covers the 10-block jump.
- `handleRoll` (485-504): hop steps 7/15/23 set `(0, onGround ? 0.27 : movement.y, 0)` -> Rust
  `roll_velocity` keeps the vertical speed and zeros the horizontal push on hops.
- `playAttackSound` (333-335) fires PANDA_BITE only from `Mob.doHurtTarget` on a landed hit -> Rust
  `bite()` calls `on_attack` only when `try_attack` returns `damaged`.

No `.rs` source changed, so no rustfmt run and no new pure test: every seam (gene mutation, gene
weights, bamboo box, sit-and-eat, sneeze jump range/gift, roll velocity, bite sound) already has a
covering pure/world test.

Status: PASS fixed=0 open=0 files=docs/tracker/tpar-crates-pumpkin-core-src-entity-passive-panda-rs.md
