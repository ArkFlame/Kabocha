use std::{io, net::SocketAddr, pin::Pin, sync::Arc};

use axum::{
    Router,
    body::Bytes,
    extract::{ConnectInfo, DefaultBodyLimit, Path, State},
    http::{
        HeaderMap, HeaderValue, StatusCode, Uri,
        header::{CONTENT_TYPE, HOST},
    },
    response::{IntoResponse, Response},
    routing::{get, post},
    serve::Listener,
};
use pumpkin_auth::{jwt::Jwks, p384::ecdsa::SigningKey};
use pumpkin_world::{CURRENT_BEDROCK_MC_PROTOCOL, CURRENT_BEDROCK_MC_VERSION};
use serde::Serialize;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpListener,
    sync::{Mutex, OnceCell, mpsc},
};
use tokio_rustls::{
    TlsAcceptor,
    rustls::{
        ServerConfig,
        pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer},
    },
};
use tracing::{debug, info, trace, warn};

use crate::server::Server;

use super::ice_socket::IceSocket;
use super::state::NetherNetState;
use super::{
    discovery::NetherNetDiscovery, ice_router::IceRouter, peer::negotiate_direct,
    session::IncomingSession,
};

const MAX_SDP_SIZE: usize = 1 << 20;

/// Accepts Bedrock `NetherNet` connections negotiated through Mojang's HTTP endpoint.
pub struct NetherNetListener {
    incoming: Mutex<mpsc::Receiver<IncomingSession>>,
    local_addr: SocketAddr,
}

impl NetherNetListener {
    pub async fn bind(
        server: &Arc<Server>,
        identity_key: Arc<SigningKey>,
        oidc_verifier: Option<Arc<OnceCell<(String, Jwks)>>>,
        ice_socket: IceSocket,
    ) -> std::io::Result<Self> {
        let config = &server.advanced_config.networking.bedrock.nethernet;
        let address = config.address;
        let listener = TcpListener::bind(address).await?;
        let local_addr = listener.local_addr()?;
        let tls = TlsAcceptor::from(Arc::new(signaling_tls_config()?));
        let ice_router = Arc::new(
            IceRouter::bind(
                ice_socket,
                Arc::downgrade(server),
                server.server_guid,
                server.stop_token.clone(),
            )
            .await?,
        );
        let ice_local_addr = ice_router.public_addr();
        let (incoming, receiver) = mpsc::channel(128);
        let state = NetherNetState {
            server: Arc::downgrade(server),
            incoming,
            identity_key,
            require_client_identity: server.advanced_config.networking.bedrock.online_mode,
            oidc_verifier,
            stun_servers: config.stun_servers.clone().into(),
            ice_local_addr,
            external_ip: config.external_ip,
            ice_router,
            stop_token: server.stop_token.clone(),
        };
        let router = Router::new()
            .route("/v1/join", get(status))
            .route("/join", get(status))
            .route("/v1/ping", get(status))
            .route("/ping", get(status))
            .route("/status", get(status))
            .route("/v1/join/{network_id}", post(join))
            .fallback(unknown_request)
            .layer(DefaultBodyLimit::max(MAX_SDP_SIZE))
            .with_state(state.clone());

        match NetherNetDiscovery::bind(local_addr, server.server_guid).await {
            Ok(discovery) => {
                if let Ok(address) = discovery.local_addr() {
                    info!("Bedrock NetherNet LAN discovery is listening on {address}");
                }
                let state = state.clone();
                tokio::spawn(async move {
                    let mut buffer = vec![0; 65_535];
                    loop {
                        tokio::select! {
                            () = state.stop_token.cancelled() => break,
                            result = discovery.receive(&state, &mut buffer) => {
                                if let Err(error) = result {
                                    debug!("NetherNet LAN discovery packet failed: {error}");
                                }
                            }
                        }
                    }
                });
            }
            Err(error) => warn!("Failed to bind NetherNet LAN discovery: {error}"),
        }

        let stop_token = server.stop_token.clone();
        tokio::spawn(async move {
            let result = axum::serve(
                SignalingListener::new(listener, tls),
                router.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .with_graceful_shutdown(stop_token.cancelled_owned())
            .await;
            if let Err(error) = result {
                warn!("NetherNet signaling server stopped: {error}");
            }
        });

        info!("Bedrock NetherNet signaling is listening on {local_addr}");
        info!("Bedrock NetherNet ICE is listening on {ice_local_addr}");
        Ok(Self {
            incoming: Mutex::new(receiver),
            local_addr,
        })
    }

    pub async fn accept(&self) -> Option<IncomingSession> {
        self.incoming.lock().await.recv().await
    }

    pub const fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }
}

/// Builds the signaling endpoint's TLS configuration.
///
/// Bedrock authenticates NetherNet peers through the identity signed into the SDP
/// offer, not through the TLS certificate, so a self-signed certificate is enough.
fn signaling_tls_config() -> io::Result<ServerConfig> {
    let key = rcgen::generate_simple_self_signed(vec!["localhost".to_string()])
        .map_err(io::Error::other)?;
    let certificate = CertificateDer::from(key.cert.der().to_vec());
    let private_key =
        PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key.signing_key.serialize_der()));
    let mut config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![certificate], private_key)
        .map_err(io::Error::other)?;
    config.alpn_protocols = vec![b"http/1.1".to_vec()];
    Ok(config)
}

trait AsyncStream: AsyncRead + AsyncWrite + Send {}
impl<T: AsyncRead + AsyncWrite + Send> AsyncStream for T {}

/// Accepts both the TLS connections current Bedrock clients open and the plaintext
/// HTTP connections older clients use on the same signaling port.
struct SignalingListener {
    listener: TcpListener,
    tls: TlsAcceptor,
}

impl SignalingListener {
    fn new(listener: TcpListener, tls: TlsAcceptor) -> Self {
        Self { listener, tls }
    }
}

impl Listener for SignalingListener {
    type Io = Pin<Box<dyn AsyncStream>>;
    type Addr = SocketAddr;

    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        loop {
            let (stream, address) = match self.listener.accept().await {
                Ok(connection) => connection,
                Err(error) => {
                    debug!("NetherNet signaling accept failed: {error}");
                    continue;
                }
            };
            let mut first = [0u8; 1];
            match stream.peek(&mut first).await {
                Ok(0) => continue,
                Ok(_) if first[0] == 0x16 => match self.tls.accept(stream).await {
                    Ok(stream) => return (Box::pin(stream), address),
                    Err(error) => debug!(%address, "NetherNet TLS handshake failed: {error}"),
                },
                Ok(_) => return (Box::pin(stream), address),
                Err(error) => debug!(%address, "NetherNet signaling peek failed: {error}"),
            }
        }
    }

    fn local_addr(&self) -> io::Result<Self::Addr> {
        self.listener.local_addr()
    }
}

async fn unknown_request(ConnectInfo(address): ConnectInfo<SocketAddr>, uri: Uri) -> Response {
    debug!(%address, %uri, "Unhandled NetherNet signaling request");
    (StatusCode::NOT_FOUND, "Unknown NetherNet endpoint").into_response()
}

/// Server description returned by `GET /v1/join`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct NetherNetStatus<'a> {
    name: &'a str,
    protocol: u32,
    version: &'static str,
    level: &'a str,
    players: u32,
    max_players: u32,
    game_type: u8,
}

async fn status(
    State(state): State<NetherNetState>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
) -> Response {
    let Some(server) = state.server.upgrade() else {
        warn!(%address, "Rejected NetherNet status request because the server is stopping");
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };
    let players = u32::try_from(server.get_player_count()).unwrap_or(u32::MAX);
    let game_type = server
        .defaultgamemode
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .gamemode as u8;
    let status = NetherNetStatus {
        name: &server.advanced_config.networking.bedrock.motd,
        protocol: CURRENT_BEDROCK_MC_PROTOCOL,
        version: CURRENT_BEDROCK_MC_VERSION,
        level: &server.basic_config.default_level_name,
        players,
        max_players: server.advanced_config.networking.bedrock.max_players,
        game_type,
    };
    let Ok(body) = serde_json::to_vec(&status) else {
        warn!(%address, "Failed to encode NetherNet server status");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    let mut response = (StatusCode::OK, body).into_response();
    response
        .headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    trace!(
        %address,
        players,
        max_players = status.max_players,
        protocol = status.protocol,
        "Sent NetherNet server status"
    );
    response
}

async fn join(
    State(state): State<NetherNetState>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    Path(network_id): Path<String>,
    uri: Uri,
    headers: HeaderMap,
    offer: Bytes,
) -> Response {
    trace!(%address, %network_id, length = offer.len(), "Received NetherNet SDP offer");
    if offer.is_empty() {
        debug!(%address, %network_id, "Rejected empty NetherNet SDP offer");
        return (StatusCode::BAD_REQUEST, "Missing SDP offer").into_response();
    }
    let Ok(offer) = String::from_utf8(offer.to_vec()) else {
        debug!(%address, %network_id, "Rejected non-UTF-8 NetherNet SDP offer");
        return (StatusCode::BAD_REQUEST, "SDP offer must be UTF-8").into_response();
    };

    let host = headers.get(HOST).and_then(|host| host.to_str().ok());
    let uri_authority = uri.authority().map(axum::http::uri::Authority::as_str);
    let advertised_ip = host
        .or(uri_authority)
        .and_then(|host| host.parse::<axum::http::uri::Authority>().ok())
        .and_then(|authority| authority.host().parse().ok());
    trace!(
        ?host,
        ?uri_authority,
        ?advertised_ip,
        "Resolved NetherNet advertised address"
    );
    match Box::pin(negotiate_direct(&state, address, &offer, advertised_ip)).await {
        Ok((answer, _session)) => {
            trace!(%address, %network_id, length = answer.len(), "Returning NetherNet SDP answer");
            let mut response = (StatusCode::OK, answer).into_response();
            response
                .headers_mut()
                .insert(CONTENT_TYPE, HeaderValue::from_static("application/sdp"));
            response
        }
        Err(error) => {
            warn!("NetherNet negotiation with {address} failed: {error}");
            (StatusCode::BAD_REQUEST, error).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signaling_tls_config_negotiates_http1() {
        let config = signaling_tls_config().unwrap();
        assert_eq!(config.alpn_protocols, vec![b"http/1.1".to_vec()]);
    }
}
