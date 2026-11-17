// Last verified for v2169

use pumpkin_macros::packet;
use std::borrow::Cow;
use std::io::Error;
use uuid::Uuid;

use crate::codec::var_uint::VarUInt;
use crate::serial::{PacketRead, PacketReadSlice};

/// `CommandOriginType::DevConsole`. Carries a player unique id on the wire.
pub const COMMAND_ORIGIN_DEV_CONSOLE: u32 = 3;
/// `CommandOriginType::Test`. Carries a player unique id on the wire.
pub const COMMAND_ORIGIN_TEST: u32 = 4;

/// Sent by the client to request server-side execution of a slash command.
#[derive(Debug, PacketRead, PacketReadSlice)]
#[packet(77)]
pub struct SCommandRequest<'a> {
    /// Raw command line string (with leading slash).
    pub command: Cow<'a, str>,
    /// Origin context information identifying the command sender.
    pub origin: CommandOriginData<'a>,
    /// Whether the command originates from an internal engine subsystem.
    pub is_internal: bool,

    // TODO: enum CurrentCmdVersion
    /// Protocol command syntax version string.
    pub version: Cow<'a, str>,
}

/// Metadata describing the origin and sender of an executed command.
#[derive(Debug)]
pub struct CommandOriginData<'a> {
    /// `CommandOriginType` enum value, encoded as a varuint.
    pub origin_type: u32,
    /// UUID uniquely identifying the sender instance.
    pub uuid: Uuid,
    /// Request identifier string for correlating command outputs.
    pub request_id: Cow<'a, str>,
    /// Player unique id. Only present on the wire for dev console and test origins, else `-1`.
    pub player_id: i64,
}

impl PacketRead for CommandOriginData<'_> {
    fn read<R: std::io::Read>(reader: &mut R) -> Result<Self, Error> {
        let origin_type = VarUInt::read(reader)?.0;
        let uuid = Uuid::read(reader)?;
        let request_id = <Cow<'_, str> as PacketRead>::read(reader)?;
        let player_id = if matches!(
            origin_type,
            COMMAND_ORIGIN_DEV_CONSOLE | COMMAND_ORIGIN_TEST
        ) {
            i64::read(reader)?
        } else {
            -1
        };
        Ok(Self {
            origin_type,
            uuid,
            request_id,
            player_id,
        })
    }
}

impl<'a> PacketReadSlice<'a> for CommandOriginData<'a> {
    fn read_slice(buf: &mut &'a [u8]) -> Result<Self, Error> {
        let origin_type = VarUInt::read(&mut *buf)?.0;
        let uuid = Uuid::read(&mut *buf)?;
        let request_id = <Cow<'a, str> as PacketReadSlice<'a>>::read_slice(buf)?;
        let player_id = if matches!(
            origin_type,
            COMMAND_ORIGIN_DEV_CONSOLE | COMMAND_ORIGIN_TEST
        ) {
            i64::read(&mut *buf)?
        } else {
            -1
        };
        Ok(Self {
            origin_type,
            uuid,
            request_id,
            player_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const UUID: [u8; 16] = [0x11; 16];

    fn encode(origin_type: u8, player_id: Option<i64>) -> Vec<u8> {
        let mut bytes = vec![origin_type];
        bytes.extend_from_slice(&UUID);
        bytes.extend_from_slice(&[0x02, b'r', b'q']);
        if let Some(id) = player_id {
            bytes.extend_from_slice(&id.to_le_bytes());
        }
        bytes
    }

    #[test]
    fn player_origin_has_no_player_id_on_wire() -> Result<(), Error> {
        let bytes = encode(0, None);
        let mut slice = bytes.as_slice();
        let origin = CommandOriginData::read_slice(&mut slice)?;
        assert_eq!(origin.origin_type, 0);
        assert_eq!(origin.request_id, "rq");
        assert_eq!(origin.player_id, -1);
        assert!(slice.is_empty());
        Ok(())
    }

    #[test]
    fn dev_console_origin_reads_player_id() -> Result<(), Error> {
        let bytes = encode(COMMAND_ORIGIN_DEV_CONSOLE as u8, Some(42));
        let mut slice = bytes.as_slice();
        let origin = CommandOriginData::read_slice(&mut slice)?;
        assert_eq!(origin.origin_type, COMMAND_ORIGIN_DEV_CONSOLE);
        assert_eq!(origin.player_id, 42);
        assert!(slice.is_empty());
        Ok(())
    }
}
