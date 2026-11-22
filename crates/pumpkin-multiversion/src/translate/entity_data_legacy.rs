//! Entity data for 1.9 - 1.20.6, whose field indices and serializer ids predate the
//! generated tables in `entity_data.rs`. Only the shared high-value fields are
//! translated; unmappable entries are dropped one by one, not the whole packet.

use pumpkin_data::{entity::EntityType, meta_data_type::MetaDataType, packet::CURRENT_MC_VERSION};
use pumpkin_protocol::{
    VarInt,
    codec::item_stack_seralizer::ItemStackSerializer,
    ser::{NetworkReadExt, NetworkReadSliceExt, NetworkWriteExt},
};
use pumpkin_util::version::JavaMinecraftVersion;

use crate::{
    legacy::LegacyWriteExt,
    translate::{nbt::split_network_nbt, particle::skip_particle},
};

use super::entity_data::entity_type;

eras! {
    enum LegacyDataFormat {
        /// 1.9 - 1.12: old type ids, no pose/ticks, custom name is a plain chat string.
        V1_9 = V_1_9,
        /// 1.13: optional chat, reordered type ids.
        V1_13 = V_1_13,
        /// 1.14 - 1.19.2: pose and ticks frozen add two shared fields.
        V1_14 = V_1_14,
        /// 1.19.3: a var long inserts into the type ids.
        V1_19_3 = V_1_19_3,
        /// 1.19.4 - 1.20.6: current shared indices; components become NBT in 1.20.3.
        V1_19_4 = V_1_19_4,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EntityKind {
    Other,
    Living,
    Player,
}

/// 26.3 indices of the shared fields this converter understands.
mod current {
    pub const FLAGS: u8 = 0;
    pub const AIR: u8 = 1;
    pub const CUSTOM_NAME: u8 = 2;
    pub const NAME_VISIBLE: u8 = 3;
    pub const SILENT: u8 = 4;
    pub const NO_GRAVITY: u8 = 5;
    pub const POSE: u8 = 6;
    pub const TICKS_FROZEN: u8 = 7;
    pub const LIVING_FLAGS: u8 = 8;
    pub const HEALTH: u8 = 9;
    pub const EFFECT_PARTICLES: u8 = 10;
    pub const EFFECT_AMBIENCE: u8 = 11;
    pub const ARROW_COUNT: u8 = 12;
    pub const MAIN_HAND: u8 = 15;
    pub const SKIN_PARTS: u8 = 16;
    pub const ABSORPTION: u8 = 17;
    pub const SCORE: u8 = 18;
}

const fn bool_type(era: LegacyDataFormat) -> i32 {
    match era {
        LegacyDataFormat::V1_9 => 6,
        LegacyDataFormat::V1_13 | LegacyDataFormat::V1_14 => 7,
        LegacyDataFormat::V1_19_3 | LegacyDataFormat::V1_19_4 => 8,
    }
}

const fn float_type(era: LegacyDataFormat) -> i32 {
    match era {
        LegacyDataFormat::V1_9 | LegacyDataFormat::V1_13 | LegacyDataFormat::V1_14 => 2,
        LegacyDataFormat::V1_19_3 | LegacyDataFormat::V1_19_4 => 3,
    }
}

const fn custom_name_type(era: LegacyDataFormat) -> i32 {
    match era {
        // Chat: a plain JSON string, absent as an empty string.
        LegacyDataFormat::V1_9 => 4,
        // Optional chat: a boolean then a JSON string.
        LegacyDataFormat::V1_13 | LegacyDataFormat::V1_14 => 5,
        LegacyDataFormat::V1_19_3 | LegacyDataFormat::V1_19_4 => 6,
    }
}

fn pose_type(era: LegacyDataFormat, version: JavaMinecraftVersion) -> i32 {
    match era {
        LegacyDataFormat::V1_14 => 18,
        LegacyDataFormat::V1_19_3 => 19,
        LegacyDataFormat::V1_19_4 if version >= JavaMinecraftVersion::V_1_20_5 => 21,
        LegacyDataFormat::V1_19_4 => 20,
        LegacyDataFormat::V1_9 | LegacyDataFormat::V1_13 => -1,
    }
}

fn classify(entity_id: i32) -> EntityKind {
    let Some(entity) = entity_type(entity_id).and_then(EntityType::from_raw) else {
        return EntityKind::Other;
    };
    if entity.resource_name == "player" {
        EntityKind::Player
    } else if entity.mob || matches!(entity.resource_name, "armor_stand" | "mannequin") {
        EntityKind::Living
    } else {
        EntityKind::Other
    }
}

fn skip_bytes(read: &mut &[u8], len: usize) -> Option<()> {
    *read = read.get(len..)?;
    Some(())
}

fn copy_byte(read: &mut &[u8]) -> Option<Vec<u8>> {
    Some(vec![read.get_u8().ok()?])
}

fn copy_bool(read: &mut &[u8]) -> Option<Vec<u8>> {
    Some(vec![u8::from(read.get_bool().ok()?)])
}

fn copy_var_int(read: &mut &[u8]) -> Option<Vec<u8>> {
    let value = read.get_var_int().ok()?;
    let mut out = Vec::new();
    out.write_var_int(&value).ok()?;
    Some(out)
}

fn copy_float(read: &mut &[u8]) -> Option<Vec<u8>> {
    Some(read.get_f32_be().ok()?.to_be_bytes().to_vec())
}

/// 26.3's humanoid arm enum is a var int, the legacy main hand is a byte.
fn copy_main_hand(read: &mut &[u8]) -> Option<Vec<u8>> {
    Some(vec![u8::try_from(read.get_var_int().ok()?.0).ok()?])
}

fn copy_custom_name(
    read: &mut &[u8],
    version: JavaMinecraftVersion,
    era: LegacyDataFormat,
) -> Option<Vec<u8>> {
    let present = read.get_bool().ok()?;
    let mut out = Vec::new();
    if era != LegacyDataFormat::V1_9 {
        out.write_bool(present).ok()?;
    }
    if present {
        let component = read.get_component_borrowed(&CURRENT_MC_VERSION).ok()?;
        out.write_component_legacy(&component, &version).ok()?;
    } else if era == LegacyDataFormat::V1_9 {
        out.write_string("").ok()?;
    }
    Some(out)
}

/// The 26.3 effect particle list supersedes the legacy potion colour var int. An
/// empty list maps to "no colour"; a non-empty one has no colour equivalent.
fn copy_effect_color(read: &mut &[u8]) -> Option<Vec<u8>> {
    if read.get_var_int().ok()?.0 != 0 {
        return None;
    }
    let mut out = Vec::new();
    out.write_var_int(&VarInt(0)).ok()?;
    Some(out)
}

fn skip_optional(read: &mut &[u8], value: impl FnOnce(&mut &[u8]) -> Option<()>) -> Option<()> {
    if read.get_bool().ok()? {
        value(read)
    } else {
        Some(())
    }
}

/// Consumes one 26.3 value so a later, mappable entry can still be translated.
fn skip_value(serializer: i32, read: &mut &[u8]) -> Option<()> {
    use MetaDataType as M;
    match serializer {
        id if id == M::BYTE.id || id == M::BOOLEAN.id => skip_bytes(read, 1),
        id if id == M::FLOAT.id => skip_bytes(read, 4),
        id if id == M::BLOCK_POS.id => skip_bytes(read, 8),
        id if id == M::ROTATIONS.id || id == M::VECTOR3.id => skip_bytes(read, 12),
        id if id == M::QUATERNION.id => skip_bytes(read, 16),
        id if id == M::LONG.id => {
            read.get_var_long().ok()?;
            Some(())
        }
        id if id == M::STRING.id => {
            let len = read.get_var_int().ok()?.0;
            skip_bytes(read, usize::try_from(len).ok()?)
        }
        id if id == M::COMPONENT.id => {
            split_network_nbt(read)?;
            Some(())
        }
        id if id == M::OPTIONAL_COMPONENT.id => skip_optional(read, |r| {
            split_network_nbt(r)?;
            Some(())
        }),
        id if id == M::OPTIONAL_BLOCK_POS.id => skip_optional(read, |r| skip_bytes(r, 8)),
        id if id == M::OPTIONAL_LIVING_ENTITY_REFERENCE.id => {
            skip_optional(read, |r| skip_bytes(r, 16))
        }
        id if id == M::OPTIONAL_GLOBAL_POS.id => skip_optional(read, |r| {
            let len = r.get_var_int().ok()?.0;
            skip_bytes(r, usize::try_from(len).ok()?)?;
            skip_bytes(r, 8)
        }),
        id if id == M::ITEM_STACK.id => {
            ItemStackSerializer::read(read).ok()?;
            Some(())
        }
        id if id == M::PARTICLE.id => skip_particle(read),
        id if id == M::PARTICLES.id => {
            let count = read.get_var_int().ok()?.0;
            for _ in 0..count {
                skip_particle(read)?;
            }
            Some(())
        }
        id if id == M::VILLAGER_DATA.id => {
            for _ in 0..3 {
                read.get_var_int().ok()?;
            }
            Some(())
        }
        // All built-in enum, registry and optional int serializers are one var int.
        id if id == M::INT.id
            || id == M::DIRECTION.id
            || id == M::OPTIONAL_UNSIGNED_INT.id
            || id == M::POSE.id
            || id == M::SNIFFER_STATE.id
            || id == M::ARMADILLO_STATE.id
            || id == M::COPPER_GOLEM_STATE.id
            || id == M::WEATHERING_COPPER_STATE.id
            || id == M::HUMANOID_ARM.id
            || id == M::DYE_COLOR.id
            || id == M::PAINTING_VARIANT.id
            || id == M::BLOCK_STATE.id
            || id == M::OPTIONAL_BLOCK_STATE.id
            || (21..=32).contains(&id) =>
        {
            read.get_var_int().ok()?;
            Some(())
        }
        // Resolvable profiles and inline painting variants have no safe skip here.
        _ => None,
    }
}

/// Converts one entry into `(client index, client type id, value bytes)`. The value
/// is read from a scratch cursor, so a failure leaves the caller's cursor untouched.
fn convert_field(
    index: u8,
    serializer: i32,
    kind: EntityKind,
    era: LegacyDataFormat,
    version: JavaMinecraftVersion,
    read: &mut &[u8],
) -> Option<(u8, i32, Vec<u8>)> {
    use self::current as C;
    use MetaDataType as M;

    let pose = era >= LegacyDataFormat::V1_14;
    let living = kind != EntityKind::Other;
    let player = kind == EntityKind::Player;
    let boolean = bool_type(era);
    let float = float_type(era);

    let field = match (index, serializer) {
        (C::FLAGS, id) if id == M::BYTE.id => (0, 0, copy_byte(read)?),
        (C::AIR, id) if id == M::INT.id => (1, 1, copy_var_int(read)?),
        (C::CUSTOM_NAME, id) if id == M::OPTIONAL_COMPONENT.id => (
            2,
            custom_name_type(era),
            copy_custom_name(read, version, era)?,
        ),
        (C::NAME_VISIBLE, id) if id == M::BOOLEAN.id => (3, boolean, copy_bool(read)?),
        (C::SILENT, id) if id == M::BOOLEAN.id => (4, boolean, copy_bool(read)?),
        (C::NO_GRAVITY, id) if id == M::BOOLEAN.id => (5, boolean, copy_bool(read)?),
        (C::POSE, id) if pose && id == M::POSE.id => {
            (6, pose_type(era, version), copy_var_int(read)?)
        }
        (C::TICKS_FROZEN, id) if pose && id == M::INT.id => (7, 1, copy_var_int(read)?),
        (C::LIVING_FLAGS, id) if living && id == M::BYTE.id => {
            (if pose { 8 } else { 6 }, 0, copy_byte(read)?)
        }
        (C::HEALTH, id) if living && id == M::FLOAT.id => {
            (if pose { 9 } else { 7 }, float, copy_float(read)?)
        }
        (C::EFFECT_PARTICLES, id) if living && id == M::PARTICLES.id => {
            (if pose { 10 } else { 8 }, 1, copy_effect_color(read)?)
        }
        (C::EFFECT_AMBIENCE, id) if living && id == M::BOOLEAN.id => {
            (if pose { 11 } else { 9 }, boolean, copy_bool(read)?)
        }
        (C::ARROW_COUNT, id) if living && id == M::INT.id => {
            (if pose { 12 } else { 10 }, 1, copy_var_int(read)?)
        }
        (C::MAIN_HAND, id) if player && id == M::HUMANOID_ARM.id => {
            (if pose { 18 } else { 14 }, 0, copy_main_hand(read)?)
        }
        (C::SKIN_PARTS, id) if player && id == M::BYTE.id => {
            (if pose { 17 } else { 13 }, 0, copy_byte(read)?)
        }
        (C::ABSORPTION, id) if player && id == M::FLOAT.id => {
            (if pose { 15 } else { 11 }, float, copy_float(read)?)
        }
        (C::SCORE, id) if player && id == M::INT.id => {
            (if pose { 16 } else { 12 }, 1, copy_var_int(read)?)
        }
        _ => return None,
    };
    Some(field)
}

/// SET_ENTITY_DATA for 1.9 - 1.20.6: maps the shared fields, dropping the rest.
pub fn set_entity_data_from_current_legacy(
    mut payload: &[u8],
    version: JavaMinecraftVersion,
) -> Option<Vec<u8>> {
    let entity_id = payload.get_var_int().ok()?;
    let kind = classify(entity_id.0);
    let era = LegacyDataFormat::of(version);

    let mut out = Vec::with_capacity(payload.len() + 5);
    out.write_var_int(&entity_id).ok()?;
    loop {
        let index = payload.get_u8().ok()?;
        if index == u8::MAX {
            break;
        }
        let serializer = payload.get_var_int().ok()?.0;
        let mut value_read = payload;
        if let Some((client_index, type_id, value)) =
            convert_field(index, serializer, kind, era, version, &mut value_read)
        {
            out.push(client_index);
            out.write_var_int(&VarInt(type_id)).ok()?;
            out.extend_from_slice(&value);
            payload = value_read;
        } else if skip_value(serializer, &mut payload).is_none() {
            // An unknown serializer leaves the cursor unreadable: stop the list.
            break;
        }
    }
    out.push(u8::MAX);
    Some(out)
}

#[cfg(test)]
mod tests {
    use pumpkin_data::entity::EntityType;
    use pumpkin_nbt::serializer::NbtWriteHelperJava;
    use pumpkin_util::text::TextComponent;

    use super::*;
    use crate::translate::entity_data::{for_viewer, join_viewer, track_spawn};

    fn entry(index: u8, serializer: MetaDataType, value: &[u8], out: &mut Vec<u8>) {
        out.push(index);
        out.write_var_int(&VarInt(serializer.id)).unwrap();
        out.extend_from_slice(value);
    }

    fn var_int(value: i32) -> Vec<u8> {
        let mut out = Vec::new();
        out.write_var_int(&VarInt(value)).unwrap();
        out
    }

    fn component_nbt(text: &str) -> Vec<u8> {
        let tag = TextComponent::text(text.to_owned()).to_nbt_tag_for_version(&CURRENT_MC_VERSION);
        let mut bytes = Vec::new();
        let _ = tag.serialize(&mut NbtWriteHelperJava::new(&mut bytes));
        bytes
    }

    #[test]
    fn player_fields_use_legacy_indices_and_main_hand_is_a_byte() {
        join_viewer(20);
        for_viewer(20, || track_spawn(-1, EntityType::PLAYER.id));
        let mut current = Vec::new();
        current.write_var_int(&VarInt(-1)).unwrap();
        let float = 5.0f32.to_be_bytes();
        entry(
            current::ABSORPTION,
            MetaDataType::FLOAT,
            &float,
            &mut current,
        );
        entry(current::SCORE, MetaDataType::INT, &var_int(7), &mut current);
        entry(
            current::SKIN_PARTS,
            MetaDataType::BYTE,
            &[0x3f],
            &mut current,
        );
        entry(
            current::MAIN_HAND,
            MetaDataType::HUMANOID_ARM,
            &var_int(1),
            &mut current,
        );
        current.push(u8::MAX);

        let mut expected = Vec::new();
        expected.write_var_int(&VarInt(-1)).unwrap();
        // 1.20: absorption 15, score 16, skin parts 17, main hand 18 (byte).
        expected.extend_from_slice(&[15, 3]);
        expected.extend_from_slice(&float);
        expected.extend_from_slice(&[16, 1, 7, 17, 0, 0x3f, 18, 0, 1]);
        expected.push(u8::MAX);
        assert_eq!(
            for_viewer(20, || {
                set_entity_data_from_current_legacy(&current, JavaMinecraftVersion::V_1_20_2)
            }),
            Some(expected)
        );
    }

    #[test]
    fn old_versions_shift_living_fields_and_use_old_type_ids() {
        join_viewer(21);
        for_viewer(21, || track_spawn(-2, EntityType::COW.id));
        let mut current = Vec::new();
        current.write_var_int(&VarInt(-2)).unwrap();
        entry(current::FLAGS, MetaDataType::BYTE, &[0x20], &mut current);
        entry(
            current::HEALTH,
            MetaDataType::FLOAT,
            &10.0f32.to_be_bytes(),
            &mut current,
        );
        entry(
            current::ARROW_COUNT,
            MetaDataType::INT,
            &var_int(2),
            &mut current,
        );
        entry(current::POSE, MetaDataType::POSE, &var_int(1), &mut current);
        current.push(u8::MAX);

        let mut expected = Vec::new();
        expected.write_var_int(&VarInt(-2)).unwrap();
        // 1.9: byte type 0, float type 2, int type 1; no pose field; health 7, arrows 10.
        expected.extend_from_slice(&[0, 0, 0x20, 7, 2]);
        expected.extend_from_slice(&10.0f32.to_be_bytes());
        expected.extend_from_slice(&[10, 1, 2, u8::MAX]);
        assert_eq!(
            for_viewer(21, || {
                set_entity_data_from_current_legacy(&current, JavaMinecraftVersion::V_1_9)
            }),
            Some(expected)
        );
    }

    #[test]
    fn unmapped_entries_are_dropped_without_dropping_later_fields() {
        join_viewer(22);
        for_viewer(22, || track_spawn(-3, EntityType::COW.id));
        let mut current = Vec::new();
        current.write_var_int(&VarInt(-3)).unwrap();
        entry(current::FLAGS, MetaDataType::BYTE, &[0x01], &mut current);
        // A string entry (no shared field maps from it) must be skipped whole.
        entry(20, MetaDataType::STRING, &var_int(3), &mut current);
        current.extend_from_slice(b"abc");
        entry(
            current::HEALTH,
            MetaDataType::FLOAT,
            &8.0f32.to_be_bytes(),
            &mut current,
        );
        current.push(u8::MAX);

        let mut expected = Vec::new();
        expected.write_var_int(&VarInt(-3)).unwrap();
        expected.extend_from_slice(&[0, 0, 0x01, 9, 3]);
        expected.extend_from_slice(&8.0f32.to_be_bytes());
        expected.push(u8::MAX);
        assert_eq!(
            for_viewer(22, || {
                set_entity_data_from_current_legacy(&current, JavaMinecraftVersion::V_1_20_2)
            }),
            Some(expected)
        );
    }

    #[test]
    fn custom_name_layout_per_era() {
        join_viewer(23);
        let mut absent = Vec::new();
        absent.write_var_int(&VarInt(-4)).unwrap();
        entry(
            current::CUSTOM_NAME,
            MetaDataType::OPTIONAL_COMPONENT,
            &[0],
            &mut absent,
        );
        absent.push(u8::MAX);
        for (version, type_id) in [
            (JavaMinecraftVersion::V_1_12_2, 4),
            (JavaMinecraftVersion::V_1_13_2, 5),
            (JavaMinecraftVersion::V_1_19_3, 6),
            (JavaMinecraftVersion::V_1_20_3, 6),
        ] {
            let mut expected = Vec::new();
            expected.write_var_int(&VarInt(-4)).unwrap();
            expected.push(2);
            expected.write_var_int(&VarInt(type_id)).unwrap();
            if type_id == 4 {
                expected.write_string("").unwrap();
            } else {
                expected.write_bool(false).unwrap();
            }
            expected.push(u8::MAX);
            assert_eq!(
                for_viewer(23, || {
                    set_entity_data_from_current_legacy(&absent, version)
                }),
                Some(expected),
                "version {version}"
            );
        }

        // A present name reuses NBT bytes in 1.20.3+, but is JSON text earlier.
        let nbt = component_nbt("Bob");
        let mut present = Vec::new();
        present.write_var_int(&VarInt(-4)).unwrap();
        entry(
            current::CUSTOM_NAME,
            MetaDataType::OPTIONAL_COMPONENT,
            &[1],
            &mut present,
        );
        present.extend_from_slice(&nbt);
        present.push(u8::MAX);
        let modern = for_viewer(23, || {
            set_entity_data_from_current_legacy(&present, JavaMinecraftVersion::V_1_20_3)
        })
        .unwrap();
        assert!(modern.windows(nbt.len()).any(|w| w == nbt));
        let old = for_viewer(23, || {
            set_entity_data_from_current_legacy(&present, JavaMinecraftVersion::V_1_20_2)
        })
        .unwrap();
        assert!(!old.windows(nbt.len()).any(|w| w == nbt));
    }
}
