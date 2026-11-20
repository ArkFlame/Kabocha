//! Pre-26.3 encodings from `pumpkin-protocol`'s `java/client/config/add_resource_pack.rs`.

use crate::legacy::{LegacyWrite, LegacyWriteExt};
use pumpkin_protocol::java::client::config::CConfigAddResourcePack;
use pumpkin_protocol::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

impl LegacyWrite for CConfigAddResourcePack<'_> {
    fn write_legacy(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), pumpkin_protocol::ser::WritingError> {
        if *version >= JavaMinecraftVersion::V_1_20_3 {
            write.write_uuid(self.uuid)?;
        }
        write.write_string(self.url)?;
        write.write_string(self.hash)?;
        // Forced and prompt message did not exist before 1.17: url and hash only.
        if *version >= JavaMinecraftVersion::V_1_17 {
            write.write_bool(self.forced)?;
            if let Some(prompt) = &self.prompt_message {
                write.write_bool(true)?;
                write.write_component_legacy(prompt, version)?;
            } else {
                write.write_bool(false)?;
            }
        }
        Ok(())
    }
}
