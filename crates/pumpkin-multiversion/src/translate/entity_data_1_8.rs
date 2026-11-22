//! 1.7/1.8 entity metadata: a type/index byte per entry instead of an index and a serializer.
//!
//! The modern list starts after the entity id with a `u8` index, a VarInt serializer id and
//! the value, terminated by `0xff`; the old list writes `(type << 5) | index` and the value,
//! terminated by `0x7f`. Only common fields are translated and an unmapped field ends the list,
//! as in [`super::entity_data::set_entity_data_from_current`].

use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_protocol::ser::{NetworkReadExt, NetworkReadSliceExt, NetworkWriteExt};
use pumpkin_util::{translation::Locale, version::JavaMinecraftVersion};

use crate::legacy::text;

const END_CURRENT: u8 = u8::MAX;
const END_LEGACY: u8 = 0x7f;

/// Packs an old-format metadata `type` and `index` into one byte.
const fn entry_byte(type_id: u8, index: u8) -> u8 {
    (type_id << 5) | (index & 0x1f)
}

/// Converts a 26.3 entity-data list to the 1.7/1.8 metadata list.
///
/// `payload` starts after the entity id and holds entries of a `u8` index, a VarInt
/// serializer id and the value, terminated by `0xff`. The returned bytes hold the old
/// `(type << 5) | index` byte followed by the value, terminated by `0x7f`.
///
/// The old types are `0` byte, `1` short, `2` int, `3` float, `4` string, `5` slot and `6`
/// three ints; 1.8 also defines `7` for three floats (rotation). Only the fields shared by
/// every entity are mapped: flags (`0` to `0`, byte), air (`1` to `1`, short), custom name
/// (`2` to `2`, string), name visible (`3` to `3`, byte), health (`9` to `6`, float) and
/// player skin parts (`16` to `10`, byte).
pub fn entity_data_to_legacy_1_7_8(
    mut payload: &[u8],
    version: JavaMinecraftVersion,
) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    loop {
        let index = payload.get_u8().ok()?;
        if index == END_CURRENT {
            break;
        }
        let serializer = payload.get_var_int().ok()?.0;
        let Some((legacy_index, legacy_type)) = legacy_field(index, serializer) else {
            break;
        };
        out.push(entry_byte(legacy_type, legacy_index));
        write_value(index, legacy_type, &mut payload, version, &mut out)?;
    }
    out.push(END_LEGACY);
    Some(out)
}

/// The old index and type of a common 26.3 field, or `None` when it has no counterpart.
fn legacy_field(index: u8, serializer: i32) -> Option<(u8, u8)> {
    use MetaDataType as M;
    Some(match (index, serializer) {
        (0, id) if id == M::BYTE.id || id == M::BOOLEAN.id => (0, 0),
        (1, id) if id == M::INT.id => (1, 1),
        (2, id) if id == M::OPTIONAL_COMPONENT.id => (2, 4),
        (3, id) if id == M::BOOLEAN.id => (3, 0),
        (9, id) if id == M::FLOAT.id => (6, 3),
        (16, id) if id == M::BYTE.id => (10, 0),
        _ => return None,
    })
}

/// Writes the value of a mapped field in its old layout.
fn write_value(
    index: u8,
    legacy_type: u8,
    read: &mut &[u8],
    version: JavaMinecraftVersion,
    out: &mut Vec<u8>,
) -> Option<()> {
    match (index, legacy_type) {
        (1, 1) => {
            let air = read.get_var_int().ok()?.0;
            let air = air.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16;
            out.write_i16_be(air).ok()
        }
        (2, 4) => {
            let name = if read.get_bool().ok()? {
                legacy_name(read, version)?
            } else {
                String::new()
            };
            out.write_string(&name).ok()
        }
        (9, 3) => out.write_f32_be(read.get_f32_be().ok()?).ok(),
        _ => out.write_u8(read.get_u8().ok()?).ok(),
    }
}

/// Reads a 26.3 component and renders it as the plain string the old metadata uses.
fn legacy_name(read: &mut &[u8], version: JavaMinecraftVersion) -> Option<String> {
    let component = read
        .get_component_borrowed(&pumpkin_data::packet::CURRENT_MC_VERSION)
        .ok()?;
    Some(text::to_legacy_string(&component, &version, Locale::EnUs))
}

#[cfg(test)]
mod tests {
    use pumpkin_data::meta_data_type::MetaDataType;
    use pumpkin_protocol::VarInt;

    use super::*;

    fn entry(index: u8, serializer: MetaDataType, value: &[u8], out: &mut Vec<u8>) {
        out.push(index);
        out.write_var_int(&VarInt(serializer.id)).unwrap();
        out.extend_from_slice(value);
    }

    #[test]
    fn common_fields_get_old_indices_and_types() {
        let mut current = Vec::new();
        let mut air = Vec::new();
        air.write_var_int(&VarInt(300)).unwrap();
        entry(0, MetaDataType::BYTE, &[0x20], &mut current);
        entry(1, MetaDataType::INT, &air, &mut current);
        entry(3, MetaDataType::BOOLEAN, &[1], &mut current);
        entry(9, MetaDataType::FLOAT, &20.0f32.to_be_bytes(), &mut current);
        entry(16, MetaDataType::BYTE, &[0x07], &mut current);
        current.push(END_CURRENT);

        let mut expected = Vec::new();
        expected.extend_from_slice(&[
            0x00, 0x20, // flags: byte 0, index 0
            0x21, 0x01, 0x2c, // air: short 1, index 1, 300
            0x03, 0x01, // name visible: byte 0, index 3
            0x66, // health: float 3, index 6
        ]);
        expected.extend_from_slice(&20.0f32.to_be_bytes());
        expected.extend_from_slice(&[0x0a, 0x07]); // skin parts: byte 0, index 10
        expected.push(END_LEGACY);

        for version in [JavaMinecraftVersion::V_1_7_2, JavaMinecraftVersion::V_1_8] {
            assert_eq!(
                entity_data_to_legacy_1_7_8(&current, version),
                Some(expected.clone()),
                "{version}"
            );
        }
    }

    #[test]
    fn unmapped_field_ends_the_list() {
        let mut current = Vec::new();
        entry(0, MetaDataType::BYTE, &[1], &mut current);
        entry(4, MetaDataType::BOOLEAN, &[0], &mut current);
        entry(9, MetaDataType::FLOAT, &20.0f32.to_be_bytes(), &mut current);
        current.push(END_CURRENT);
        assert_eq!(
            entity_data_to_legacy_1_7_8(&current, JavaMinecraftVersion::V_1_8),
            Some(vec![0x00, 0x01, END_LEGACY])
        );
    }
}
