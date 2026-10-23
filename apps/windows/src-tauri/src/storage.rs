pub use nexthrum_storage::{cleanup_old_messages, get_unread_count, StoragePaths};
use std::path::PathBuf;

/// Resolve the existing platform location so migrations preserve current user data.
pub fn platform_default() -> Result<StoragePaths, String> {
    let root = platform_data_dir()
        .ok_or_else(|| "Could not resolve application data directory".to_string())?;
    let settings_dir = root.join("nexthrum_data");
    std::fs::create_dir_all(&settings_dir)
        .map_err(|e| format!("Failed to create settings directory: {e}"))?;
    Ok(StoragePaths::new(
        root.join("nexthrum_messages.db"),
        settings_dir.join("settings.json"),
    ))
}

#[cfg(target_os = "windows")]
fn platform_data_dir() -> Option<PathBuf> {
    std::env::var("APPDATA")
        .ok()
        .map(|p| PathBuf::from(p).join("Nexthrum"))
}

#[cfg(not(target_os = "windows"))]
fn platform_data_dir() -> Option<PathBuf> {
    dirs::data_dir()
}
