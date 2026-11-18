# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-net-java-m.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: mod.rs

- Row 1 (command/chat/session ordering): FIXED (see Result).
- Row 5 (unknown packet id `warn!`): FIXED (see Result).

## Result
Both rows were already fixed on disk by an earlier pass; re-verified against decompiled vanilla this pass. No new source edits were needed.

- Row 1 (command/chat/session ordering): FIXED, VERIFIED. Vanilla dispatches every packet of one connection in arrival order on the single server thread. See `.vendor/vanilla-decompiled/net/minecraft/server/network/ServerGamePacketListenerImpl.java` (`handleChat` 2552, `handleChatCommand` 2582, `handleChatAck` 2869, `handleChatSessionUpdate` 3905) and its `PacketUtils.ensureRunningOnSameThread` calls (e.g. 858/897/903). `crates/pumpkin-core/src/net/java/mod.rs` now has `chain_after` (105-115), a per-connection `deferred_packet_tail: Mutex<Option<JoinHandle<()>>>` (189, 233) and `enqueue_ordered_packet` (404-415); the `SChatAck`/`SChatCommand`/`SChatCommandSigned`/`SChatMessage` (942/952/967/989) and `SPlayerSession` (1241) arms all enqueue through it instead of `spawn_task`. The only remaining `spawn_task` calls are the chain itself (414) and the unrelated writer/respawn job (868). Unit test `a_queued_packet_runs_after_the_previous_one_finished` (1441-1458) covers FIFO ordering.
- Row 5 (unknown packet id `warn!`): FIXED, VERIFIED. Vanilla's decoder rejects an unmapped id during decode rather than logging it per packet: `.vendor/vanilla-decompiled/net/minecraft/network/PacketDecoder.java` 23-35 (`protocolInfo.codec().decode` throws; the exception closes the connection via `Connection.java` `exceptionCaught` 188). `crates/pumpkin-core/src/net/java/mod.rs` 1417-1424 replaced the old per-packet `warn!` with `try_kick(TextComponent::translate(translation::java::DISCONNECT_PACKETERROR, Vec::new()))`; the `disconnect.packetError` key exists at `crates/pumpkin-data/src/generated/translation.rs` 8441. No per-packet `warn!` remains in the play dispatcher.

No generated-data fields or subsystems are missing, so neither row is BLOCKED-DATA. No cargo run (workers may not build); orchestrator compiles once and runs rustfmt on touched files.
