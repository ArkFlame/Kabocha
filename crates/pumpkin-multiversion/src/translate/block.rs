//! Block states and block entity types: BLOCK_UPDATE, SECTION_BLOCKS_UPDATE, BLOCK_ENTITY_DATA,
//! chunk palettes.

use std::{
    io::Write,
    sync::{Mutex, PoisonError},
};

use flate2::{Compression, write::GzEncoder};
use pumpkin_protocol::{
    VarInt,
    codec::var_long::VarLong,
    ser::{NetworkReadExt, NetworkWriteExt},
};
use pumpkin_util::{math::vector3::unpacked_chunk_pos, version::JavaMinecraftVersion};

use super::{
    ceil_log2,
    nbt::{split_network_nbt, write_network_nbt},
};
use crate::legacy::block_pos_to_version;
use crate::remap::{
    block_entity_type_id_remap::remap_block_entity_type_id_for_version,
    block_state_remap::remap_block_state_for_version,
};

#[must_use]
pub fn remap_state(id: u32, version: JavaMinecraftVersion) -> u32 {
    u16::try_from(id).map_or(0, |id| {
        u32::from(remap_block_state_for_version(id, version))
    })
}

#[must_use]
pub fn remap_block_entity_type(id: u32, version: JavaMinecraftVersion) -> u32 {
    remap_block_entity_type_id_for_version(id, version)
}

/// Width of the client's global block state palette, from the highest state 26.3 maps to.
pub fn direct_bits(version: JavaMinecraftVersion) -> u8 {
    static CACHE: Mutex<Vec<(JavaMinecraftVersion, u8)>> = Mutex::new(Vec::new());
    let mut cache = CACHE.lock().unwrap_or_else(PoisonError::into_inner);
    if let Some(&(_, bits)) = cache.iter().find(|(v, _)| *v == version) {
        return bits;
    }
    let max = (0..=u16::MAX)
        .map(|id| remap_block_state_for_version(id, version))
        .max()
        .unwrap_or(0);
    let bits = ceil_log2(u32::from(max) + 1);
    cache.push((version, bits));
    bits
}

/// BLOCK_UPDATE: position, state.
pub fn block_update_from_current(
    mut payload: &[u8],
    version: JavaMinecraftVersion,
) -> Option<Vec<u8>> {
    let raw_pos = payload.get_i64_be().ok()?;
    let pos_bytes = raw_pos.to_be_bytes();
    let mut pos_slice = pos_bytes.as_slice();
    let pos = pos_slice
        .get_block_pos(&pumpkin_data::packet::CURRENT_MC_VERSION)
        .ok()?;
    if version < JavaMinecraftVersion::V_1_18 && !(0..256).contains(&pos.0.y) {
        return None;
    }
    let state = payload.get_var_int().ok()?.0 as u32;
    let remapped = remap_state(state, version);
    let mut out = Vec::new();
    if version <= JavaMinecraftVersion::V_1_7_6 {
        out.write_i32_be(pos.0.x).ok()?;
        out.write_u8(pos.0.y as u8).ok()?;
        out.write_i32_be(pos.0.z).ok()?;
        out.write_var_int(&VarInt((remapped >> 4) as i32)).ok()?;
        out.write_u8((remapped & 0xF) as u8).ok()?;
    } else {
        out.write_i64_be(block_pos_to_version(raw_pos, &version))
            .ok()?;
        out.write_var_int(&VarInt(remapped as i32)).ok()?;
    }
    Some(out)
}

eras! {
    pub enum BlockEntityDataFormat {
        /// Explicit x/y/z, byte action, gzipped NBT.
        V1_7 = V_1_7_2,
        /// Packed position, byte action, named NBT.
        V1_8 = V_1_8,
        /// Packed position, type id, network NBT.
        V1_18 = V_1_18,
    }
}

/// 26.3 network NBT as the gzipped, length-prefixed payload 1.7 clients expect.
fn write_gzipped_nbt(nbt: &[u8], version: JavaMinecraftVersion, out: &mut Vec<u8>) -> Option<()> {
    let (&tag_type, _) = nbt.split_first()?;
    if tag_type == 0 {
        out.write_i16_be(-1).ok()?;
        return Some(());
    }
    let mut named = Vec::new();
    write_network_nbt(nbt, version, &mut named);
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&named).ok()?;
    let compressed = encoder.finish().ok()?;
    out.write_i16_be(i16::try_from(compressed.len()).ok()?)
        .ok()?;
    out.extend_from_slice(&compressed);
    Some(())
}

/// BLOCK_ENTITY_DATA: position, type/action, NBT. Types the client lacks are dropped.
pub fn block_entity_data_from_current(
    mut payload: &[u8],
    version: JavaMinecraftVersion,
) -> Option<Vec<u8>> {
    let position = payload.get_i64_be().ok()?;
    let kind = payload.get_var_int().ok()?.0 as u32;
    let client_kind = remap_block_entity_type(kind, version);
    if client_kind == 0 && kind != 0 {
        return None;
    }
    let nbt = split_network_nbt(&mut payload)?;
    let format = BlockEntityDataFormat::of(version);
    let mut out = Vec::new();
    match format {
        BlockEntityDataFormat::V1_7 => {
            let pos_bytes = position.to_be_bytes();
            let mut pos_slice = pos_bytes.as_slice();
            let pos = pos_slice
                .get_block_pos(&pumpkin_data::packet::CURRENT_MC_VERSION)
                .ok()?;
            out.write_i32_be(pos.0.x).ok()?;
            out.write_i16_be(pos.0.y as i16).ok()?;
            out.write_i32_be(pos.0.z).ok()?;
            out.write_u8(client_kind as u8).ok()?;
            write_gzipped_nbt(nbt, version, &mut out)?;
        }
        BlockEntityDataFormat::V1_8 => {
            out.write_i64_be(block_pos_to_version(position, &version))
                .ok()?;
            out.write_u8(client_kind as u8).ok()?;
            write_network_nbt(nbt, version, &mut out);
        }
        BlockEntityDataFormat::V1_18 => {
            out.write_i64_be(position).ok()?;
            out.write_var_int(&VarInt(client_kind as i32)).ok()?;
            write_network_nbt(nbt, version, &mut out);
        }
    }
    Some(out)
}

eras! {
    pub enum SectionUpdateFormat {
        /// Per chunk: i16 count, i32 byte length, i16 position + i16 state records.
        V1_7 = V_1_7_2,
        /// Per chunk: varint count, i16 position + varint state records.
        V1_8 = V_1_8,
        /// Per section with packed var long entries and a light flag.
        V1_16 = V_1_16,
        /// Light flag removed; same as 26.3.
        V1_20 = V_1_20,
    }
}

/// SECTION_BLOCKS_UPDATE before 1.16: chunk coordinates and unpacked records.
fn section_blocks_update_legacy(
    mut payload: &[u8],
    format: SectionUpdateFormat,
    version: JavaMinecraftVersion,
) -> Option<Vec<u8>> {
    let old = format == SectionUpdateFormat::V1_7;
    let section = payload.get_i64_be().ok()?;
    let chunk = unpacked_chunk_pos(section);
    let count = if old {
        i32::from(payload.get_i16_be().ok()?)
    } else {
        payload.get_var_int().ok()?.0
    };
    if count < 0 {
        return None;
    }
    let count = count as usize;
    let mut out = Vec::new();
    out.write_i32_be(chunk.x).ok()?;
    out.write_i32_be(chunk.z).ok()?;
    if old {
        out.write_i16_be(count as i16).ok()?;
        out.write_i32_be((count * 4) as i32).ok()?;
    } else {
        out.write_var_int(&VarInt(count as i32)).ok()?;
    }
    for _ in 0..count {
        let entry = payload.get_var_long().ok()?.0 as u64;
        let local = entry & 0xFFF;
        let rel_x = (local >> 8) & 0xF;
        let rel_z = (local >> 4) & 0xF;
        let rel_y = local & 0xF;
        let packed_pos = (rel_x << 12) | (rel_z << 8) | rel_y;
        let state = remap_state((entry >> 12) as u32, version);
        out.write_i16_be(packed_pos as i16).ok()?;
        if old {
            out.write_i16_be(state as i16).ok()?;
        } else {
            out.write_var_int(&VarInt(state as i32)).ok()?;
        }
    }
    Some(out)
}

/// SECTION_BLOCKS_UPDATE: `state << 12 | local position` entries.
pub fn section_blocks_update_from_current(
    mut payload: &[u8],
    version: JavaMinecraftVersion,
) -> Option<Vec<u8>> {
    let format = SectionUpdateFormat::of(version);
    if format == SectionUpdateFormat::V1_7 || format == SectionUpdateFormat::V1_8 {
        return section_blocks_update_legacy(payload, format, version);
    }
    let section = payload.get_i64_be().ok()?;
    let count = payload.get_var_int().ok()?;
    let mut out = Vec::new();
    out.write_i64_be(section).ok()?;
    if format == SectionUpdateFormat::V1_16 {
        out.write_bool(false).ok()?;
    }
    out.write_var_int(&count).ok()?;
    for _ in 0..count.0 {
        let entry = payload.get_var_long().ok()?.0 as u64;
        let state = remap_state((entry >> 12) as u32, version);
        let packed = (u64::from(state) << 12) | (entry & 0xFFF);
        out.write_var_long(&VarLong(packed as i64)).ok()?;
    }
    Some(out)
}
