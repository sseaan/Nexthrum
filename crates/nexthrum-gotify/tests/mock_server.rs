//! Integration tests that exercise the Gotify client against a local mock server.
//!
//! The mock speaks just enough HTTP/1.1 to assert request shape and to drive
//! success, authentication failure, server error and handshake-rejection paths.

use nexthrum_gotify::{
    delete_all_messages, run_websocket_client, test_connection, GotifyError, GotifyEvent,
};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::{mpsc, oneshot, watch};

struct MockServer {
    base_url: String,
    request: oneshot::Receiver<String>,
}

/// Spawn a one-shot HTTP server that replies with `response` and reports the
/// raw request it received.
async fn mock_server(status_line: &str, body: &str) -> MockServer {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address: SocketAddr = listener.local_addr().unwrap();
    let (request_tx, request_rx) = oneshot::channel();
    let status_line = status_line.to_string();
    let body = body.to_string();

    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buffer = vec![0u8; 4096];
        let mut request = String::new();
        loop {
            let read = socket.read(&mut buffer).await.unwrap();
            if read == 0 {
                break;
            }
            request.push_str(&String::from_utf8_lossy(&buffer[..read]));
            if request.contains("\r\n\r\n") {
                break;
            }
        }
        let _ = request_tx.send(request);
        let response = format!(
            "HTTP/1.1 {status_line}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
            body.len()
        );
        let _ = socket.write_all(response.as_bytes()).await;
        let _ = socket.flush().await;
    });

    MockServer {
        base_url: format!("http://{address}"),
        request: request_rx,
    }
}

#[tokio::test]
async fn test_connection_sends_the_token_and_returns_the_server_version() {
    let server = mock_server("200 OK", r#"{"version":"2.6.1"}"#).await;
    let result = test_connection(&server.base_url, "secret-token")
        .await
        .unwrap();
    assert!(result.contains("2.6.1"), "unexpected result: {result}");

    let request = server.request.await.unwrap();
    assert!(request.starts_with("GET /version "), "request: {request}");
    assert!(
        request
            .to_lowercase()
            .contains("x-gotify-key: secret-token"),
        "request: {request}"
    );
}

#[tokio::test]
async fn test_connection_preserves_a_server_base_path() {
    let server = mock_server("200 OK", r#"{"version":"2.6.1"}"#).await;
    let base = format!("{}/gotify?ignored=1", server.base_url);
    test_connection(&base, "secret-token").await.unwrap();
    let request = server.request.await.unwrap();
    assert!(
        request.starts_with("GET /gotify/version "),
        "request: {request}"
    );
}

#[tokio::test]
async fn test_connection_reports_rejected_credentials() {
    let server = mock_server("401 Unauthorized", r#"{"error":"Unauthorized"}"#).await;
    let error = test_connection(&server.base_url, "wrong-token")
        .await
        .unwrap_err();
    assert_eq!(error, GotifyError::Unauthorized);
    assert!(error.to_string().contains("token"));
}

#[tokio::test]
async fn test_connection_forbidden_is_also_unauthorized() {
    let server = mock_server("403 Forbidden", "").await;
    let error = test_connection(&server.base_url, "wrong-token")
        .await
        .unwrap_err();
    assert_eq!(error, GotifyError::Unauthorized);
}

#[tokio::test]
async fn test_connection_reports_a_missing_endpoint() {
    let server = mock_server("404 Not Found", "").await;
    let error = test_connection(&server.base_url, "secret-token")
        .await
        .unwrap_err();
    assert_eq!(error, GotifyError::NotFound);
    assert!(error.to_string().contains("404"));
}

#[tokio::test]
async fn test_connection_reports_server_errors() {
    let server = mock_server("500 Internal Server Error", "").await;
    let error = test_connection(&server.base_url, "secret-token")
        .await
        .unwrap_err();
    assert_eq!(error, GotifyError::HttpStatus(500));
}

#[tokio::test]
async fn test_connection_reports_malformed_json() {
    let server = mock_server("200 OK", "not-json").await;
    let error = test_connection(&server.base_url, "secret-token")
        .await
        .unwrap_err();
    assert!(matches!(error, GotifyError::InvalidResponse(_)));
}

#[tokio::test]
async fn test_connection_reports_unreachable_servers() {
    // Bind and immediately drop a listener to obtain a closed port.
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    drop(listener);

    let error = test_connection(&format!("http://{address}"), "secret-token")
        .await
        .unwrap_err();
    // An HTTP proxy in the environment may answer instead of failing at the
    // transport layer, so accept a classified status as well.
    assert!(
        matches!(
            error,
            GotifyError::Network(_) | GotifyError::Timeout | GotifyError::HttpStatus(_)
        ),
        "unexpected error: {error:?}"
    );
}

#[tokio::test]
async fn invalid_server_urls_are_rejected_before_any_request() {
    let error = test_connection("ftp://example.test", "secret-token")
        .await
        .unwrap_err();
    assert_eq!(error, GotifyError::InvalidUrl);

    let error = test_connection("not a url", "secret-token")
        .await
        .unwrap_err();
    assert_eq!(error, GotifyError::InvalidUrl);
}

#[tokio::test]
async fn delete_all_messages_uses_the_expected_route_and_key() {
    let server = mock_server("204 No Content", "").await;
    delete_all_messages(&server.base_url, "secret-token")
        .await
        .unwrap();

    let request = server.request.await.unwrap();
    assert!(
        request.starts_with("DELETE /message "),
        "request: {request}"
    );
    assert!(
        request
            .to_lowercase()
            .contains("x-gotify-key: secret-token"),
        "request: {request}"
    );
}

#[tokio::test]
async fn delete_all_messages_surfaces_server_rejection() {
    let server = mock_server("403 Forbidden", "").await;
    let error = delete_all_messages(&server.base_url, "secret-token")
        .await
        .unwrap_err();
    assert_eq!(error, GotifyError::Unauthorized);
}

#[tokio::test]
async fn websocket_handshake_rejection_emits_disconnected_after_connecting() {
    // A 401 handshake response makes the WebSocket upgrade fail immediately.
    let server = mock_server("401 Unauthorized", "").await;
    let (events_tx, mut events_rx) = mpsc::channel(8);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let client = tokio::spawn(run_websocket_client(
        server.base_url.clone(),
        "wrong-token".into(),
        events_tx,
        shutdown_rx,
    ));

    assert!(matches!(
        events_rx.recv().await,
        Some(GotifyEvent::Connecting)
    ));
    assert!(matches!(
        events_rx.recv().await,
        Some(GotifyEvent::Disconnected { .. })
    ));

    let request = server.request.await.unwrap();
    assert!(
        request.starts_with("GET /stream?token=wrong-token "),
        "request: {request}"
    );

    shutdown_tx.send(true).unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(1), client)
        .await
        .expect("retry wait should be cancelled")
        .unwrap();
}

/// Minimal WebSocket server: completes the upgrade handshake, then streams the
/// supplied text frames before closing the connection.
async fn mock_websocket_server(frames: Vec<String>) -> MockServer {
    use base64::Engine;
    use sha1::{Digest, Sha1};

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address: SocketAddr = listener.local_addr().unwrap();
    let (request_tx, request_rx) = oneshot::channel();

    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buffer = vec![0u8; 4096];
        let mut request = String::new();
        loop {
            let read = socket.read(&mut buffer).await.unwrap();
            if read == 0 {
                break;
            }
            request.push_str(&String::from_utf8_lossy(&buffer[..read]));
            if request.contains("\r\n\r\n") {
                break;
            }
        }
        // Preserve the original value: the key is case-sensitive base64.
        let key = request
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                if name.eq_ignore_ascii_case("sec-websocket-key") {
                    Some(value.trim().to_string())
                } else {
                    None
                }
            })
            .expect("handshake must include Sec-WebSocket-Key");

        let mut hasher = Sha1::new();
        hasher.update(key.as_bytes());
        hasher.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
        let accept = base64::engine::general_purpose::STANDARD.encode(hasher.finalize());

        let handshake = format!(
            "HTTP/1.1 101 Switching Protocols\r\nupgrade: websocket\r\nconnection: upgrade\r\nsec-websocket-accept: {accept}\r\n\r\n"
        );
        socket.write_all(handshake.as_bytes()).await.unwrap();
        let _ = request_tx.send(request);

        for frame in frames {
            let payload = frame.into_bytes();
            let mut encoded = vec![0x81];
            if payload.len() < 126 {
                encoded.push(payload.len() as u8);
            } else {
                encoded.push(126);
                encoded.extend_from_slice(&(payload.len() as u16).to_be_bytes());
            }
            encoded.extend_from_slice(&payload);
            socket.write_all(&encoded).await.unwrap();
        }
        // Close frame so the client observes a clean disconnect and reconnects.
        socket.write_all(&[0x88, 0x00]).await.unwrap();
        let _ = socket.flush().await;
    });

    MockServer {
        base_url: format!("http://{address}"),
        request: request_rx,
    }
}

#[tokio::test]
async fn websocket_streams_messages_then_reports_disconnected() {
    let server = mock_websocket_server(vec![
        r#"{"id":7,"appid":2,"message":"hello","title":"greeting","priority":3,"date":"2026-01-01T00:00:00Z"}"#
            .to_string(),
    ])
    .await;

    let (events_tx, mut events_rx) = mpsc::channel(8);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let client = tokio::spawn(run_websocket_client(
        server.base_url.clone(),
        "secret-token".into(),
        events_tx,
        shutdown_rx,
    ));

    assert!(matches!(
        events_rx.recv().await,
        Some(GotifyEvent::Connecting)
    ));
    assert!(matches!(
        events_rx.recv().await,
        Some(GotifyEvent::Connected)
    ));
    match events_rx.recv().await {
        Some(GotifyEvent::Message(message)) => {
            assert_eq!(message.id, 7);
            assert_eq!(message.title, "greeting");
            assert_eq!(message.priority, 3);
        }
        other => panic!("expected decoded message, got {other:?}"),
    }
    assert!(matches!(
        events_rx.recv().await,
        Some(GotifyEvent::Disconnected { .. })
    ));

    let request = server.request.await.unwrap();
    assert!(
        request.starts_with("GET /stream?token=secret-token "),
        "request: {request}"
    );

    shutdown_tx.send(true).unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(1), client)
        .await
        .expect("retry wait should be cancelled")
        .unwrap();
}

#[tokio::test]
async fn stalled_handshake_times_out_with_an_actionable_reason() {
    use nexthrum_gotify::run_websocket_client_with_timeout;

    // Accept the TCP connection but never answer the WebSocket handshake.
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (socket, _) = listener.accept().await.unwrap();
        // Hold the connection open without writing a response.
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        drop(socket);
    });

    let (events_tx, mut events_rx) = mpsc::channel(8);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let client = tokio::spawn(run_websocket_client_with_timeout(
        format!("http://{address}"),
        "secret-token".into(),
        events_tx,
        shutdown_rx,
        std::time::Duration::from_millis(200),
    ));

    assert!(matches!(
        events_rx.recv().await,
        Some(GotifyEvent::Connecting)
    ));
    match events_rx.recv().await {
        Some(GotifyEvent::Disconnected { reason }) => {
            let reason = reason.expect("timeout must explain itself");
            assert!(
                reason.contains("did not complete the WebSocket handshake"),
                "unexpected reason: {reason}"
            );
        }
        other => panic!("expected a timeout disconnect, got {other:?}"),
    }

    shutdown_tx.send(true).unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(1), client)
        .await
        .expect("retry wait should be cancelled")
        .unwrap();
}

#[tokio::test]
async fn rejected_handshake_reason_names_the_status() {
    let server = mock_server("403 Forbidden", "").await;
    let (events_tx, mut events_rx) = mpsc::channel(8);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let client = tokio::spawn(run_websocket_client(
        server.base_url.clone(),
        "wrong-token".into(),
        events_tx,
        shutdown_rx,
    ));

    assert!(matches!(
        events_rx.recv().await,
        Some(GotifyEvent::Connecting)
    ));
    match events_rx.recv().await {
        Some(GotifyEvent::Disconnected { reason }) => {
            let reason = reason.expect("rejection must explain itself");
            assert!(reason.contains("token"), "unexpected reason: {reason}");
        }
        other => panic!("expected a rejected handshake, got {other:?}"),
    }

    shutdown_tx.send(true).unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(1), client)
        .await
        .expect("retry wait should be cancelled")
        .unwrap();
}

#[test]
fn crypto_provider_installation_is_idempotent() {
    // Hosts (and the Tauri app) may install their own provider first; calling
    // this repeatedly must never panic.
    nexthrum_gotify::ensure_crypto_provider();
    nexthrum_gotify::ensure_crypto_provider();
}

#[tokio::test]
async fn tls_websocket_urls_do_not_panic_without_host_setup() {
    // Regression guard: before `ensure_crypto_provider`, the first TLS handshake
    // panicked with "Could not automatically determine the process-level
    // CryptoProvider", leaving hosts stuck on "connecting" with no event.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (socket, _) = listener.accept().await.unwrap();
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        drop(socket);
    });

    let (events_tx, mut events_rx) = mpsc::channel(8);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let client = tokio::spawn(nexthrum_gotify::run_websocket_client_with_timeout(
        format!("https://{address}"),
        "secret-token".into(),
        events_tx,
        shutdown_rx,
        std::time::Duration::from_millis(300),
    ));

    assert!(matches!(
        events_rx.recv().await,
        Some(GotifyEvent::Connecting)
    ));
    // Either a TLS error or the handshake timeout is acceptable; a silent hang
    // (or a panicking worker) is not.
    match events_rx.recv().await {
        Some(GotifyEvent::Disconnected { reason }) => {
            assert!(reason.is_some(), "failure must explain itself");
        }
        other => panic!("expected a disconnect, got {other:?}"),
    }

    shutdown_tx.send(true).unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(1), client)
        .await
        .expect("retry wait should be cancelled")
        .unwrap();
}

#[tokio::test]
async fn list_applications_returns_names_for_filter_chips() {
    let server = mock_server(
        "200 OK",
        r#"[{"id":2,"name":"CI","token":"x"},{"id":4,"name":"家庭服务器"}]"#,
    )
    .await;
    let applications = nexthrum_gotify::list_applications(&server.base_url, "secret-token")
        .await
        .unwrap();

    assert_eq!(applications.len(), 2);
    assert_eq!(applications[0].id, 2);
    assert_eq!(applications[0].name, "CI");
    assert_eq!(applications[1].name, "家庭服务器");

    let request = server.request.await.unwrap();
    assert!(
        request.starts_with("GET /application "),
        "request: {request}"
    );
    assert!(request
        .to_lowercase()
        .contains("x-gotify-key: secret-token"));
}

#[tokio::test]
async fn list_applications_rejects_restricted_servers() {
    let server = mock_server("403 Forbidden", "").await;
    let error = nexthrum_gotify::list_applications(&server.base_url, "secret-token")
        .await
        .unwrap_err();
    assert_eq!(error, GotifyError::Unauthorized);
}

#[tokio::test]
async fn list_applications_reports_malformed_payloads() {
    let server = mock_server("200 OK", r#"{"id":2}"#).await;
    let error = nexthrum_gotify::list_applications(&server.base_url, "secret-token")
        .await
        .unwrap_err();
    assert!(matches!(error, GotifyError::InvalidResponse(_)));
}
