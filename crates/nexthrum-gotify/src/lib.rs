use futures_util::StreamExt;
use nexthrum_core::GotifyMessage;
use tokio::sync::{mpsc, watch};
use tokio_tungstenite::{connect_async, tungstenite::Message};

static CRYPTO_PROVIDER: std::sync::Once = std::sync::Once::new();

/// Install the process-wide rustls crypto provider exactly once.
///
/// Without this, the first TLS handshake panics with "Could not automatically
/// determine the process-level CryptoProvider" because the WebSocket stack only
/// enables rustls' `std` feature. Hosts may install their own provider first;
/// that case is ignored.
pub fn ensure_crypto_provider() {
    CRYPTO_PROVIDER.call_once(|| {
        if rustls::crypto::ring::default_provider()
            .install_default()
            .is_err()
        {
            log::debug!("A rustls crypto provider was already installed");
        }
    });
}

/// Typed failure for Gotify HTTP operations so hosts can react precisely
/// (prompt for a new token vs. retry later vs. fix the URL).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GotifyError {
    /// The server URL is malformed or does not use HTTP(S).
    InvalidUrl,
    /// The server rejected the client token (HTTP 401/403).
    Unauthorized,
    /// The endpoint is missing (HTTP 404) — usually a wrong base path.
    NotFound,
    /// Any other non-success HTTP status.
    HttpStatus(u16),
    /// The response body could not be parsed as the expected JSON payload.
    InvalidResponse(String),
    /// Transport failure before a response was received (DNS, refused, TLS).
    Network(String),
    /// The request exceeded the client timeout.
    Timeout,
    /// TLS negotiation failed (usually a self-signed or expired certificate).
    Tls(String),
}

impl std::fmt::Display for GotifyError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidUrl => formatter.write_str("Gotify server URL must use HTTP or HTTPS"),
            Self::Unauthorized => {
                formatter.write_str("The Gotify server rejected the client token (HTTP 401/403)")
            }
            Self::NotFound => formatter.write_str(
                "The Gotify API endpoint was not found (HTTP 404); check the server URL path",
            ),
            Self::HttpStatus(status) => {
                write!(formatter, "The Gotify server returned HTTP {status}")
            }
            Self::InvalidResponse(detail) => {
                write!(formatter, "Unexpected Gotify response: {detail}")
            }
            Self::Network(detail) => write!(formatter, "Cannot reach the Gotify server: {detail}"),
            Self::Timeout => formatter.write_str("The Gotify server did not respond in time"),
            Self::Tls(detail) => write!(
                formatter,
                "TLS handshake with the Gotify server failed: {detail}"
            ),
        }
    }
}

impl std::error::Error for GotifyError {}

fn classify_transport_error(error: &reqwest::Error) -> GotifyError {
    if error.is_timeout() {
        return GotifyError::Timeout;
    }
    let detail = error.to_string();
    let lowered = detail.to_lowercase();
    if lowered.contains("certificate") || lowered.contains("tls") || lowered.contains("ssl") {
        return GotifyError::Tls(detail);
    }
    GotifyError::Network(detail)
}

fn classify_status(status: reqwest::StatusCode) -> Option<GotifyError> {
    if status.is_success() {
        return None;
    }
    Some(match status.as_u16() {
        401 | 403 => GotifyError::Unauthorized,
        404 => GotifyError::NotFound,
        other => GotifyError::HttpStatus(other),
    })
}

#[derive(Debug)]
pub enum GotifyEvent {
    Connecting,
    Connected,
    /// The stream is not (or no longer) connected; the reason is user-facing.
    Disconnected {
        reason: Option<String>,
    },
    Message(GotifyMessage),
}

/// A WebSocket handshake that neither succeeds nor fails would otherwise leave
/// hosts stuck on "connecting" forever.
pub const WEBSOCKET_CONNECT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
const RECONNECT_DELAY: std::time::Duration = std::time::Duration::from_secs(5);

/// Turn a tungstenite/transport failure into an actionable reason.
fn describe_connect_error(error: &tokio_tungstenite::tungstenite::Error) -> String {
    use tokio_tungstenite::tungstenite::Error as WsError;
    let detail = error.to_string();
    let lowered = detail.to_lowercase();
    match error {
        WsError::Http(response) => {
            let status = response.status();
            match status.as_u16() {
                401 | 403 => {
                    "The Gotify server rejected the client token while opening the stream".into()
                }
                404 => "The Gotify stream endpoint was not found; check the server URL path".into(),
                other => format!("The Gotify server returned HTTP {other} for the stream"),
            }
        }
        WsError::Url(_) => "The Gotify WebSocket URL is invalid".into(),
        _ if lowered.contains("certificate")
            || lowered.contains("tls")
            || lowered.contains("ssl") =>
        {
            format!("TLS handshake with the Gotify server failed: {detail}")
        }
        _ if lowered.contains("dns") || lowered.contains("failed to lookup") => {
            format!("Cannot resolve the Gotify host: {detail}")
        }
        _ => format!("WebSocket connection failed: {detail}"),
    }
}

/// Same as [`run_websocket_client`] with an explicit handshake timeout.
pub async fn run_websocket_client_with_timeout(
    server_url: String,
    client_token: String,
    events: mpsc::Sender<GotifyEvent>,
    shutdown: watch::Receiver<bool>,
    connect_timeout: std::time::Duration,
) {
    run_client(server_url, client_token, events, shutdown, connect_timeout).await
}

/// Own the Gotify WebSocket reconnect loop without depending on any desktop host.
pub async fn run_websocket_client(
    server_url: String,
    client_token: String,
    events: mpsc::Sender<GotifyEvent>,
    shutdown: watch::Receiver<bool>,
) {
    run_client(
        server_url,
        client_token,
        events,
        shutdown,
        WEBSOCKET_CONNECT_TIMEOUT,
    )
    .await
}

async fn run_client(
    server_url: String,
    client_token: String,
    events: mpsc::Sender<GotifyEvent>,
    mut shutdown: watch::Receiver<bool>,
    connect_timeout: std::time::Duration,
) {
    ensure_crypto_provider();
    let url = match websocket_url(&server_url, &client_token) {
        Ok(url) => url,
        Err(error) => {
            log::error!("Invalid Gotify WebSocket configuration: {error}");
            let _ = events
                .send(GotifyEvent::Disconnected {
                    reason: Some("The Gotify server URL must use HTTP or HTTPS".into()),
                })
                .await;
            return;
        }
    };

    loop {
        if *shutdown.borrow() {
            return;
        }
        if events.send(GotifyEvent::Connecting).await.is_err() {
            return;
        }
        let connected = tokio::select! {
            result = tokio::time::timeout(connect_timeout, connect_async(&url)) => Some(result),
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() {
                    return;
                }
                None
            }
        };
        let Some(connected) = connected else {
            continue;
        };
        match connected {
            Err(_elapsed) => {
                let reason = format!(
                    "The Gotify server did not complete the WebSocket handshake within {}s",
                    connect_timeout.as_secs()
                );
                log::warn!("{reason}");
                if events
                    .send(GotifyEvent::Disconnected {
                        reason: Some(reason),
                    })
                    .await
                    .is_err()
                {
                    return;
                }
            }
            Ok(Err(error)) => {
                let reason = describe_connect_error(&error);
                log::warn!("Gotify WebSocket connection failed: {reason}");
                if events
                    .send(GotifyEvent::Disconnected {
                        reason: Some(reason),
                    })
                    .await
                    .is_err()
                {
                    return;
                }
            }
            Ok(Ok((socket, _))) => {
                if events.send(GotifyEvent::Connected).await.is_err() {
                    return;
                }
                let mut stream = socket;
                let close_reason = loop {
                    tokio::select! {
                        changed = shutdown.changed() => {
                            if changed.is_err() || *shutdown.borrow() {
                                return;
                            }
                        }
                        item = stream.next() => {
                            match item {
                                Some(Ok(Message::Text(text))) => match decode_message(&text) {
                                    Ok(message) => {
                                        if events.send(GotifyEvent::Message(message)).await.is_err() {
                                            return;
                                        }
                                    }
                                    Err(error) => log::warn!("Failed to decode Gotify message: {error}"),
                                },
                                Some(Ok(Message::Close(frame))) => {
                                    break frame
                                        .map(|frame| format!("The Gotify stream was closed: {}", frame.reason))
                                        .unwrap_or_else(|| "The Gotify stream was closed by the server".to_string());
                                }
                                None => break "The Gotify stream ended".to_string(),
                                Some(Err(error)) => break format!("Gotify stream error: {error}"),
                                Some(Ok(Message::Ping(_))) | Some(Ok(_)) => {}
                            }
                        }
                    }
                };
                if events
                    .send(GotifyEvent::Disconnected {
                        reason: Some(close_reason),
                    })
                    .await
                    .is_err()
                {
                    return;
                }
            }
        }

        tokio::select! {
            _ = tokio::time::sleep(RECONNECT_DELAY) => {}
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() {
                    return;
                }
            }
        }
    }
}

/// Convert a Gotify HTTP(S) base URL into its authenticated WebSocket stream URL.
pub fn websocket_url(server_url: &str, client_token: &str) -> Result<String, String> {
    let mut url =
        url::Url::parse(server_url.trim()).map_err(|_| "Invalid Gotify server URL".to_string())?;
    let websocket_scheme = match url.scheme() {
        "http" => "ws".to_string(),
        "https" => "wss".to_string(),
        "ws" | "wss" => url.scheme().to_string(),
        _ => return Err("Gotify server URL must use HTTP or HTTPS".to_string()),
    };
    url.set_scheme(&websocket_scheme)
        .map_err(|_| "Invalid Gotify server URL scheme".to_string())?;
    let path = format!("{}/stream", url.path().trim_end_matches('/'));
    url.set_path(&path);
    url.set_query(None);
    url.set_fragment(None);
    url.query_pairs_mut().append_pair("token", client_token);
    Ok(url.into())
}

/// Decode one server payload into the stable message DTO.
pub fn decode_message(payload: &str) -> Result<GotifyMessage, serde_json::Error> {
    serde_json::from_str(payload)
}

fn api_url(server_url: &str, endpoint: &str) -> Result<url::Url, GotifyError> {
    let mut url = url::Url::parse(server_url.trim()).map_err(|_| GotifyError::InvalidUrl)?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(GotifyError::InvalidUrl);
    }
    let base_path = url.path().trim_end_matches('/');
    url.set_path(&format!("{base_path}/{endpoint}"));
    url.set_query(None);
    url.set_fragment(None);
    Ok(url)
}

/// List the applications (message sources) so hosts can show real names.
///
/// Gotify only puts `appid` in streamed messages, so names require this call.
/// Some servers restrict `/application`; callers must fall back to `App <id>`.
pub async fn list_applications(
    server_url: &str,
    client_token: &str,
) -> Result<Vec<nexthrum_core::GotifyApplication>, GotifyError> {
    ensure_crypto_provider();
    let url = api_url(server_url, "application")?;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| GotifyError::Network(e.to_string()))?;
    let response = client
        .get(url)
        .header("X-Gotify-Key", client_token)
        .send()
        .await
        .map_err(|e| classify_transport_error(&e))?;

    if let Some(error) = classify_status(response.status()) {
        return Err(error);
    }

    let payload: serde_json::Value = response
        .json()
        .await
        .map_err(|e| GotifyError::InvalidResponse(e.to_string()))?;
    let entries = payload.as_array().ok_or_else(|| {
        GotifyError::InvalidResponse("expected an array of applications".to_string())
    })?;

    Ok(entries
        .iter()
        .filter_map(|entry| {
            let id = entry.get("id")?.as_i64()?;
            let name = entry.get("name").and_then(|value| value.as_str())?;
            Some(nexthrum_core::GotifyApplication {
                id,
                name: name.to_string(),
            })
        })
        .collect())
}

/// Delete all server-side messages. A non-success response is an error so hosts
/// can avoid claiming synchronization succeeded or clearing local history.
pub async fn delete_all_messages(server_url: &str, client_token: &str) -> Result<(), GotifyError> {
    ensure_crypto_provider();
    let url = api_url(server_url, "message")?;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| GotifyError::Network(e.to_string()))?;
    let response = client
        .delete(url)
        .header("X-Gotify-Key", client_token)
        .send()
        .await
        .map_err(|e| classify_transport_error(&e))?;
    if let Some(error) = classify_status(response.status()) {
        return Err(error);
    }
    Ok(())
}

/// Verify a Gotify server and return its reported version.
pub async fn test_connection(server_url: &str, client_token: &str) -> Result<String, GotifyError> {
    ensure_crypto_provider();
    let url = api_url(server_url, "version")?;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| GotifyError::Network(e.to_string()))?;
    let response = client
        .get(url)
        .header("X-Gotify-Key", client_token)
        .send()
        .await
        .map_err(|e| classify_transport_error(&e))?;

    if let Some(error) = classify_status(response.status()) {
        return Err(error);
    }

    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| GotifyError::InvalidResponse(e.to_string()))?;
    let version = body
        .get("version")
        .and_then(|value| value.as_str())
        .unwrap_or("unknown");
    Ok(format!("Connected! Gotify server version: {version}"))
}

#[cfg(test)]
mod tests {
    use super::{api_url, decode_message, run_websocket_client, websocket_url};

    #[test]
    fn websocket_url_encodes_token_and_preserves_base_path() {
        let url = websocket_url("https://example.test/gotify/", "token +/&=?").unwrap();
        let parsed = url::Url::parse(&url).unwrap();
        assert_eq!(parsed.scheme(), "wss");
        assert_eq!(parsed.path(), "/gotify/stream");
        assert_eq!(
            parsed
                .query_pairs()
                .find(|(key, _)| key == "token")
                .unwrap()
                .1,
            "token +/&=?"
        );
    }

    #[test]
    fn http_api_urls_preserve_base_path_and_strip_query_and_fragment() {
        let url = api_url("https://example.test/gotify/?old=1#fragment", "message").unwrap();
        assert_eq!(url.as_str(), "https://example.test/gotify/message");
        assert_eq!(
            api_url("https://example.test", "version").unwrap().path(),
            "/version"
        );
        assert!(api_url("file:///tmp/gotify", "version").is_err());
    }

    #[test]
    fn websocket_url_replaces_existing_query_and_fragment() {
        let url =
            websocket_url("https://example.test/gotify/?token=stale#fragment", "fresh").unwrap();
        let parsed = url::Url::parse(&url).unwrap();
        assert_eq!(parsed.path(), "/gotify/stream");
        assert_eq!(parsed.query_pairs().count(), 1);
        assert_eq!(
            parsed.query_pairs().next().unwrap(),
            ("token".into(), "fresh".into())
        );
        assert!(parsed.fragment().is_none());
    }

    #[test]
    fn websocket_url_rejects_unsupported_scheme() {
        assert!(websocket_url("file:///tmp/gotify", "token").is_err());
    }

    #[test]
    fn decoder_maps_gotify_payload() {
        let payload = r#"{"id":12,"appid":3,"message":"hello","title":"test","priority":5,"date":"2026-01-01T00:00:00Z"}"#;
        let message = decode_message(payload).unwrap();
        assert_eq!(message.id, 12);
        assert_eq!(message.appid, 3);
        assert!(!message.read);
    }

    #[tokio::test]
    async fn shutdown_cancels_an_in_progress_websocket_handshake() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (events_tx, mut events_rx) = tokio::sync::mpsc::channel(4);
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let client = tokio::spawn(run_websocket_client(
            format!("http://{address}"),
            "unused".into(),
            events_tx,
            shutdown_rx,
        ));
        let (_socket, _) = listener.accept().await.unwrap();
        assert!(matches!(
            events_rx.recv().await,
            Some(super::GotifyEvent::Connecting)
        ));

        shutdown_tx.send(true).unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(1), client)
            .await
            .expect("client should stop promptly")
            .unwrap();
        assert!(events_rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn invalid_configuration_emits_disconnected_state() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        run_websocket_client("file:///invalid".into(), "unused".into(), tx, shutdown_rx).await;
        assert!(matches!(
            rx.recv().await,
            Some(super::GotifyEvent::Disconnected { .. })
        ));
    }

    #[tokio::test]
    async fn failed_attempt_emits_connecting_then_disconnected_and_can_stop_retrying() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        drop(listener);
        let (events_tx, mut events_rx) = tokio::sync::mpsc::channel(4);
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let client = tokio::spawn(run_websocket_client(
            format!("http://{address}"),
            "unused".into(),
            events_tx,
            shutdown_rx,
        ));

        assert!(matches!(
            events_rx.recv().await,
            Some(super::GotifyEvent::Connecting)
        ));
        assert!(matches!(
            events_rx.recv().await,
            Some(super::GotifyEvent::Disconnected { .. })
        ));
        shutdown_tx.send(true).unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(1), client)
            .await
            .expect("retry wait should be cancelled")
            .unwrap();
    }

    #[tokio::test]
    async fn client_observes_shutdown_before_connecting() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(true);
        run_websocket_client(
            "http://127.0.0.1:1".into(),
            "unused".into(),
            tx,
            shutdown_rx,
        )
        .await;
        assert!(rx.recv().await.is_none());
        drop(shutdown_tx);
    }
}
