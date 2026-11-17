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

- `server/command_request.rs` origin type: fixed to a varuint `CommandOriginType` (UNVERIFIED, no cargo run).
  The player id is read only for DevConsole (3) and Test (4), else `-1`. Wire encoding of the id (LE i64)
  is kept from the prior code and is UNVERIFIED. No captured fixture exists for this packet.
- `server/inventory_transaction.rs` bool-prefixed `legacy_set_item_slots`, `window_id`, `source_flags`: VERIFIED
  as-is. The existing decode test vectors (captured bytes) include the bool prefixes, so the code matches
  fixtures. The bare-varint layout from the public spec is not used by the captured client.
- `InventoryActionSource` lacks `Global = 1` and `UntrackedInteractionUI = 100`. Both decode to
  `Unknown(n)`, which nothing branches on today.
- `server/modal_form_response.rs` cancel reason is a raw `u8`. Bedrock values: closed 0, busy 1. No consumer.
- `play/player_action.rs` groups `ContinueDestroyBlock` with `StartDestroyBlock`. Ordering is handled by
  the mining position check, so behaviour is kept.

## Java play sweep

Checked `crates/pumpkin-core/src/net/java/play/*`: slot ranges (`set_held_item` 0..=8, creative slot 1..=45),
sign lines (bounded 384 at decode), crafter slot index (bounds in `slot_can_be_disabled`), float finiteness
(`move_vehicle`, `use_item`, `confirm_teleport`), chat timestamp/signature. No new gap found; no change.

## Not checked

- Movement validation modes and `player_auth_input.rs` jump code (out of scope).
- Form response and `set_title` handler call sites beyond the enum mappings.
- Furnace XP row (skipped).
