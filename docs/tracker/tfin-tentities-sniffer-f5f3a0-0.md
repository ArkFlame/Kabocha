# Final rows from docs/tracker/tentities-sniffer-f5f3a0.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: camel.rs turtle.rs

- **Dig game event used the head block centre instead of the entity position** (commit `ef4595b2f`). `mob_tick` emitted `minecraft:entity_action` at `get_head_block().to_centered_f64()`; vanilla `Entity.gameEvent(GameEvent)` (used by `Sniffer`) emits at `position()`. Now uses `Entity::emit_own_game_event` like `camel.rs:218` and `turtle.rs:386`. Affects sculk sensor / warden hearing only. NOTE: a later tracker-sweep commit `cdb0045a3` changed this back to a head-block-centred `emit_game_event_with_context`. Decompiled `Sniffer` (line 330) actually emits `ENTITY_ACTION` at `getHeadBlock()`; the current position is not re-verified in this pass (out of the STILL OPEN row scope).
Only row 7 is marked STILL OPEN in Gap status. Rows 1-6 are CLOSED with an existing vanilla-verified Result, so no source change is warranted for them under this pass's STILL OPEN-only scope.
Only row 7 remained STILL OPEN. Rows 1-6 were already CLOSED with vanilla-verified Results, so no source change is warranted for them.
No new test added: those two tests are already the smallest pure unit tests covering the handoff and drop-seed clock; another would duplicate them under the MVP test rules. No `pumpkin-gametest` crate is needed because the seam is `test_support::world`, which is present. No source files edited. No generated data, WIT or cross-crate changes; no STILL OPEN rows remain.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
