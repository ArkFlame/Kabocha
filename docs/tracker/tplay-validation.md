# Java play packet validation sweep

| Handler | Check | Status | Note |
|---|---|---|---|
| player_position / player_position_rotation | non-finite x/y/z, yaw/pitch | fixed | kick `multiplayer.disconnect.invalid_player_movement`, matches vehicle handler |
| player_rotation | non-finite yaw/pitch | fixed | same kick |
| select_trade | negative index cast to usize | fixed | ignored before offer index; out-of-range already guarded by `MerchantScreenHandler` |
| select_trade | event index truncated `i32 as u8` | open | `TradeSelectEvent` takes u8; needs plugin API change, not done |
| set_creative_slot | slot 1..=45, count <= max stack | ok | existing |
| set_held_item | slot 0..=8 | ok | kicks `Invalid held slot` |
| edit_book | page/total size vs config | ok | existing, `book_size` config |
| update_sign | chunk loaded, editable gate | ok | line length bounded by protocol codec |
| move_vehicle | non-finite values, distance | ok | existing |
| interact, use_item_on, player_action, pick_item, chat_command, container click, rename, command block, jigsaw, structure block, client_command, player_input, recipe book, spectate | not audited in this pass | open | left for follow-up |
