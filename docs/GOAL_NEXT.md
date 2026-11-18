# GOAL: perfection and quality standards (next after GOAL_RELEASE, limit 4000 chars)

Raise Kabocha from "builds and matches on paper" to "provably vanilla-faithful and robust". Done means each bar is met with automated evidence.

1. Close the open rows. Every STILL OPEN row in PARITY.tsv is either fixed with a test that fails without the fix, or reclassified BLOCKED-DATA with the exact missing field/subsystem. Then unblock data: add missing extractor outputs to assets/, regenerate pumpkin-data, and reopen the BLOCKED-DATA rows it frees. Subsystems repeatedly blocked (entity sub-predicate dispatch, Breeze brain behaviours, push reaction, shelf shapes, last_hurt_mob) each get an owner slice.
2. Differential testing against vanilla. Build a harness comparing Pumpkin to extracted vanilla data (loot tables, recipes, block states, collision shapes, worldgen chunk dumps, packet bytes). Fail CI on any diff. Add `pumpkin-gametest` scenarios for fire spread, villager trading, mob AI goals, redstone, fluids.
3. Bot-driven soak. Agent-driven bots join, move, build, fight, trade and disconnect for 30 minutes at 20 TPS target; assert no panic, no tick over 50 ms p99, no memory growth over a fixed bound, no desync. Record results in docs/investigations/.
4. Quality gates stay green permanently: clippy pedantic -D warnings, tests, typos, WIT sync, Nix flake, versioning, release lints, `cargo deny`/audit, no new `unwrap`/`panic`, every function under 100 lines. Add a pre-push `scripts/check.sh -f` for touched crates.
5. Performance budget. Benchmark chunk gen, lighting, entity tick, pathfinding, packet encode; commit baselines; fail on more than 5 percent regression. Never block Tokio workers or hold guards across await.
6. Security and robustness. Bound every allocation from network/disk/plugins, fuzz packet decoders and NBT/anvil/datapack parsers, handle malformed input without panics, review plugin WIT surface for additive-only changes.
7. Docs and ops. docs/ matches reality (GOAL, QUEUE, LEARNINGS, BUILD), pitfalls recorded, repeated procedures promoted to scripts/ and .agents/skills/, hourly upstream sync running and logged in docs/SYNC.md, release workflow producing signed binaries.
8. Report: per-bar status table, metrics before/after, remaining risk. Parity is checked mechanically, never by plausibility.
