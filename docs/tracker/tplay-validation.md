# Java play packet validation sweep

| Handler | Check | Status | Note |
|---|---|---|---|
| player_position / player_position_rotation | non-finite x/y/z, yaw/pitch | fixed | kick `multiplayer.disconnect.invalid_player_movement`, matches vehicle handler |
| player_rotation | non-finite yaw/pitch | fixed | same kick |
| select_trade | negative index cast to usize | fixed | ignored before offer index; out-of-range already guarded by `MerchantScreenHandler` |
| select_trade | event index truncated `i32 as u8` | fixed | event is skipped for indices outside `u8`; selection still bounded by `MerchantScreenHandler` |
| set_creative_slot | slot 1..=45, count <= max stack | ok | existing |
| set_held_item | slot 0..=8 | ok | kicks `Invalid held slot` |
| edit_book | page/total size vs config | ok | existing, `book_size` config |
| update_sign | chunk loaded, editable gate | ok | line length bounded by protocol codec |
| move_vehicle | non-finite values, distance | ok | existing |
| interact | entity reach `entity_interaction_range + 1.0` on eye-to-hitbox | ok | `can_interact_with_entity`, unknown entity only warns |
| use_item_on | reach 4.5+1, hit within block, face/hand, sequence | ok | cursor check runs before face/hand, silent drop like vanilla |
| sign_update | chunk loaded, editable gate, line length | ok | line length bounded by codec |
| edit_book | page count 100, page 1024, title 32 | ok | codec rejects over-limit; pages > 100 rejected, not truncated |
| edit_book | writes held item, ignores packet slot | open | vanilla writes `slot` (hotbar or 40); log only, behaviour kept |
| rename_item | 50 chars | ok | `validate_name` in anvil handler |
| container_click | slot -999/-1/0..size, swap button 0..8/40, mode | ok | `is_slot_valid` mirrors vanilla `isValidSlotIndex`; negative slots guarded in inventory |
| container_click | changed-slot cap 256 | open | vanilla cap unverified (believed 128); log only |
| container_button | window id match | ok | button id bounds left to the screen handler |
| set_command_block / set_jigsaw / set_structure_block | creative + op 2 | ok | matches `canUseGameMasterBlocks` |
| set_structure_block | offset/size clamp | open | not clamped in handler; vanilla behaviour unverified |
| player_command | horse jump 0..100 | ok | `player_jump_pending_scale` clamps |
| player_input | sneak/flags | ok | delegates to `apply_client_input` |
| pick_item from block | reach 1.0, creative include data | ok | |
| pick_item from entity | 64 sq distance | open | vanilla distance unverified |
| recipe_book_seen_recipe | recipe existence | open | event fires for any id; log only |
| chat_session_update | validation | ok | `validate_chat_session` |
| custom_payload | 32767 bytes | ok | codec `MAX_PAYLOAD_SIZE` |
| client_command / player_action / chat_command / spectate | not re-audited | open | left for follow-up |
