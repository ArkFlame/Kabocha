//! Built-in multi-version Java Edition support: translates packets between the current protocol
//! and every older client from 1.7.2 up to the version before [`CURRENT_MC_VERSION`].
//!
//! The server core calls the functions below at the point where a packet is queued
//! (`outgoing_*`) or decoded (`incoming_*`); nothing here owns a connection.

// Ported translation tables and codecs (from pumpkin-java-multiversion): they predate the
// workspace's pedantic lint set. Tightening them is incremental; the glue in `pumpkin-core` stays strict.
#![allow(
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::large_stack_arrays,
    clippy::if_then_some_else_none,
    clippy::semicolon_outside_block,
    clippy::separated_literal_suffix,
    clippy::redundant_test_prefix,
    clippy::wildcard_enum_match_arm,
    clippy::option_option
)]

pub mod legacy;
pub mod packet;
#[cfg(test)]
mod reference;
pub mod remap;
pub mod tag;
pub mod translate;

use pumpkin_data::packet::CURRENT_MC_VERSION;
use pumpkin_util::version::JavaMinecraftVersion;

pub use packet::JavaConnectionState;
use packet::translator::PacketTranslator;
use translate::{chunk, entity_data, tags};

use crate::legacy::ids;

/// Oldest client with chunks, player info and entity data translated.
pub const OLDEST_SUPPORTED: JavaMinecraftVersion = JavaMinecraftVersion::V_1_7_2;

/// A packet in the client's format: id and payload.
pub type ClientPacket = (i32, Vec<u8>);

/// What the translator decided for one outgoing packet.
#[derive(Debug, Default)]
pub struct Outgoing {
    /// The translated packet; `None` drops it (no counterpart for the client).
    pub packet: Option<ClientPacket>,
    /// Packets to send right after it, already in the client's format.
    pub extras: Vec<ClientPacket>,
}

/// Every client version this build translates, oldest first, excluding the current version.
#[must_use]
pub fn supported_versions() -> Vec<JavaMinecraftVersion> {
    legacy::KNOWN_VERSIONS
        .iter()
        .copied()
        .filter(|&version| version >= OLDEST_SUPPORTED && version != CURRENT_MC_VERSION)
        .collect()
}

/// Translates a play-state packet an older client sent. `None` drops it.
/// `rotation` supplies the player's `(yaw, pitch)` for packets that omit them.
#[must_use]
pub fn incoming_play(
    version: JavaMinecraftVersion,
    packet_id: i32,
    payload: &[u8],
    rotation: impl FnOnce() -> (f32, f32),
) -> Option<ClientPacket> {
    if version == CURRENT_MC_VERSION {
        return Some((packet_id, payload.to_vec()));
    }
    PacketTranslator::translate_incoming_packet(packet_id, payload, version, rotation)
}

/// Translates a play-state packet for an older client, with the extra packets it needs
/// (join tags, split-out light). `viewer` keys the client's per-connection state.
#[must_use]
pub fn outgoing_play(
    viewer: u128,
    version: JavaMinecraftVersion,
    packet_id: i32,
    payload: &[u8],
) -> Outgoing {
    if version == CURRENT_MC_VERSION {
        return Outgoing {
            packet: Some((packet_id, payload.to_vec())),
            extras: Vec::new(),
        };
    }
    let is_join = packet_id == ids::clientbound::play::LOGIN.current();
    if is_join {
        entity_data::join_viewer(viewer);
    }
    // Versions below 1.18 want light as its own packet; chunk::chunk_from_current only
    // produces the chunk half, so the original payload is kept to build it separately.
    let is_chunk = packet_id == ids::clientbound::play::LEVEL_CHUNK_WITH_LIGHT.current();
    let packet = entity_data::for_viewer(viewer, || {
        PacketTranslator::translate_outgoing_packet(packet_id, payload, version)
    });
    let mut extras = Vec::new();
    if packet.is_some() {
        if is_join
            && let Some(tag_payload) = tags::join_tags(version)
            && let Some(id) = PacketTranslator::translate_clientbound_packet_id(
                ids::clientbound::play::UPDATE_TAGS.current(),
                version,
            )
        {
            extras.push((id, tag_payload));
        }
        if is_chunk
            && let Some(light) = chunk::light_from_current(payload, version)
            && let Some(id) = PacketTranslator::translate_clientbound_packet_id(
                ids::clientbound::play::LIGHT_UPDATE.current(),
                version,
            )
        {
            extras.push((id, light));
        }
    }
    Outgoing { packet, extras }
}

/// Translates a handshake/status/login/configuration packet an older client sent.
#[must_use]
pub fn incoming_connection(
    state: JavaConnectionState,
    version: JavaMinecraftVersion,
    packet_id: i32,
    payload: &[u8],
) -> Option<ClientPacket> {
    if version == CURRENT_MC_VERSION {
        return Some((packet_id, payload.to_vec()));
    }
    PacketTranslator::translate_connection_incoming(state, packet_id, payload, version)
}

/// Translates a handshake/status/login/configuration packet for an older client.
#[must_use]
pub fn outgoing_connection(
    state: JavaConnectionState,
    version: JavaMinecraftVersion,
    packet_id: i32,
    payload: &[u8],
) -> Option<ClientPacket> {
    if version == CURRENT_MC_VERSION {
        return Some((packet_id, payload.to_vec()));
    }
    PacketTranslator::translate_connection_outgoing(state, packet_id, payload, version)
}

/// Drops the translation state of a leaving client.
pub fn forget_viewer(viewer: u128) {
    entity_data::forget_viewer(viewer);
}
