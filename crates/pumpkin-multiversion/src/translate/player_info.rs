//! PLAYER_INFO_UPDATE and PLAYER_INFO_REMOVE (and pre-1.19.3 PLAYER_INFO).

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex, PoisonError};

use crate::legacy::{LegacyWriteExt, ids};
use pumpkin_protocol::{
    Property, VarInt,
    java::client::play::{CPlayerInfoUpdate, InitChat, Player, PlayerAction, PlayerInfoFlags},
    ser::{NetworkReadExt, NetworkReadSliceExt, NetworkWriteExt, ReadingError},
};
use pumpkin_util::{text::TextComponent, version::JavaMinecraftVersion};

eras! {
    pub enum PlayerInfoFormat {
        /// Single-entry string + online + i16 ping packet.
        V1_7 = V_1_7_2,
        /// Action VarInt (0..=4) + UUID entry list.
        V1_8 = V_1_8,
        /// Action bit set, which core's writer masks per version.
        V1_19_3 = V_1_19_3,
    }
}

/// Names from ADD_PLAYER, keyed by UUID, for 1.7's name-based player list.
static NAMES: LazyLock<Mutex<HashMap<uuid::Uuid, Box<str>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// One player's actions, owned so the borrowed `PlayerAction`s can point into it.
struct Entry {
    uuid: uuid::Uuid,
    add: Option<(Box<str>, Vec<Property>)>,
    chat: Option<Option<InitChat>>,
    game_mode: Option<i32>,
    listed: Option<bool>,
    latency: Option<i32>,
    display_name: Option<Option<TextComponent>>,
    list_order: Option<i32>,
    hat: Option<bool>,
}

fn read_entry(read: &mut &[u8], flags: PlayerInfoFlags) -> Option<Entry> {
    let uuid = read.get_uuid().ok()?;
    let add = if flags.contains(PlayerInfoFlags::ADD_PLAYER) {
        let name = read.get_str().ok()?;
        let properties = read
            .get_list(|read| {
                Ok(Property {
                    name: read.get_str()?,
                    value: read.get_str()?,
                    signature: read.get_option(|read| read.get_str())?,
                })
            })
            .ok()?;
        Some((name, properties))
    } else {
        None
    };
    let chat = if flags.contains(PlayerInfoFlags::INITIALIZE_CHAT) {
        Some(
            read.get_option(|read| {
                let session_id = read.get_uuid()?;
                let expires_at = read.get_i64_be()?;
                let key_len = read.get_var_int()?.0 as usize;
                let public_key = read.read_slice_borrowed(key_len)?.into();
                let signature_len = read.get_var_int()?.0 as usize;
                let signature = read.read_slice_borrowed(signature_len)?.into();
                Ok(InitChat {
                    session_id,
                    expires_at,
                    public_key,
                    signature,
                })
            })
            .ok()?,
        )
    } else {
        None
    };
    let game_mode = flags
        .contains(PlayerInfoFlags::UPDATE_GAME_MODE)
        .then(|| read.get_var_int().map(|v| v.0))
        .transpose()
        .ok()?;
    let listed = flags
        .contains(PlayerInfoFlags::UPDATE_LISTED)
        .then(|| read.get_bool())
        .transpose()
        .ok()?;
    let latency = flags
        .contains(PlayerInfoFlags::UPDATE_LATENCY)
        .then(|| read.get_var_int().map(|v| v.0))
        .transpose()
        .ok()?;
    let display_name = flags
        .contains(PlayerInfoFlags::UPDATE_DISPLAY_NAME)
        .then(|| {
            read.get_option(|read| read.get_component(&pumpkin_data::packet::CURRENT_MC_VERSION))
        })
        .transpose()
        .ok()?;
    let list_order = flags
        .contains(PlayerInfoFlags::UPDATE_LIST_PRIORITY)
        .then(|| read.get_var_int().map(|v| v.0))
        .transpose()
        .ok()?;
    let hat = flags
        .contains(PlayerInfoFlags::UPDATE_HAT)
        .then(|| read.get_bool())
        .transpose()
        .ok()?;
    Some(Entry {
        uuid,
        add,
        chat,
        game_mode,
        listed,
        latency,
        display_name,
        list_order,
        hat,
    })
}

/// The actions present in `entry` ->  the writer orders them by flag.
fn actions(entry: &Entry) -> Vec<PlayerAction<'_>> {
    let mut actions = Vec::new();
    if let Some((name, properties)) = &entry.add {
        actions.push(PlayerAction::AddPlayer { name, properties });
    }
    if let Some(chat) = &entry.chat {
        actions.push(PlayerAction::InitializeChat(chat.as_ref().map(|chat| {
            InitChat {
                session_id: chat.session_id,
                expires_at: chat.expires_at,
                public_key: chat.public_key.clone(),
                signature: chat.signature.clone(),
            }
        })));
    }
    if let Some(game_mode) = entry.game_mode {
        actions.push(PlayerAction::UpdateGameMode(game_mode.into()));
    }
    if let Some(listed) = entry.listed {
        actions.push(PlayerAction::UpdateListed(listed));
    }
    if let Some(latency) = entry.latency {
        actions.push(PlayerAction::UpdateLatency(latency.into()));
    }
    if let Some(display_name) = &entry.display_name {
        actions.push(PlayerAction::UpdateDisplayName(display_name.as_ref()));
    }
    if let Some(order) = entry.list_order {
        actions.push(PlayerAction::UpdateListOrder(order.into()));
    }
    if let Some(hat) = entry.hat {
        actions.push(PlayerAction::UpdateHat(hat));
    }
    actions
}

/// Drops the list order before 1.21.2 and the hat before 1.21.4, display names as JSON
/// before 1.20.3, and converts to legacy `PLAYER_INFO` before 1.19.3.
pub fn player_info_update_from_current(
    mut payload: &[u8],
    version: JavaMinecraftVersion,
) -> Option<Vec<u8>> {
    let format = PlayerInfoFormat::of(version);
    let bits = payload.get_u8().ok()?;
    let flags = PlayerInfoFlags::from_bits_retain(bits);
    let entries = payload
        .get_list(|read| {
            read_entry(read, flags).ok_or_else(|| ReadingError::Message("player".into()))
        })
        .ok()?;
    if format == PlayerInfoFormat::V1_7 && flags.contains(PlayerInfoFlags::ADD_PLAYER) {
        remember_names(&entries);
    }

    match format {
        PlayerInfoFormat::V1_19_3 => {
            let actions: Vec<_> = entries.iter().map(actions).collect();
            let players: Vec<_> = entries
                .iter()
                .zip(&actions)
                .map(|(entry, actions)| Player {
                    uuid: entry.uuid,
                    actions,
                })
                .collect();
            super::legacy_bytes(&CPlayerInfoUpdate::new(bits, &players), version)
        }
        PlayerInfoFormat::V1_8 => encode_player_info_v1_8(flags, &entries, version),
        PlayerInfoFormat::V1_7 => encode_player_info_v1_7(flags, &entries),
    }
}

/// Outgoing `PLAYER_INFO_UPDATE` mapped to `(client_id, payload)`.
pub fn player_info_packet_from_current(
    payload: &[u8],
    version: JavaMinecraftVersion,
) -> Option<(i32, Vec<u8>)> {
    let packet_id = if version >= JavaMinecraftVersion::V_1_19_3 {
        ids::clientbound::play::PLAYER_INFO_UPDATE.to_id(version)
    } else {
        ids::clientbound::play::PLAYER_INFO.to_id(version)
    };
    if packet_id == -1 {
        return None;
    }
    Some((
        packet_id,
        player_info_update_from_current(payload, version)?,
    ))
}

/// Outgoing `PLAYER_INFO_REMOVE` mapped to `(client_id, payload)`.
pub fn player_info_remove_packet_from_current(
    payload: &[u8],
    version: JavaMinecraftVersion,
) -> Option<(i32, Vec<u8>)> {
    match PlayerInfoFormat::of(version) {
        PlayerInfoFormat::V1_19_3 => {
            let id = ids::clientbound::play::PLAYER_INFO_REMOVE.to_id(version);
            (id != -1).then(|| (id, payload.to_vec()))
        }
        PlayerInfoFormat::V1_8 => {
            let id = ids::clientbound::play::PLAYER_INFO.to_id(version);
            if id == -1 {
                return None;
            }
            let mut out = Vec::with_capacity(1 + payload.len());
            out.write_var_int(&VarInt(4)).ok()?;
            out.extend_from_slice(payload);
            Some((id, out))
        }
        PlayerInfoFormat::V1_7 => {
            let id = ids::clientbound::play::PLAYER_INFO.to_id(version);
            if id == -1 {
                return None;
            }
            let mut read = payload;
            let players = read.get_list(|read| read.get_uuid()).ok()?;
            let uuid = players.first()?;
            let name = take_name(*uuid)?;
            let mut out = Vec::new();
            out.write_string(&name).ok()?;
            out.write_bool(false).ok()?;
            out.write_i16_be(0).ok()?;
            Some((id, out))
        }
    }
}

/// Remembers the names a viewer saw in ADD_PLAYER so a later remove can name the player.
fn remember_names(entries: &[Entry]) {
    let mut names = NAMES.lock().unwrap_or_else(PoisonError::into_inner);
    for entry in entries {
        if let Some((name, _)) = &entry.add {
            names.insert(entry.uuid, truncate_chars(name, 16).into());
        }
    }
}

/// Returns and forgets a tracked name.
fn take_name(uuid: uuid::Uuid) -> Option<Box<str>> {
    NAMES
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .remove(&uuid)
}

fn truncate_chars(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

fn encode_player_info_v1_8(
    flags: PlayerInfoFlags,
    entries: &[Entry],
    version: JavaMinecraftVersion,
) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    if flags.contains(PlayerInfoFlags::ADD_PLAYER) {
        out.write_var_int(&VarInt(0)).ok()?;
        out.write_var_int(&VarInt(entries.len() as i32)).ok()?;
        for entry in entries {
            let (name, properties) = entry.add.as_ref()?;
            out.write_uuid(&entry.uuid).ok()?;
            out.write_string(truncate_chars(name, 16)).ok()?;
            out.write_list(properties, |w, prop| {
                w.write_string(&prop.name)?;
                w.write_string(&prop.value)?;
                w.write_option(&prop.signature, |w, sig| w.write_string(sig))
            })
            .ok()?;
            out.write_var_int(&VarInt(entry.game_mode.unwrap_or(0)))
                .ok()?;
            out.write_var_int(&VarInt(entry.latency.unwrap_or(0)))
                .ok()?;
            let dn = entry.display_name.as_ref().and_then(Option::as_ref);
            out.write_option(&dn, |w, comp| w.write_component_legacy(comp, &version))
                .ok()?;
            if version >= JavaMinecraftVersion::V_1_19 {
                out.write_bool(false).ok()?;
            }
        }
        Some(out)
    } else if flags.contains(PlayerInfoFlags::UPDATE_GAME_MODE) {
        out.write_var_int(&VarInt(1)).ok()?;
        out.write_var_int(&VarInt(entries.len() as i32)).ok()?;
        for entry in entries {
            out.write_uuid(&entry.uuid).ok()?;
            out.write_var_int(&VarInt(entry.game_mode.unwrap_or(0)))
                .ok()?;
        }
        Some(out)
    } else if flags.contains(PlayerInfoFlags::UPDATE_LATENCY) {
        out.write_var_int(&VarInt(2)).ok()?;
        out.write_var_int(&VarInt(entries.len() as i32)).ok()?;
        for entry in entries {
            out.write_uuid(&entry.uuid).ok()?;
            out.write_var_int(&VarInt(entry.latency.unwrap_or(0)))
                .ok()?;
        }
        Some(out)
    } else if flags.contains(PlayerInfoFlags::UPDATE_DISPLAY_NAME) {
        out.write_var_int(&VarInt(3)).ok()?;
        out.write_var_int(&VarInt(entries.len() as i32)).ok()?;
        for entry in entries {
            out.write_uuid(&entry.uuid).ok()?;
            let dn = entry.display_name.as_ref().and_then(Option::as_ref);
            out.write_option(&dn, |w, comp| w.write_component_legacy(comp, &version))
                .ok()?;
        }
        Some(out)
    } else {
        None
    }
}

fn encode_player_info_v1_7(flags: PlayerInfoFlags, entries: &[Entry]) -> Option<Vec<u8>> {
    if !flags.contains(PlayerInfoFlags::ADD_PLAYER) {
        return None;
    }
    let first = entries.first()?;
    let (name, _) = first.add.as_ref()?;
    let mut out = Vec::new();
    out.write_string(truncate_chars(name, 16)).ok()?;
    out.write_bool(true).ok()?;
    let ping = first
        .latency
        .unwrap_or(0)
        .clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16;
    out.write_i16_be(ping).ok()?;
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::legacy::LegacyWrite;
    use pumpkin_data::packet::CURRENT_MC_VERSION;

    fn packet(version: JavaMinecraftVersion) -> Vec<u8> {
        let properties = [Property {
            name: "textures".into(),
            value: "abc".into(),
            signature: None,
        }];
        let display_name = TextComponent::text("Steve");
        let actions = [
            PlayerAction::AddPlayer {
                name: "Steve",
                properties: &properties,
            },
            PlayerAction::UpdateGameMode(1.into()),
            PlayerAction::UpdateListed(true),
            PlayerAction::UpdateLatency(20.into()),
            PlayerAction::UpdateDisplayName(Some(&display_name)),
            PlayerAction::UpdateListOrder(3.into()),
            PlayerAction::UpdateHat(true),
        ];
        let players = [Player {
            uuid: uuid::Uuid::from_u128(7),
            actions: &actions,
        }];
        let bits = (PlayerInfoFlags::all() - PlayerInfoFlags::INITIALIZE_CHAT).bits();
        let mut out = Vec::new();
        CPlayerInfoUpdate::new(bits, &players)
            .write_legacy(&mut out, &version)
            .unwrap();
        out
    }

    fn current_add(uuid: uuid::Uuid, name: &str) -> Vec<u8> {
        let mut out = Vec::new();
        out.write_u8(PlayerInfoFlags::ADD_PLAYER.bits()).unwrap();
        out.write_var_int(&VarInt(1)).unwrap();
        out.write_uuid(&uuid).unwrap();
        out.write_string(name).unwrap();
        out.write_var_int(&VarInt(0)).unwrap();
        out
    }

    #[test]
    fn remove_1_7_uses_the_tracked_name() {
        let version = JavaMinecraftVersion::V_1_7_2;
        let uuid = uuid::Uuid::from_u128(7);
        let _ = player_info_update_from_current(&current_add(uuid, "Steve"), version);

        let mut remove = Vec::new();
        remove.write_var_int(&VarInt(1)).unwrap();
        remove.write_uuid(&uuid).unwrap();
        let (id, out) = player_info_remove_packet_from_current(&remove, version).unwrap();
        assert_eq!(id, ids::clientbound::play::PLAYER_INFO.to_id(version));

        let mut slice = out.as_slice();
        assert_eq!(&*slice.get_str().unwrap(), "Steve");
        assert!(!slice.get_bool().unwrap());
        assert_eq!(slice.get_i16_be().unwrap(), 0);
        assert!(slice.is_empty());
        // A second remove has no name to write.
        assert_eq!(player_info_remove_packet_from_current(&remove, version), None);
    }

    #[test]
    fn older_clients_get_their_own_layout() {
        let current = packet(CURRENT_MC_VERSION);
        for version in [
            JavaMinecraftVersion::V_1_19_3,
            JavaMinecraftVersion::V_1_20_3,
            JavaMinecraftVersion::V_1_21_2,
            JavaMinecraftVersion::V_1_21_4,
        ] {
            assert_eq!(
                player_info_update_from_current(&current, version),
                Some(packet(version)),
                "{version}"
            );
        }
        assert_ne!(
            packet(JavaMinecraftVersion::V_1_21_2),
            current,
            "hat is dropped"
        );
    }
}
