//! End-to-end check of the FFI stream path: settings -> start -> poll -> stop.
//!
//! Uses a minimal WebSocket server (handshake + one text frame) so the test does
//! not depend on a real Gotify instance. This mirrors what the Swift host does.

use base64::Engine;
use nexthrum_ffi::{BridgeService, BridgeSettings, ConnectionState};
use sha1::{Digest, Sha1};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::{Duration, Instant};

fn read_request(stream: &mut std::net::TcpStream) -> String {
    let mut buffer = [0u8; 4096];
    let mut request = String::new();
    loop {
        let read = stream.read(&mut buffer).expect("read handshake");
        if read == 0 {
            break;
        }
        request.push_str(&String::from_utf8_lossy(&buffer[..read]));
        if request.contains("\r\n\r\n") {
            break;
        }
    }
    request
}

fn websocket_frame(payload: &str) -> Vec<u8> {
    let bytes = payload.as_bytes();
    let mut frame = vec![0x81];
    if bytes.len() < 126 {
        frame.push(bytes.len() as u8);
    } else {
        frame.push(126);
        frame.extend_from_slice(&(bytes.len() as u16).to_be_bytes());
    }
    frame.extend_from_slice(bytes);
    frame
}

/// Serves exactly one WebSocket connection, then keeps it open long enough for
/// the client to read the frame.
fn spawn_websocket_server(payload: Option<String>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let request = read_request(&mut stream);
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
            .expect("handshake key");

        let mut hasher = Sha1::new();
        hasher.update(key.as_bytes());
        hasher.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
        let accept = base64::engine::general_purpose::STANDARD.encode(hasher.finalize());
        let response = format!(
            "HTTP/1.1 101 Switching Protocols\r\nupgrade: websocket\r\nconnection: upgrade\r\nsec-websocket-accept: {accept}\r\n\r\n"
        );
        stream.write_all(response.as_bytes()).unwrap();
        stream.flush().unwrap();
        if let Some(payload) = payload {
            stream.write_all(&websocket_frame(&payload)).unwrap();
            stream.flush().unwrap();
        }
        std::thread::sleep(Duration::from_secs(3));
    });
    format!("http://{address}")
}

fn settings(server_url: &str) -> BridgeSettings {
    BridgeSettings {
        server_url: server_url.to_string(),
        client_token: "secret-token".into(),
        auto_start: false,
        history_retention_days: 7,
        do_not_disturb_start: "22:00".into(),
        do_not_disturb_end: "08:00".into(),
    }
}

#[test]
fn bridge_reports_connected_and_delivers_a_message() {
    let root = tempfile::tempdir().unwrap();
    let server_url = spawn_websocket_server(Some(
        r#"{"id":11,"appid":2,"message":"body","title":"hello","priority":3,"date":"2026-01-01T00:00:00Z"}"#
            .into(),
    ));

    let service = BridgeService::new();
    service
        .set_storage_paths(
            root.path()
                .join("messages.sqlite")
                .to_string_lossy()
                .into_owned(),
            root.path()
                .join("settings.json")
                .to_string_lossy()
                .into_owned(),
        )
        .unwrap();
    service.set_settings(settings(&server_url)).unwrap();
    service.start_connection().unwrap();

    let deadline = Instant::now() + Duration::from_secs(5);
    let mut states = Vec::new();
    let mut saw_message = false;
    while Instant::now() < deadline {
        if let Some(event) = service.poll_event() {
            states.push(event.state);
            if let Some(message) = event.message {
                assert_eq!(message.id, 11);
                saw_message = true;
            }
            if saw_message {
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    service.stop_connection();

    assert!(
        states.contains(&ConnectionState::Connected),
        "bridge never reported Connected; states = {states:?}"
    );
    assert!(saw_message, "bridge never delivered the streamed message");
}

/// A server that accepts TCP but never answers the WebSocket handshake must not
/// leave the host waiting forever: the bridge has to report Disconnected.
#[test]
fn stalled_handshake_surfaces_as_disconnected() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        // Read the request, then stall without responding.
        let mut stream = stream;
        let _ = read_request(&mut stream);
        std::thread::sleep(Duration::from_secs(30));
    });

    let root = tempfile::tempdir().unwrap();
    let service = BridgeService::new();
    service
        .set_storage_paths(
            root.path()
                .join("messages.sqlite")
                .to_string_lossy()
                .into_owned(),
            root.path()
                .join("settings.json")
                .to_string_lossy()
                .into_owned(),
        )
        .unwrap();
    service
        .set_settings(settings(&format!("http://{address}")))
        .unwrap();
    service.start_connection().unwrap();

    let deadline = Instant::now() + Duration::from_secs(15);
    let mut saw_disconnect = None;
    while Instant::now() < deadline {
        if let Some(event) = service.poll_event() {
            if event.state == ConnectionState::Disconnected {
                saw_disconnect = event.error;
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    service.stop_connection();

    let reason =
        saw_disconnect.expect("bridge must report Disconnected after the handshake stalls");
    assert!(
        reason.contains("handshake"),
        "unexpected disconnect reason: {reason}"
    );
}

/// Same as above but over TLS: a server that accepts TCP and never answers the
/// TLS handshake must also time out instead of hanging.
#[test]
fn stalled_tls_handshake_surfaces_as_disconnected() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        // Read the ClientHello (TLS record) and stay silent.
        let mut buffer = [0u8; 4096];
        let _ = stream.read(&mut buffer);
        std::thread::sleep(Duration::from_secs(30));
    });

    let root = tempfile::tempdir().unwrap();
    let service = BridgeService::new();
    service
        .set_storage_paths(
            root.path()
                .join("messages.sqlite")
                .to_string_lossy()
                .into_owned(),
            root.path()
                .join("settings.json")
                .to_string_lossy()
                .into_owned(),
        )
        .unwrap();
    service
        .set_settings(settings(&format!("https://{address}")))
        .unwrap();
    service.start_connection().unwrap();

    let deadline = Instant::now() + Duration::from_secs(15);
    let mut disconnect_reason = None;
    while Instant::now() < deadline {
        if let Some(event) = service.poll_event() {
            if event.state == ConnectionState::Disconnected {
                disconnect_reason = event.error;
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    service.stop_connection();

    let reason = disconnect_reason.expect("TLS stall must surface as Disconnected");
    assert!(!reason.is_empty(), "disconnect must explain itself");
    println!("tls stall reason: {reason}");
}
