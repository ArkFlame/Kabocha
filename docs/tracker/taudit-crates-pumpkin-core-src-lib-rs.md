# Audit: crates/pumpkin-core/src/lib.rs

Tracker: category=audit, coverage=AUDIT. No TODO/FIXME markers exist in the file; the
tracker's "2 vanilla markers" are the `vanilla_data` constructor parameter (lines 326/334).

## Fixed in this pass

- lib.rs:619 and server/mod.rs:942 - shutdown kick used the literal text "Server stopped".
  Vanilla `MinecraftServer.stopServer` disconnects players with
  `Component.translatable("multiplayer.disconnect.server_shutdown")` ("Server closed").
  Both sites now send the generated `translation::java::MULTIPLAYER_DISCONNECT_SERVER_SHUTDOWN` key.
  The login-phase kick in `Server::add_player` also uses that vanilla key.
  No test: the change only swaps a constant component for the vanilla translation key; a test
  would mirror the constant without exercising behaviour.

## Remaining gaps (kept out of this pass)

| Location | Vanilla class | Gap | Suggested fix | Size |
|---|---|---|---|---|
| lib.rs:240 (`PacketHandlerResult::Stop => pending.close()`) | `ServerLoginPacketListenerImpl.tick` / `ServerConfigurationPacketListenerImpl` | On server stop during login/config, vanilla sends a state-appropriate disconnect packet with `multiplayer.disconnect.server_shutdown` before closing; Pumpkin closes the TCP socket silently. | Track the pending connection's protocol state and call `pending.kick(TextComponent::translate(translation::java::MULTIPLAYER_DISCONNECT_SERVER_SHUTDOWN, []))` on the login/config state before `close()`. | M |
| lib.rs:599-636 (`shutdown`) | `MinecraftServer.stopServer` | Vanilla logs and broadcasts "Stopping the server", "Saving players", "Saving worlds" milestones and removes all players from the list after kicking. Pumpkin logs its own wording and relies on connection tasks to remove players. Cosmetic log parity only. | Optional: align info lines with vanilla; no behaviour change. | S |
| lib.rs:839 (`rl.readline("$ ")`) | `DedicatedServer` console thread | Vanilla's dedicated console has no prompt; Pumpkin prints a custom "$ " prompt. Cosmetic, config-gated `use_tty`. | Leave as is unless strict console-output parity is wanted. | S |
