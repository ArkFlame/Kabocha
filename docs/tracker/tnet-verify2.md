# TNET verify 2: remaining play handlers vs vanilla ServerGamePacketListenerImpl

Continues `docs/tracker/tnet-verify.md` (x6). Vanilla source:
`.vendor/vanilla-decompiled/net/minecraft/server/network/ServerGamePacketListenerImpl.java`.
Build not run in this pass (worker rule); all edits are unverified by cargo.

## Fixed (kab/x8)

| Area | Vanilla | Pumpkin before | Change |
|---|---|---|---|
| Creative drop spam | `dropSpamThrottler = TickThrottler(20, 1480)`, excess drops warned and ignored | every negative-slot packet dropped | `CreativeDropThrottle` per connection (`net/java/mod.rs`, `play/set_creative_slot.rs`), decays from `world_age`; tests `burst_drops_stop_at_threshold`, `count_decays_one_per_tick` |
| Sign line formatting | `ChatFormatting.stripFormatting` on all four lines before filter and update | `§` codes kept | `strip_formatting` in `play/update_sign.rs`; tests `strips_legacy_format_codes`, `keeps_non_format_sequences` |
| Sign edit while dead | `updateSignText` returns on `isImmobile()` (dead or dying) before action time reset | no check | early return in `handle_sign_update` |
| Container click while dead | `handleContainerClick` returns on `isImmobile()` before action time reset | action time reset, click processed | early return at top of `Player::on_slot_click` (`entity/player.rs`) |
| Command suggestion, first word | suggests command names (range starts after `/`) | returned nothing without a space | `last_word_start` helper returns 0 when no whitespace; tests `first_word_starts_at_zero`, `argument_starts_after_last_space` |
| Client information view distance | `viewDistance < 0` disconnects "Invalid client settings" (Paper guard, matches vanilla intent) | `as u8` wrapped -1 to 255 | kick in `play/client_information.rs`; no test (single trivial branch) |
| Idle timeout boundary | `Util.getMillis() - last > minutes` (strict) | `>=` | `>` in `Player` idle check (`entity/player.rs`); no test (ms boundary) |

## Verified matching (no change)

- Chat ack: `applyOffset` rejects negative and offsets above `size - lastSeenCount`, kicks on failure
  (`play/chat_ack.rs` plus `Player` `apply_offset`).
- Chat ack does not reset action time in either implementation.

## Blocked

- Creative feature gate (`itemStack.isItemEnabled(enabledFeatures)` in `handleSetCreativeModeSlot`):
  needs per-item `requiredFeatures` from the Extractor. `assets/items.json` has no feature flags and
  `pumpkin-data` `Item` has no field for them. Mark blocked (needs Extractor). Pumpkin already has
  `get_enabled_features()` for the datapack side.

## Divergences logged, not fixed

- Command suggestion Paper throttles: `tabSpamThrottler` and the 64-char first-word spam disconnect are
  Paper-only (`Paper start - throttle tab completions`). Kabocha targets vanilla behaviour, so not ported.
- Container click: the rest of vanilla `handleContainerClick` (state id resync via `fullResyncNeeded`,
  spectator and dead `sendAllDataToRemote`, CraftBukkit click events) was not audited line by line in
  `Player::on_slot_click`. Only the immobile gate was checked.
- Sign: Paper `limit-client-sign-length` (`MAX_SIGN_LINE_LENGTH`) is a Paper config, not ported.
- Idle timeout: vanilla skips kick when `player.wonGame`. Pumpkin has no `wonGame` equivalent in this path.
- Client information in the config phase (`pending.rs`) still wraps a negative view distance; vanilla
  guards only the play handler.

## Not verified

- No `cargo` run (worker rule). Required before landing: `scripts/check.sh pumpkin-core` (fmt + check) and
  `scripts/check.sh -f pumpkin-core` for the new unit tests.
