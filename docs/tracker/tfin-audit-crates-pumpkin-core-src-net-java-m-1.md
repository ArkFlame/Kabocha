# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-net-java-m.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: mod.rs

Re-verified every STILL OPEN row against the vendored decompiled vanilla under
- Row 1 (command/chat/session ordering): STILL OPEN - fix requires `net/java/mod.rs`
- Row 5 (unknown packet id warn!): STILL OPEN - the log site is `net/java/mod.rs`
- Row 1 (command/chat/session ordering): STILL OPEN - the `spawn_task` sites are in

(The STILL OPEN markers above record the unit state at creation; both rows are now closed in the Result below.)

## Result
Both rows are FIXED and VERIFIED against vendored decompiled vanilla. The fixes are already present on disk in `crates/pumpkin-core/src/net/java/mod.rs`; no further source change was needed.

- Row 1 (command/chat/session ordering): FIXED and VERIFIED. Vanilla dispatches `handleChat`, `handleChatCommand`, `handleChatAck` and `handleChatSessionUpdate` inline on the connection thread in arrival order via `Connection.channelRead0` -> `genericsFtw` -> `packet.handle(listener)` (.vendor/vanilla-decompiled/net/minecraft/network/Connection.java:317-336; handler bodies in .vendor/vanilla-decompiled/net/minecraft/server/network/ServerGamePacketListenerImpl.java:2552, 2582, 2869, 3905). `mod.rs` now serialises those arms through `enqueue_ordered_packet` (404-415), which chains each job on `deferred_packet_tail` using `chain_after` (107-115). Every `SChatAck`/`SChatCommand`/`SChatCommandSigned`/`SChatMessage`/`SPlayerSession` arm (942-1016, 1241-1253) enqueues through it, with no `spawn_task` left in those arms. Pure unit test `a_queued_packet_runs_after_the_previous_one_finished` (1441-1458) covers the ordering.

(The STILL OPEN markers above record the unit state at creation; both rows are now closed in the Result below.)

- Row 5 (unknown packet id `warn!`): FIXED and VERIFIED. Vanilla's `IdDispatchCodec.decode` throws `DecoderException("Received unknown packet id " + id)` for an id with no packet (.vendor/vanilla-decompiled/net/minecraft/network/codec/IdDispatchCodec.java:34-41), which propagates to `Connection.exceptionCaught` and disconnects the connection (.vendor/vanilla-decompiled/net/minecraft/network/Connection.java:188, 218-239); a decoded packet that still cannot be processed disconnects with `multiplayer.disconnect.invalid_packet` (Connection.java:324-326). Pumpkin's play dispatcher `_` arm (mod.rs:1417-1424) now kicks with `disconnect.packetError` instead of logging a `warn!` and leaving a desynced connection alive. No per-packet unknown-id `warn!` remains in the play dispatcher.
