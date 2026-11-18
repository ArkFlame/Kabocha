# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-net-java-m.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: mod.rs

- Row 1 (command/chat/session ordering): VERIFIED fixed on disk (see Result).
- Row 5 (unknown packet id `warn!`): VERIFIED fixed on disk (see Result).

## Result

Ground truth for row 1:
`.vendor/vanilla-decompiled/net/minecraft/network/protocol/PacketUtils.java:17-34`
(`ensureRunningOnSameThread` schedules every inbound packet onto the tick thread via
`player.getBukkitEntity().schedulePacket(...)`, preserving FIFO arrival order) and
`.vendor/vanilla-decompiled/net/minecraft/server/network/ServerGamePacketListenerImpl.java:2582`
(`handleChatCommand` runs on that thread). Ground truth for row 5:
`.vendor/vanilla-decompiled/net/minecraft/network/PacketDecoder.java:23-35` (the codec's
`decode` throws on an id it cannot resolve, with no per-packet warning log).

- Row 1 (command/chat/session ordering): FIXED, VERIFIED. `crates/pumpkin-core/src/net/java/mod.rs`
  defines `chain_after` (105-115), a `deferred_packet_tail: Mutex<Option<JoinHandle<()>>>` field
  (189, 233) and `enqueue_ordered_packet` (404-415); every `SChatAck`, `SChatCommand`,
  `SChatCommandSigned`, `SChatMessage` and `SPlayerSession` arm (946, 958, 980, 1001, 1246)
  enqueues through it instead of `spawn_task`. This preserves per-connection arrival order like
  vanilla's `ensureRunningOnSameThread`, without blocking a Tokio worker. Pure unit test
  `a_queued_packet_runs_after_the_previous_one_finished` (1431-1459) covers the chain. No code
  change needed in this pass; the fix was already on disk.
- Row 5 (unknown packet id `warn!`): FIXED, VERIFIED. The old per-packet `warn!` arm is gone;
  `mod.rs:1417-1424` now kicks with `disconnect.packetError` (translation key
  `DISCONNECT_PACKETERROR`), matching vanilla's decoder rejection of an unresolvable id.
  No per-packet `warn!` remains in the play dispatcher. No code change needed in this pass;
  the fix was already on disk.

No source files edited this pass (both fixes were already present and verified against the
vendored decompiled vanilla). No cargo run: workers may not build; the orchestrator compiles once.
