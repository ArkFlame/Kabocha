// Last verified for v2169

use pumpkin_macros::packet;
use pumpkin_util::math::position::BlockPos;

use crate::serial::PacketRead;

/// Sent by the client when picking a targeted block (middle-click / pick block action).
#[derive(Debug, PacketRead)]
#[packet(34)]
pub struct SBlockPickRequest {
    /// World position coordinates of the targeted block.
    pub position: BlockPos,
    /// Whether block entity data / NBT should be copied into the picked item stack.
    pub with_data: bool,
    /// Number of hotbar slots the server may use for the picked item (not a slot index).
    pub max_slots: u8,
}
