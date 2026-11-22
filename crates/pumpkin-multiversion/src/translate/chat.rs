//! Clientbound text components, chat, scoreboard, and scalar layout shifts.

use pumpkin_data::packet::CURRENT_MC_VERSION;
use pumpkin_protocol::{
    ClientPacket, ServerPacket, VarInt,
    java::client::play::{
        CChangeDifficulty, CDisplayObjective, CHeadRot, CKeepAlive, COpenSignEditor,
        CRemoveMobEffect, CSetEntityLink, CSetExperience, CSetHealth, CSetSelectedSlot,
        CTakeItemEntity, CUnloadChunk, CUpdateMobEffect,
    },
    ser::{NetworkReadExt, NetworkReadSliceExt, NetworkWriteExt},
};
use pumpkin_util::version::JavaMinecraftVersion;

use crate::legacy::{LegacyWriteExt, ids};

fn reencode<'a, P: ServerPacket<'a> + ClientPacket>(
    mut payload: &'a [u8],
    version: JavaMinecraftVersion,
) -> Option<Vec<u8>> {
    let packet = P::read(&mut payload, &CURRENT_MC_VERSION).ok()?;
    let mut out = Vec::new();
    packet.write_packet_data(&mut out, &version).ok()?;
    Some(out)
}

fn truncate_chars(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

/// Single `TextComponent` packet (`DISCONNECT`, `SET_ACTION_BAR_TEXT`, `SET_TITLE_TEXT`, `SET_SUBTITLE_TEXT`).
pub fn component_from_current(
    mut payload: &[u8],
    version: JavaMinecraftVersion,
) -> Option<Vec<u8>> {
    let comp = payload.get_component(&CURRENT_MC_VERSION).ok()?;
    let mut out = Vec::new();
    out.write_component_legacy(&comp, &version).ok()?;
    Some(out)
}

/// Legacy `TITLE` action for `version`: 1.11+ inserts actionbar at 2 and shifts the rest.
fn legacy_title_action(version: JavaMinecraftVersion, before_1_11: i32, from_1_11: i32) -> i32 {
    if version >= JavaMinecraftVersion::V_1_11 {
        from_1_11
    } else {
        before_1_11
    }
}

/// The combined `TITLE` packet id, or `None` before 1.8 when titles did not exist.
fn legacy_title_id(version: JavaMinecraftVersion) -> Option<i32> {
    let id = crate::legacy::ids::clientbound::play::TITLE.to_id(version);
    (id != -1).then_some(id)
}

/// Writes a legacy `TITLE` with a JSON text body (title, subtitle and actionbar actions).
fn legacy_title_text(
    mut payload: &[u8],
    version: JavaMinecraftVersion,
    action: i32,
) -> Option<(i32, Vec<u8>)> {
    let component = payload.get_component(&CURRENT_MC_VERSION).ok()?;
    let json = crate::legacy::text::to_json(&component, &version);
    let id = legacy_title_id(version)?;
    let mut out = Vec::new();
    out.write_var_int(&VarInt(action)).ok()?;
    out.write_string(&json).ok()?;
    Some((id, out))
}

/// `SET_TITLE_TEXT`: legacy `TITLE` action 0 before 1.17, split packet after.
pub fn set_title_text_from_current(
    payload: &[u8],
    version: JavaMinecraftVersion,
) -> Option<(i32, Vec<u8>)> {
    if version >= JavaMinecraftVersion::V_1_17 {
        let id = crate::legacy::ids::clientbound::play::SET_TITLE_TEXT.to_id(version);
        return Some((id, component_from_current(payload, version)?));
    }
    legacy_title_text(payload, version, 0)
}

/// `SET_SUBTITLE_TEXT`: legacy `TITLE` action 1 before 1.17, split packet after.
pub fn set_subtitle_text_from_current(
    payload: &[u8],
    version: JavaMinecraftVersion,
) -> Option<(i32, Vec<u8>)> {
    if version >= JavaMinecraftVersion::V_1_17 {
        let id = crate::legacy::ids::clientbound::play::SET_SUBTITLE_TEXT.to_id(version);
        return Some((id, component_from_current(payload, version)?));
    }
    legacy_title_text(payload, version, 1)
}

/// `SET_ACTION_BAR_TEXT`: legacy `TITLE` action 2 from 1.11, split packet from 1.17.
pub fn set_action_bar_text_from_current(
    payload: &[u8],
    version: JavaMinecraftVersion,
) -> Option<(i32, Vec<u8>)> {
    if version >= JavaMinecraftVersion::V_1_17 {
        let id = crate::legacy::ids::clientbound::play::SET_ACTION_BAR_TEXT.to_id(version);
        return Some((id, component_from_current(payload, version)?));
    }
    // 1.8-1.10 have no actionbar `TITLE` action (2 is times there).
    if version < JavaMinecraftVersion::V_1_11 {
        return None;
    }
    legacy_title_text(payload, version, 2)
}

/// `SET_TITLES_ANIMATION`: legacy `TITLE` times action before 1.17, split packet after.
pub fn set_titles_animation_from_current(
    mut payload: &[u8],
    version: JavaMinecraftVersion,
) -> Option<(i32, Vec<u8>)> {
    if version >= JavaMinecraftVersion::V_1_17 {
        let id = crate::legacy::ids::clientbound::play::SET_TITLES_ANIMATION.to_id(version);
        return Some((id, payload.to_vec()));
    }
    let fade_in = payload.get_i32_be().ok()?;
    let stay = payload.get_i32_be().ok()?;
    let fade_out = payload.get_i32_be().ok()?;
    let action = legacy_title_action(version, 2, 3);
    let id = legacy_title_id(version)?;
    let mut out = Vec::new();
    out.write_var_int(&VarInt(action)).ok()?;
    out.write_i32_be(fade_in).ok()?;
    out.write_i32_be(stay).ok()?;
    out.write_i32_be(fade_out).ok()?;
    Some((id, out))
}

/// `CLEAR_TITLES`: legacy `TITLE` hide/reset action before 1.17, split packet after.
pub fn clear_titles_from_current(
    mut payload: &[u8],
    version: JavaMinecraftVersion,
) -> Option<(i32, Vec<u8>)> {
    if version >= JavaMinecraftVersion::V_1_17 {
        let id = crate::legacy::ids::clientbound::play::CLEAR_TITLES.to_id(version);
        return Some((id, payload.to_vec()));
    }
    let reset = payload.get_bool().ok()?;
    let action = if reset {
        legacy_title_action(version, 4, 5)
    } else {
        legacy_title_action(version, 3, 4)
    };
    let id = legacy_title_id(version)?;
    let mut out = Vec::new();
    out.write_var_int(&VarInt(action)).ok()?;
    Some((id, out))
}

/// `BOSS_EVENT`: before 1.20.3 the title is a JSON string, not a 26.3 NBT component.
pub fn boss_bar_from_current(mut payload: &[u8], version: JavaMinecraftVersion) -> Option<Vec<u8>> {
    if version >= JavaMinecraftVersion::V_1_20_3 {
        return Some(payload.to_vec());
    }
    let original = payload;
    let uuid = payload.get_uuid().ok()?;
    let action = payload.get_var_int().ok()?;
    if action.0 != 0 && action.0 != 3 {
        // No title field in this action, so the layout is already identical.
        return Some(original.to_vec());
    }
    let component = payload.get_component(&CURRENT_MC_VERSION).ok()?;
    let json = crate::legacy::text::to_json(&component, &version);
    let mut out = Vec::new();
    out.write_uuid(&uuid).ok()?;
    out.write_var_int(&action).ok()?;
    out.write_string(&json).ok()?;
    out.extend_from_slice(payload);
    Some(out)
}

/// `TAB_LIST`: header and footer `TextComponent`s.
pub fn tab_list_from_current(mut payload: &[u8], version: JavaMinecraftVersion) -> Option<Vec<u8>> {
    let header = payload.get_component(&CURRENT_MC_VERSION).ok()?;
    let footer = payload.get_component(&CURRENT_MC_VERSION).ok()?;
    let mut out = Vec::new();
    out.write_component_legacy(&header, &version).ok()?;
    out.write_component_legacy(&footer, &version).ok()?;
    Some(out)
}

/// `SYSTEM_CHAT` (1.19+) / `CHAT` (1.7 - 1.18.2).
pub fn system_chat_from_current(
    mut payload: &[u8],
    version: JavaMinecraftVersion,
) -> Option<(i32, Vec<u8>)> {
    let content = payload.get_component(&CURRENT_MC_VERSION).ok()?;
    let overlay = payload.get_bool().ok()?;

    if version <= JavaMinecraftVersion::V_1_7_6 && overlay {
        return None;
    }

    let packet_id = if version >= JavaMinecraftVersion::V_1_19 {
        ids::clientbound::play::SYSTEM_CHAT.to_id(version)
    } else {
        ids::clientbound::play::CHAT.to_id(version)
    };
    if packet_id == -1 {
        return None;
    }

    let mut out = Vec::new();
    out.write_component_legacy(&content, &version).ok()?;
    if version >= JavaMinecraftVersion::V_1_19_1 {
        out.write_bool(overlay).ok()?;
    } else if version >= JavaMinecraftVersion::V_1_19 {
        out.write_var_int(&VarInt(if overlay { 2 } else { 1 }))
            .ok()?;
    } else if version >= JavaMinecraftVersion::V_1_16 {
        out.write_u8(if overlay { 2 } else { 1 }).ok()?;
        out.write_uuid(&uuid::Uuid::nil()).ok()?;
    } else if version >= JavaMinecraftVersion::V_1_8 {
        out.write_u8(if overlay { 2 } else { 1 }).ok()?;
    }
    Some((packet_id, out))
}

/// `SERVER_DATA` (1.19+).
pub fn server_data_from_current(
    mut payload: &[u8],
    version: JavaMinecraftVersion,
) -> Option<Vec<u8>> {
    let motd = payload.get_component(&CURRENT_MC_VERSION).ok()?;
    let _has_icon = payload.get_bool().ok()?;
    let mut out = Vec::new();
    if version >= JavaMinecraftVersion::V_1_20_5 {
        out.write_component_legacy(&motd, &version).ok()?;
        out.write_bool(false).ok()?;
    } else if version >= JavaMinecraftVersion::V_1_19_4 {
        out.write_component_legacy(&motd, &version).ok()?;
        out.write_bool(false).ok()?;
        out.write_bool(false).ok()?;
    } else {
        out.write_bool(true).ok()?;
        out.write_component_legacy(&motd, &version).ok()?;
        out.write_bool(false).ok()?;
        if version < JavaMinecraftVersion::V_1_19_3 {
            out.write_bool(false).ok()?;
        }
        if version >= JavaMinecraftVersion::V_1_19_1 {
            out.write_bool(false).ok()?;
        }
    }
    Some(out)
}

pub fn change_difficulty_from_current(
    payload: &[u8],
    version: JavaMinecraftVersion,
) -> Option<Vec<u8>> {
    reencode::<CChangeDifficulty>(payload, version)
}

pub fn keep_alive_from_current(payload: &[u8], version: JavaMinecraftVersion) -> Option<Vec<u8>> {
    reencode::<CKeepAlive>(payload, version)
}

pub fn set_held_slot_from_current(
    mut payload: &[u8],
    version: JavaMinecraftVersion,
) -> Option<Vec<u8>> {
    let slot = payload.get_var_int().ok()?.0 as i8;
    let mut out = Vec::new();
    CSetSelectedSlot::new(slot)
        .write_packet_data(&mut out, &version)
        .ok()?;
    Some(out)
}

pub fn forget_level_chunk_from_current(
    payload: &[u8],
    version: JavaMinecraftVersion,
) -> Option<Vec<u8>> {
    reencode::<CUnloadChunk>(payload, version)
}

pub fn set_health_from_current(payload: &[u8], version: JavaMinecraftVersion) -> Option<Vec<u8>> {
    reencode::<CSetHealth>(payload, version)
}

pub fn set_experience_from_current(
    payload: &[u8],
    version: JavaMinecraftVersion,
) -> Option<Vec<u8>> {
    reencode::<CSetExperience>(payload, version)
}

pub fn rotate_head_from_current(payload: &[u8], version: JavaMinecraftVersion) -> Option<Vec<u8>> {
    reencode::<CHeadRot>(payload, version)
}

pub fn take_item_entity_from_current(
    payload: &[u8],
    version: JavaMinecraftVersion,
) -> Option<Vec<u8>> {
    reencode::<CTakeItemEntity>(payload, version)
}

pub fn set_entity_link_from_current(
    payload: &[u8],
    version: JavaMinecraftVersion,
) -> Option<Vec<u8>> {
    reencode::<CSetEntityLink>(payload, version)
}

pub fn display_objective_from_current(
    payload: &[u8],
    version: JavaMinecraftVersion,
) -> Option<Vec<u8>> {
    reencode::<CDisplayObjective>(payload, version)
}

pub fn open_sign_editor_from_current(
    payload: &[u8],
    version: JavaMinecraftVersion,
) -> Option<Vec<u8>> {
    reencode::<COpenSignEditor>(payload, version)
}

pub fn set_objective_from_current(
    mut payload: &[u8],
    version: JavaMinecraftVersion,
) -> Option<Vec<u8>> {
    let name = payload.get_str_borrowed().ok()?;
    let mode = payload.get_u8().ok()?;
    let mut out = Vec::new();
    let max_name = if version >= JavaMinecraftVersion::V_1_18 {
        32767
    } else {
        16
    };
    out.write_string(truncate_chars(name, max_name)).ok()?;
    if version <= JavaMinecraftVersion::V_1_7_6 {
        // 1.7 writes value before mode byte!
        let display = if mode == 0 || mode == 2 {
            payload
                .get_component(&CURRENT_MC_VERSION)
                .ok()?
                .to_pretty_console()
        } else {
            String::new()
        };
        out.write_string(truncate_chars(&display, 32)).ok()?;
        out.write_u8(mode).ok()?;
        return Some(out);
    }
    out.write_u8(mode).ok()?;
    if mode == 0 || mode == 2 {
        let display = payload.get_component(&CURRENT_MC_VERSION).ok()?;
        let render_type = payload.get_var_int().ok()?;
        if version < JavaMinecraftVersion::V_1_13 {
            let text = display.to_pretty_console();
            out.write_string(truncate_chars(&text, 32)).ok()?;
            let render_str = if render_type.0 == 1 {
                "hearts"
            } else {
                "integer"
            };
            out.write_string(render_str).ok()?;
        } else {
            out.write_component_legacy(&display, &version).ok()?;
            out.write_var_int(&render_type).ok()?;
            if version >= JavaMinecraftVersion::V_1_20_3 {
                out.write_bool(false).ok()?;
            }
        }
    }
    Some(out)
}

pub fn set_score_from_current(
    mut payload: &[u8],
    version: JavaMinecraftVersion,
) -> Option<Vec<u8>> {
    let entity_name = payload.get_str_borrowed().ok()?;
    let objective_name = payload.get_str_borrowed().ok()?;
    let value = payload.get_var_int().ok()?;
    let display_name = payload
        .get_option(|r| r.get_component(&CURRENT_MC_VERSION))
        .ok()?;
    let mut out = Vec::new();
    if version >= JavaMinecraftVersion::V_1_20_3 {
        out.write_string(entity_name).ok()?;
        out.write_string(objective_name).ok()?;
        out.write_var_int(&value).ok()?;
        out.write_option(&display_name, |w, comp| {
            w.write_component_legacy(comp, &version)
        })
        .ok()?;
        out.write_bool(false).ok()?;
    } else if version <= JavaMinecraftVersion::V_1_7_6 {
        out.write_string(truncate_chars(entity_name, 16)).ok()?;
        out.write_u8(0).ok()?;
        out.write_string(truncate_chars(objective_name, 16)).ok()?;
        out.write_i32_be(value.0).ok()?;
    } else {
        let max_ent = if version >= JavaMinecraftVersion::V_1_18 {
            32767
        } else {
            40
        };
        let max_obj = if version >= JavaMinecraftVersion::V_1_18 {
            32767
        } else {
            16
        };
        out.write_string(truncate_chars(entity_name, max_ent))
            .ok()?;
        out.write_u8(0).ok()?;
        out.write_string(truncate_chars(objective_name, max_obj))
            .ok()?;
        out.write_var_int(&value).ok()?;
    }
    Some(out)
}

pub fn update_mob_effect_from_current(
    mut payload: &[u8],
    version: JavaMinecraftVersion,
) -> Option<Vec<u8>> {
    let entity_id = payload.get_var_int().ok()?;
    let effect_id = payload.get_var_int().ok()?;
    let amplifier = payload.get_var_int().ok()?;
    let duration = payload.get_var_int().ok()?;
    let flags = payload.get_i8().ok()?;
    let mut out = Vec::new();
    CUpdateMobEffect::new(entity_id, effect_id, amplifier, duration, flags)
        .write_packet_data(&mut out, &version)
        .ok()?;
    Some(out)
}

pub fn remove_mob_effect_from_current(
    mut payload: &[u8],
    version: JavaMinecraftVersion,
) -> Option<Vec<u8>> {
    let entity_id = payload.get_var_int().ok()?;
    let effect_id = payload.get_var_int().ok()?;
    let mut out = Vec::new();
    CRemoveMobEffect::new(entity_id, effect_id)
        .write_packet_data(&mut out, &version)
        .ok()?;
    Some(out)
}

pub fn block_event_from_current(
    mut payload: &[u8],
    version: JavaMinecraftVersion,
) -> Option<Vec<u8>> {
    let location = payload.get_block_pos(&CURRENT_MC_VERSION).ok()?;
    if version < JavaMinecraftVersion::V_1_18 && !(0..256).contains(&location.0.y) {
        return None;
    }
    let action_id = payload.get_u8().ok()?;
    let action_parameter = payload.get_u8().ok()?;
    let block_type = payload.get_var_int().ok()?;
    let mut out = Vec::new();
    if version <= JavaMinecraftVersion::V_1_7_6 {
        out.write_i32_be(location.0.x).ok()?;
        out.write_i16_be(location.0.y as i16).ok()?;
        out.write_i32_be(location.0.z).ok()?;
    } else {
        out.write_block_pos_legacy(&location, &version).ok()?;
    }
    out.write_u8(action_id).ok()?;
    out.write_u8(action_parameter).ok()?;
    out.write_var_int(&block_type).ok()?;
    Some(out)
}

pub fn block_destruction_from_current(
    mut payload: &[u8],
    version: JavaMinecraftVersion,
) -> Option<Vec<u8>> {
    let entity_id = payload.get_var_int().ok()?;
    let location = payload.get_block_pos(&CURRENT_MC_VERSION).ok()?;
    let stage = payload.get_i8().ok()?;
    let mut out = Vec::new();
    out.write_var_int(&entity_id).ok()?;
    if version <= JavaMinecraftVersion::V_1_7_6 {
        out.write_i32_be(location.0.x).ok()?;
        out.write_i32_be(location.0.y).ok()?;
        out.write_i32_be(location.0.z).ok()?;
    } else {
        out.write_block_pos_legacy(&location, &version).ok()?;
    }
    out.write_i8(stage).ok()?;
    Some(out)
}

pub fn level_event_from_current(
    mut payload: &[u8],
    version: JavaMinecraftVersion,
) -> Option<Vec<u8>> {
    let event = payload.get_i32_be().ok()?;
    let location = payload.get_block_pos(&CURRENT_MC_VERSION).ok()?;
    let mut data = payload.get_i32_be().ok()?;
    let disable_relative_volume = payload.get_bool().ok()?;
    if event == 2001 && data >= 0 {
        let remapped = super::block::remap_state(data as u32, version);
        data = if version <= JavaMinecraftVersion::V_1_7_6 {
            ((remapped >> 4) | ((remapped & 0xF) << 12)) as i32
        } else {
            remapped as i32
        };
    }
    let mut out = Vec::new();
    out.write_i32_be(event).ok()?;
    if version <= JavaMinecraftVersion::V_1_7_6 {
        if !(0..256).contains(&location.0.y) {
            return None;
        }
        out.write_i32_be(location.0.x).ok()?;
        out.write_u8(location.0.y as u8).ok()?;
        out.write_i32_be(location.0.z).ok()?;
    } else {
        out.write_block_pos_legacy(&location, &version).ok()?;
    }
    out.write_i32_be(data).ok()?;
    out.write_bool(disable_relative_volume).ok()?;
    Some(out)
}

#[cfg(test)]
mod tests {
    use pumpkin_util::text::TextComponent;

    use super::*;

    fn current_component(component: &TextComponent) -> Vec<u8> {
        let mut payload = Vec::new();
        let _ = payload.write_component(component, &CURRENT_MC_VERSION);
        payload
    }

    #[test]
    fn legacy_title_text_uses_json_and_action_zero() -> Result<(), String> {
        let text = TextComponent::text("Hello");
        let payload = current_component(&text);
        for version in [JavaMinecraftVersion::V_1_10, JavaMinecraftVersion::V_1_16_2] {
            let id = crate::legacy::ids::clientbound::play::TITLE.to_id(version);
            let (got_id, out) =
                set_title_text_from_current(&payload, version).ok_or("legacy title missing")?;
            if got_id != id {
                return Err(format!("wrong title id for {version:?}"));
            }
            let mut expected = Vec::new();
            expected
                .write_var_int(&VarInt(0))
                .map_err(|e| e.to_string())?;
            expected
                .write_string(&crate::legacy::text::to_json(&text, &version))
                .map_err(|e| e.to_string())?;
            if out != expected {
                return Err(format!("title bytes differ for {version:?}"));
            }
        }
        Ok(())
    }

    #[test]
    fn legacy_title_times_action_shifts_at_1_11() -> Result<(), String> {
        let mut payload = Vec::new();
        payload.write_i32_be(10).map_err(|e| e.to_string())?;
        payload.write_i32_be(70).map_err(|e| e.to_string())?;
        payload.write_i32_be(20).map_err(|e| e.to_string())?;
        for (version, action) in [
            (JavaMinecraftVersion::V_1_10, 2),
            (JavaMinecraftVersion::V_1_16_2, 3),
        ] {
            let (_, out) = set_titles_animation_from_current(&payload, version)
                .ok_or("legacy title times missing")?;
            let mut expected = Vec::new();
            expected
                .write_var_int(&VarInt(action))
                .map_err(|e| e.to_string())?;
            expected.extend_from_slice(&payload);
            if out != expected {
                return Err(format!("times bytes differ for {version:?}"));
            }
        }
        Ok(())
    }

    #[test]
    fn boss_bar_title_becomes_json_string() -> Result<(), String> {
        let text = TextComponent::text("Boss");
        let uuid = uuid::Uuid::nil();
        let mut payload = Vec::new();
        payload.write_uuid(&uuid).map_err(|e| e.to_string())?;
        payload
            .write_var_int(&VarInt(0))
            .map_err(|e| e.to_string())?;
        payload.extend_from_slice(&current_component(&text));
        payload.write_f32_be(0.5).map_err(|e| e.to_string())?;
        payload
            .write_var_int(&VarInt(0))
            .map_err(|e| e.to_string())?;
        payload
            .write_var_int(&VarInt(0))
            .map_err(|e| e.to_string())?;
        payload.write_u8(0).map_err(|e| e.to_string())?;

        let version = JavaMinecraftVersion::V_1_12_2;
        let out = boss_bar_from_current(&payload, version).ok_or("boss bar missing")?;
        let mut expected = Vec::new();
        expected.write_uuid(&uuid).map_err(|e| e.to_string())?;
        expected
            .write_var_int(&VarInt(0))
            .map_err(|e| e.to_string())?;
        expected
            .write_string(&crate::legacy::text::to_json(&text, &version))
            .map_err(|e| e.to_string())?;
        expected.write_f32_be(0.5).map_err(|e| e.to_string())?;
        expected
            .write_var_int(&VarInt(0))
            .map_err(|e| e.to_string())?;
        expected
            .write_var_int(&VarInt(0))
            .map_err(|e| e.to_string())?;
        expected.write_u8(0).map_err(|e| e.to_string())?;
        if out != expected {
            return Err("boss bar bytes differ".into());
        }
        Ok(())
    }
}
