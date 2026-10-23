//! UniFFI bridge shared by the Windows host (via Tauri) and the macOS client.
//!
//! Only stable DTOs and the `BridgeService` facade cross the FFI boundary;
//! no Tauri, AppKit or storage types leak into the generated bindings.

mod connection;
#[cfg(test)]
#[path = "tests.rs"]
mod contract_tests;
mod dto;
mod events;
mod messages;
mod service;
mod settings;

pub use dto::{
    BridgeApplication, BridgeError, BridgeEvent, BridgeMessage, BridgeMessageSnapshot,
    BridgeSettings, ConnectionState, PreparedMessage,
};
pub use service::BridgeService;
pub use settings::validate_settings;

uniffi::setup_scaffolding!();
