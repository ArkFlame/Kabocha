# Soak and robustness

## Tools in the repo
- `tools/pumpkin-fuzzer`: external TCP fuzzer (`--host`, `--port`, `--mode raw|framed|stateful|corrupt-varint|all`, `--duration`). It talks to a running server and does not link server code.
- `tools/pumpkin-tester`: in-process server plus headless bots (`HeadlessBot` in `src/bot.rs`: join, move, use block, dig, command) and Mojang GameTests. Runs through `cargo run -p pumpkin-tester`.
- `scripts/soak.sh [minutes]`: boots the debug server on a fresh world, samples `VmRSS` from `/proc/<pid>/status` every 10 s, stops it, and fails on panic lines in the log.
- `scripts/boot.sh`: 90 s boot smoke test (no RSS, no bots).

## Tick time
`Server::get_tick_times_nanos_copy()` returns the last 100 tick durations; the rolling 1/5/15 minute rings and `aggregated_tick_times_nanos` back `/tps` and `/tick`. A soak that needs p99 must compute it from the 100-sample copy (or the 1 minute ring).

## Not implemented yet
- Bot load in soak mode (`--soak <minutes>` in `pumpkin-tester`): join, move, place and break, attack, disconnect and rejoin. Reuse `HeadlessBot`; the tester main loop does not run it for a duration yet.
- p99 MSPT gate (fail above 50 ms). It needs a tick-time read path from the tester.
- RSS growth gate. `soak.sh` records the samples but does not fail on growth.

## Robustness tests (no fuzzer dependency)
Deterministic never-panic tests feed a fixed-seed xorshift and mutated seeds into:
- NBT readers: `crates/pumpkin-nbt/src/robustness_tests.rs`
- serverbound packet decoding (`SClickSlot`, `SChatCommandSigned`, `SSetGameRule`): `crates/pumpkin-protocol/src/java/server/robustness_tests.rs`
- anvil chunk records and decompression: `crates/pumpkin-world/src/chunk/format/anvil/robustness_tests.rs`

Parser bounds added: `read_var_i32_array_from_slice` and `read_var_i64_array_from_slice` (`crates/pumpkin-nbt/src/deserializer.rs`) now pre-allocate at most 4096 elements from the wire length prefix. Other length-prefixed reads already had bounds (`MAX_ARRAY_LENGTH`, `MAX_LIST_SIZE`, `BitSet::MAX_LENGTH`, `MAX_CHANGED_SLOTS`, anvil sector limits, `MAX_DECOMPRESSED_CHUNK_SIZE`).

Known unbounded case, not changed: `read_gzip_compound_tag` (`crates/pumpkin-nbt/src/nbt_compress.rs`) reads the full decompressed stream into a Vec. It is only used for local files, so it is not a network risk.

## Bot soak (`pumpkin-tester --soak <minutes>`)
Runs 8 headless bots moving in circles against a real `Ticker` loop, samples the tick ring every second, and fails when MSPT p99 exceeds 50 ms. Prints RSS first/max/last. Gate on a release build: `scripts/cargo.sh run --release -p pumpkin-tester -- --soak 30`. Debug smoke (1 min, 8 bots) measured p99 74.7 ms, RSS 221 to 265 MB, so debug is not the gate.
