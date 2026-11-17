# Plugin API / wasm host hygiene sweep

Scope: `crates/pumpkin-wasm-host-v0_1`, `crates/pumpkin-wasm-host-v0_2`, `crates/pumpkin-core/src/plugin`.

## Done

- v0_1 recursion warning (`player.rs`, `overflow evaluating the requirement`): added `#![recursion_limit = "256"]` at the v0_1 crate root (`crates/pumpkin-wasm-host-v0_1/src/lib.rs`). Not compiled (UNVERIFIED): the warning is an auto-trait proof overflow from the nested async `kick` / `pump_reentry` future chain, and raising the limit is the lowest-risk fix.

## Open

- unwrap/expect/panic/println sweep: about 1850 matches across the three trees, most under crate-level `#![allow(clippy::unwrap_used, ...)]`. Not mechanically replaced (no compile, behaviour risk). Remaining non-test sites worth a follow-up:
  - `crates/pumpkin-wasm-host-v0_1/src/world.rs` and `crates/pumpkin-wasm-host-v0_2/src/world.rs`: `panic!` on a failed Wasm chunk generation phase. Kept: converting to a log changes chunk-gen failure behaviour.
  - `crates/pumpkin-core/src/plugin/mod.rs` (~line 1231, 1238): `println!` for the interactive permission prompt, which goes through readline. Kept: needs a prompt path, not `tracing`.
- Events in `plugin/api/events/`: fire point and `cancelled()` honouring were not audited in this pass. Follow-up needed.
- WIT additivity vs `upstream/master` (`crates/pumpkin-plugin-wit`):
  - Removed records (present upstream, absent in HEAD): `bedrock-connection-packet-received-event-data`, `bedrock-connection-packet-sent-event-data`, `java-connection-packet-received-event-data`, `java-connection-packet-sent-event-data` (v0.2 `event.wit`), and `entity-toggle-sit-event-data`. Breaking for v0.2 plugins; needs restoring or a recorded decision.
  - Packet records: `properties: list<property>` appears 3 times per version upstream (v0.1 `java-packets.wit`, v0.2 `java-packets.wit`) and once in HEAD. Verify whether the removed packet fields were dropped or moved.
  - Doc comment removals (`world.wit`) are not API changes.
