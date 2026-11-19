# Benchmarks

Criterion benchmarks guard hot paths against slowdowns. Each bench is a
`harness = false` target declared in its crate's `Cargo.toml`, with
`criterion.workspace = true` under `[dev-dependencies]`. Copy the layout of
`crates/pumpkin-data/benches/recipe_matching.rs`.

## Current benches

| Crate | Bench | What it measures |
|---|---|---|
| `pumpkin-protocol` | `packet_encode` | `serialize_packet` for KeepAlive, EntityPositionSync and HeadRot (1.21.4) |
| `pumpkin-data` | `recipe_matching`, `has_random_ticks` | Generated lookup tables |
| `pumpkin-world` | `chunk_gen`, `chunk_gen_concurrent`, `chunk_io`, `noise_router`, ... | Generation stages, including `lighting_generation` |

Pathfinding has no bench yet. `WalkNodeEvaluator` needs a live world and mob,
and its only pure entry (`is_outside_home`) is a single comparison.

## Run

    scripts/bench.sh compare                 # protocol and data benches vs main baseline
    scripts/bench.sh compare pumpkin-world   # any crate with benches
    scripts/bench.sh record                  # re-record baselines for the default crates

`compare` fails when any benchmark's mean change exceeds 5 percent. It reads
`target/criterion/**/change/estimates.json` (`mean.point_estimate`, a fraction)
and only counts files written by the current run.

Benches run through `scripts/cargo.sh`, which uses the shared warm target and
its RAM guard. Never call `cargo bench` directly.

## Baselines

Criterion keeps raw baselines in `target/criterion/<group>/<bench>/main/`.
These are local and not committed. The committed record is
`benches/baselines/<crate>.json`, written by `scripts/bench.sh record`:

    {
      "crate": "pumpkin-protocol",
      "unit": "ns",
      "benchmarks": {
        "java_packet_encode/keep_alive": 12.3
      }
    }

To record or update a baseline:

1. Run `scripts/bench.sh record <crate>` on an idle machine, with no other
   builds running.
2. Check the numbers look sane, then commit `benches/baselines/<crate>.json`
   on its own with a `chore(bench)` subject.
3. Do not commit `target/`.

Baselines are machine-specific. Record them on the same host class you
compare on. A large move in the JSON with no related code change points to
the machine, not the code.

## Adding a bench

- Use fixed seeds and constant inputs so runs are deterministic.
- Keep each bench small. Target a few seconds per benchmark.
- Do not `unwrap` or `expect`. Wrap the result in `black_box` instead. If an
  unwrap is unavoidable, add a local `#[expect(clippy::unwrap_used, reason = "..")]`.
- Benches are checked by clippy with `--all-targets`, so keep them pedantic-clean.
