# TODO 2: parity-driven work (from the Pumpkin Discord, 2026-11-16, and the parity tracker)

Sources: a Pumpkin Discord thread (worldgen math determinism, parity tracker, vanilla bugs) and the live tracker
`pumpkinmc.org/tracker/` whose data is `Pumpkin-MC/Pumpkin-Website/src/pages/tracker/data.json` (fetch with
`gh api repos/Pumpkin-MC/Pumpkin-Website/contents/src/pages/tracker/data.json -H "Accept: application/vnd.github.raw"`).
Snapshot: version 26.3-26.63, updated 2026-11-16, pumpkin commit 79c88fdb. Complements docs/TODO.md and docs/QUEUE.md.

## 1. What the tracker says (the vanilla parity scoreboard)

| Category | Entries | Done | Partial | Planned |
|---|---|---|---|---|
| Entities | 160 | 79 | 81 | 0 |
| Blocks | 201 | 174 | 27 | 0 |
| Items | 54 | 48 | 6 | 0 |
| Commands | 92 | 92 | 0 | 0 |
| Redstone | 27 | 27 | 0 | 0 |
| Combat | 14 | 13 | 0 | 1 (1.8 combat mode) |
| World | 16 | 16 | 0 | 0 |

Rules of the tracker: "done" means a source review against vanilla; registry presence alone is not parity. Many `partial`
rows only say "the reviewed source has changed, needs a new review": those are re-review jobs, not proven gaps.
Entries link upstream issues, so the tracker is a ready-made TODO-vs-vanilla ledger.

## 2. Concrete gaps named by the tracker (implement in this order: smallest first)

Entities with a stated missing behaviour (each is one worker slice with a vanilla-parity brief):
- Warden: vibration listening, sniffing, anger-based targeting, sonic boom (issue #4122, branch kab/i4122 already holds a slice).
- Silverfish: infested-block break scan on countdown end, stone merging. Infested blocks: break constructs a generic entity.
- Vex: flying charge, owner-target copy, summoned lifetime. Illusioner: no bow goal, no spell state. Ravager: stun/roar.
- Skeleton: bow interval fixed 20 ticks, vanilla reassesses by difficulty. Stray and Parched: plain ARROW, need slowness/variant payload.
- Spider and Cave Spider: LeapAtTargetGoal and wall climbing missing.
- Hoglin: repellent/retreat activities. Piglin Brute: home and return-to-home. Piglin: aggro bug (#3376).
- Witch: target selection lacks injured-raider healing. Zoglin: target constructor scoped to PLAYER only.
- Bee: hive, pollination, crop behaviour not wired. Blaze: no movement toward restricted area; also #1960.
- Ender Dragon: dragon parts constructed but not registered in the world; collision incomplete (#2220 landed a slice).
- Drowned: #3069 (trident range), #2162 (burn in water). Enderman: #2872. Sulfur Cube: no AI (#3114).
- About 45 further entities (Creeper, Guardian, Wither, Fox, Panda, Axolotl, Goat, Camel Husk, Copper Golem, Happy Ghast,
  Nautilus, Painting, Armor Stand, Item, ...) are "needs re-review": run a re-review audit worker per entity, then file the
  concrete gaps it finds. Boats and rafts (about 36 variants) are one shared slice.
Blocks: Sculk Catalyst (death listener, bloom spread), Sculk Shrieker, Sculk Sensor, Moving Piston (player context lost),
Ender Chest (piglin anger on open), Infested blocks, Dirt Path, Farmland, Tripwire, plus 14 re-review rows (Beacon, Beds,
Beehive, Campfire, Chests, Chiseled Bookshelf, Creaking Heart, Jukebox, Lodestone, Note Block, Shulker Box, Structure Block,
Vault, Fire, Pistons, Piston Head, Daylight Detector, Dispenser).
Items: Fishing Rod, Mace, Compass and Recovery Compass, Elytra, Goat Horn, Name Tag (all "re-review").
Combat: 1.8 combat mode is planned (no source yet).

## 3. What the Discord thread taught us

- **World generation determinism.** Vanilla Java output can differ by CPU: the JVM picks between Intel math stubs and fdlibm
  by CPU. Measured (Intel i5-13420H vs AMD Zen 1, 1M inputs per function, JDK 17/21/25): `log`, `exp`, `sin`, `cos`, `tan` identical;
  `log10` differs in 18 of 1M values, `pow` in 6 of 1M, each by 1 ulp. Impact: a block only changes where a noise value lands exactly
  on a threshold, which is extremely rare. Our chunks equal an AMD vanilla server; vanilla against itself on one seed is 99.954% block-equal.
  The remaining gap is not yet attributed (rounding versus other causes); a `-Xint` (interpreter-only) vanilla run separates
  intrinsic rounding from algorithmic differences.
- **Order of terrain generation matters a lot** (feature and decoration order decides tree placement and overlaps); trees can differ
  between vanilla runs depending on the direction a chunk is generated from. Keep decoration order identical to vanilla.
- **Pumpkin should replicate vanilla bugs**, including at least two short/int overflow bugs. Do not "fix" vanilla quirks silently.
- Sculk in deep dark was fixed by upstream PR #2882; anything else seen in the thread was filed as issues. Practice worth copying:
  AI audits find candidates, humans (here: us) file concrete issues, and the audit output is actually read.

## 4. Next things to do (new work, in priority order)

1. **Parity ledger from the tracker.** Add `scripts/parity.sh`: download data.json, write docs/PARITY.tsv (category, name, status,
   issues, note) and a diff against the previous snapshot. Use it as the source for "re-review" and "gap" slices and to report
   progress in percent (done / total per category). Refresh hourly with scripts/sync-upstream.sh.
2. **Re-review audit workers.** One OpenCode worker per "needs re-review" row: compare the Rust source with decompiled vanilla,
   return a list of concrete gaps with file:line, and either file them in docs/QUEUE.md or hand them to implementers (scripts/impl.sh).
3. **Math parity harness.** Add a differential test of `log10` and `pow` (as used by worldgen noise) against Java `StrictMath`
   vectors; if Rust `f64::powf/log10` ever differ from fdlibm, port fdlibm `pow` and `log10` into pumpkin-world (pure Rust, no libm
   dependency on the platform) so output is identical on every CPU. Keep the test tiny: a few thousand golden values.
4. **Block-equality benchmark.** Script that generates N chunks for fixed seeds and compares block hashes against stored vanilla
   chunk dumps; track the percent (target at or above the 99.954% vanilla self-parity). Add a `-Xint` vanilla dump for the reference.
5. **Gap slices from section 2**, smallest first, each with a vanilla-parity brief and the automated test that fails without it.
6. **Vanilla-bug registry.** docs/VANILLA_BUGS.md listing quirks we deliberately keep (short/int overflows and others), with the
   vanilla class and method, so workers never "fix" them.
7. **1.8 combat mode**: a planned feature (attack cooldown removed, sword block); needs a design note before code.
