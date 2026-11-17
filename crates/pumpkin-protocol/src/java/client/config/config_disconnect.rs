use pumpkin_data::packet::clientbound::config::DISCONNECT;
use pumpkin_macros::java_packet;
use pumpkin_util::text::TextComponent;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(DISCONNECT)]
pub struct CConfigDisconnect<'a> {
    /// The message shown to the player. A full text component: colour, style and
    /// translation keys survive, like vanilla's `ClientboundDisconnectPacket(Component)`.
    pub reason: &'a TextComponent,
}

impl<'a> CConfigDisconnect<'a> {
    #[must_use]
    pub const fn new(reason: &'a TextComponent) -> Self {
        Self { reason }
    }
}

impl ClientPacket for CConfigDisconnect<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        // The reason is a text component (NBT since 1.20.3), not a plain string.
        write.write_component(self.reason, version)
    }
}
