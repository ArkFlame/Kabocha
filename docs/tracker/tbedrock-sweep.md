# Bedrock packet sweep

Scope: `crates/pumpkin-core/src/net/bedrock` and `crates/pumpkin-protocol/src/bedrock`.
Status: UNVERIFIED (no cargo run in this pass; wire values from Bedrock protocol knowledge).

## Checked and matching vanilla Bedrock

| Area | Pumpkin | Bedrock reference | Result |
|---|---|---|---|
| PlayerActionType | `server/player_action.rs` | start break 0, abort 1, stop break 2, respawn 7, jump 8, continue break 27 | match, test added |
| TitleType | `client/set_title.rs` | clear 0 ... actionbar text object 8 | match, test added |
| TextPacketType | `server/text.rs` | raw 0 ... json announcement 11 | match (explicit discriminants) |
| Window ids | `server/inventory_transaction.rs` | offhand 119, armour 120, UI 124 | match |
| Transaction types | `server/inventory_transaction.rs` | normal 0, mismatch 1, item use 2, item use on entity 3, item release 4 | match |
| ContainerClose | `server/container_close.rs` | window id byte, type byte, server-initiated bool | match |
| ModalFormResponse | `server/modal_form_response.rs` | form id varuint, optional json, optional cancel byte | match (cancel reason not enum) |

## Divergences logged, not changed

- `server/command_request.rs` `CommandOriginData.type` is decoded as a string. Bedrock sends a varuint
  CommandOriginType (player 0, command block 1, ..., automation player 5, dedicated server 7). Core does
  not read the origin type, so impact is low. Needs a captured packet before changing the wire layout.
- `server/inventory_transaction.rs` reads a bool-prefixed `legacy_set_item_slots` and bool-prefixed
  `window_id` / `source_flags`. The existing decode test vectors are captured bytes that include the bool
  prefix, so the layout is left as-is.
- `InventoryActionSource` lacks `Global = 1` and `UntrackedInteractionUI = 100`. Both decode to
  `Unknown(n)`, which nothing branches on today.
- `server/modal_form_response.rs` cancel reason is a raw `u8`. Bedrock values: closed 0, busy 1. No consumer.
- `play/player_action.rs` groups `ContinueDestroyBlock` with `StartDestroyBlock`. Ordering is handled by
  the mining position check, so behaviour is kept.

## Not checked

- Movement validation modes and `player_auth_input.rs` jump code (out of scope).
- Form response and `set_title` handler call sites beyond the enum mappings.
- Furnace XP row (skipped).
