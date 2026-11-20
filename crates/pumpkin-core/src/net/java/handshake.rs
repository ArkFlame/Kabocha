use pumpkin_data::{
    packet::{CURRENT_MC_VERSION, LOWEST_SUPPORTED_MC_VERSION},
    translation,
};
use pumpkin_protocol::{ConnectionState, java::server::handshake::SHandShake};
use pumpkin_util::{text::TextComponent, version::JavaMinecraftVersion};
use tracing::debug;

use crate::{
    net::{
        java::pending::PendingConnection,
        rate_limit::{Kind, Verdict, check_shared},
    },
    server::Server,
};
use std::sync::Arc;
use std::time::Instant;

impl PendingConnection {
    /// Applies the status or login rate limit for the handshake intent. `true` means drop.
    #[must_use]
    pub fn handshake_rate_limited(&self, server: &Server, next_state: ConnectionState) -> bool {
        let kind = match next_state {
            ConnectionState::Status => Kind::Status,
            ConnectionState::Login | ConnectionState::Transfer => Kind::Login,
            _ => return false,
        };
        check_shared(
            &server.rate_limiter,
            self.address.ip(),
            kind,
            Instant::now(),
        ) == Verdict::Drop
    }

    pub async fn handle_handshake(&mut self, server: &Arc<Server>, handshake: SHandShake) {
        let version = handshake.protocol_version.0 as u32;
        self.server_address = handshake.server_address.to_string();
        self.version
            .store(JavaMinecraftVersion::from_protocol(version));

        debug!("Handshake: next state is {:?}", &handshake.next_state);
        self.connection_state.store(handshake.next_state);
        if handshake.next_state == ConnectionState::Transfer
            && !server.basic_config.accepts_transfers
        {
            self.kick(TextComponent::translate_cross(
                translation::java::MULTIPLAYER_DISCONNECT_TRANSFERS_DISABLED,
                translation::java::MULTIPLAYER_DISCONNECT_TRANSFERS_DISABLED,
                [],
            ))
            .await;
            return;
        }
        if self.connection_state.load() != ConnectionState::Status {
            let protocol = version;
            // A plugin translating this version (see `register_java_versions`) admits it.
            if server.advanced_config.multiversion.enabled
                && pumpkin_multiversion::supported_versions()
                    .contains(&JavaMinecraftVersion::from_protocol(protocol))
            {
                return;
            }
            if server
                .plugin_manager
                .java_versions
                .admits_protocol(protocol)
            {
                return;
            }
            if protocol < LOWEST_SUPPORTED_MC_VERSION.protocol_version() as u32 {
                self.kick(TextComponent::translate_cross(
                    translation::java::MULTIPLAYER_DISCONNECT_OUTDATED_CLIENT,
                    translation::bedrock::DISCONNECTIONSCREEN_OUTDATEDCLIENT,
                    [TextComponent::text(CURRENT_MC_VERSION.to_string())],
                ))
                .await;
            } else if protocol > CURRENT_MC_VERSION.protocol_version() as u32 {
                self.kick(TextComponent::translate_cross(
                    translation::java::MULTIPLAYER_DISCONNECT_OUTDATED_SERVER,
                    translation::bedrock::DISCONNECTIONSCREEN_OUTDATEDSERVER,
                    [TextComponent::text(CURRENT_MC_VERSION.to_string())],
                ))
                .await;
            }
        }
    }
}
