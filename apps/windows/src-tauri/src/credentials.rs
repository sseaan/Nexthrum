use crate::AppSettings;
use nexthrum_storage::StoragePaths;

#[cfg(windows)]
const CREDENTIAL_TARGET: &str = "Nexthrum/GotifyClientToken";

/// Minimal credential vault port so the migration/rollback logic is testable.
pub trait CredentialStore {
    fn read(&self) -> Result<Option<String>, String>;
    fn write(&self, secret: &str) -> Result<(), String>;
    fn delete(&self) -> Result<(), String>;
}

#[cfg(windows)]
mod platform {
    use super::{CredentialStore, CREDENTIAL_TARGET};
    use std::ptr;
    use windows_sys::Win32::Foundation::ERROR_NOT_FOUND;
    use windows_sys::Win32::Security::Credentials::{
        CredDeleteW, CredFree, CredReadW, CredWriteW, CREDENTIALW, CRED_PERSIST_LOCAL_MACHINE,
        CRED_TYPE_GENERIC,
    };

    pub struct PlatformVault;

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn last_error() -> u32 {
        unsafe { windows_sys::Win32::Foundation::GetLastError() }
    }

    impl CredentialStore for PlatformVault {
        fn read(&self) -> Result<Option<String>, String> {
            let target = wide(CREDENTIAL_TARGET);
            let mut credential = ptr::null_mut();
            let ok = unsafe { CredReadW(target.as_ptr(), CRED_TYPE_GENERIC, 0, &mut credential) };
            if ok == 0 {
                let error = last_error();
                if error == ERROR_NOT_FOUND {
                    return Ok(None);
                }
                return Err(format!("Windows Credential Manager read failed: {error}"));
            }

            let result = unsafe {
                let credential_ref = &*credential;
                if credential_ref.CredentialBlobSize == 0 || credential_ref.CredentialBlob.is_null()
                {
                    Ok(Some(String::new()))
                } else {
                    let bytes = std::slice::from_raw_parts(
                        credential_ref.CredentialBlob,
                        credential_ref.CredentialBlobSize as usize,
                    );
                    String::from_utf8(bytes.to_vec())
                        .map(Some)
                        .map_err(|_| "Windows Credential Manager token is not UTF-8".to_string())
                }
            };
            unsafe { CredFree(credential.cast()) };
            result
        }

        fn write(&self, secret: &str) -> Result<(), String> {
            let target = wide(CREDENTIAL_TARGET);
            let mut blob = secret.as_bytes().to_vec();
            let credential = CREDENTIALW {
                Flags: 0,
                Type: CRED_TYPE_GENERIC,
                TargetName: target.as_ptr() as *mut u16,
                Comment: ptr::null_mut(),
                LastWritten: windows_sys::Win32::Foundation::FILETIME {
                    dwLowDateTime: 0,
                    dwHighDateTime: 0,
                },
                CredentialBlobSize: blob.len() as u32,
                CredentialBlob: blob.as_mut_ptr(),
                Persist: CRED_PERSIST_LOCAL_MACHINE,
                AttributeCount: 0,
                Attributes: ptr::null_mut(),
                TargetAlias: ptr::null_mut(),
                UserName: ptr::null_mut(),
            };
            let ok = unsafe { CredWriteW(&credential, 0) };
            if ok == 0 {
                return Err(format!(
                    "Windows Credential Manager write failed: {}",
                    last_error()
                ));
            }
            Ok(())
        }

        fn delete(&self) -> Result<(), String> {
            let target = wide(CREDENTIAL_TARGET);
            let ok = unsafe { CredDeleteW(target.as_ptr(), CRED_TYPE_GENERIC, 0) };
            if ok == 0 {
                let error = last_error();
                if error == ERROR_NOT_FOUND {
                    return Ok(());
                }
                return Err(format!("Windows Credential Manager delete failed: {error}"));
            }
            Ok(())
        }
    }
}

#[cfg(not(windows))]
mod platform {
    use super::CredentialStore;

    /// Non-Windows hosts own their credential store (for example macOS Keychain).
    pub struct PlatformVault;

    impl CredentialStore for PlatformVault {
        fn read(&self) -> Result<Option<String>, String> {
            Ok(None)
        }

        fn write(&self, _: &str) -> Result<(), String> {
            Ok(())
        }

        fn delete(&self) -> Result<(), String> {
            Ok(())
        }
    }
}

pub use platform::PlatformVault;

/// Loads settings, migrating a legacy plaintext token into the platform vault.
///
/// Migration failures keep the plaintext value usable for this run instead of
/// discarding the user's configuration; the next launch retries the migration.
pub fn load_settings_with<S: CredentialStore>(
    paths: &StoragePaths,
    store: &S,
) -> Result<AppSettings, String> {
    let Some(mut settings) = nexthrum_storage::load_settings(paths) else {
        return Ok(AppSettings::default());
    };

    if settings.client_token.is_empty() {
        match store.read() {
            Ok(secret) => settings.client_token = secret.unwrap_or_default(),
            Err(error) => {
                // Keep the non-secret configuration so the user can re-enter the token.
                log::warn!("Could not read the stored Gotify token: {error}");
            }
        }
        return Ok(settings);
    }

    match store.write(&settings.client_token) {
        Ok(()) => {
            let token = settings.client_token.clone();
            let mut persisted = settings.clone();
            persisted.client_token.clear();
            nexthrum_storage::save_settings(paths, &persisted)
                .map_err(|e| format!("Failed to remove legacy plaintext token: {e}"))?;
            settings.client_token = token;
            Ok(settings)
        }
        Err(error) => {
            log::warn!("Keeping legacy plaintext token until migration succeeds: {error}");
            Ok(settings)
        }
    }
}

/// Persists non-secret settings as JSON and the token in the platform vault.
///
/// The previous vault value is restored when the settings file cannot be
/// written, so a failed save leaves both stores at their prior state.
pub fn save_settings_with<S: CredentialStore>(
    paths: &StoragePaths,
    store: &S,
    settings: &AppSettings,
) -> Result<(), String> {
    let previous = store.read()?;

    if settings.client_token.is_empty() {
        store.delete()?;
    } else {
        store.write(&settings.client_token)?;
    }

    let mut persisted = settings.clone();
    persisted.client_token.clear();
    if let Err(error) = nexthrum_storage::save_settings(paths, &persisted) {
        let rollback = match previous.as_deref() {
            Some(secret) if !secret.is_empty() => store.write(secret),
            _ => store.delete(),
        };
        return match rollback {
            Ok(()) => Err(error),
            Err(rollback_error) => Err(format!(
                "{error}; restoring the previous credential also failed: {rollback_error}"
            )),
        };
    }
    Ok(())
}

pub fn load_settings(paths: &StoragePaths) -> Result<AppSettings, String> {
    load_settings_with(paths, &PlatformVault)
}

pub fn save_settings(paths: &StoragePaths, settings: &AppSettings) -> Result<(), String> {
    save_settings_with(paths, &PlatformVault, settings)
}

#[cfg(test)]
mod tests {
    use super::{load_settings_with, save_settings_with, CredentialStore};
    use crate::AppSettings;
    use nexthrum_storage::StoragePaths;
    use std::cell::RefCell;

    #[derive(Default)]
    struct MemoryVault {
        secret: RefCell<Option<String>>,
        fail_write: RefCell<bool>,
        writes: RefCell<u32>,
    }

    impl CredentialStore for MemoryVault {
        fn read(&self) -> Result<Option<String>, String> {
            Ok(self.secret.borrow().clone())
        }

        fn write(&self, secret: &str) -> Result<(), String> {
            if *self.fail_write.borrow() {
                return Err("vault unavailable".into());
            }
            *self.writes.borrow_mut() += 1;
            *self.secret.borrow_mut() = Some(secret.to_string());
            Ok(())
        }

        fn delete(&self) -> Result<(), String> {
            *self.secret.borrow_mut() = None;
            Ok(())
        }
    }

    fn paths() -> StoragePaths {
        let dir = tempfile::tempdir().unwrap().keep();
        StoragePaths {
            database_file: dir.join("db/messages.sqlite"),
            settings_file: dir.join("prefs/settings.json"),
        }
    }

    fn settings() -> AppSettings {
        AppSettings {
            server_url: "https://gotify.example.test".into(),
            client_token: "secret".into(),
            ..Default::default()
        }
    }

    fn raw(paths: &StoragePaths) -> String {
        std::fs::read_to_string(&paths.settings_file).unwrap_or_default()
    }

    #[test]
    fn missing_settings_file_yields_defaults_without_touching_the_vault() {
        let vault = MemoryVault::default();
        let loaded = load_settings_with(&paths(), &vault).unwrap();
        assert_eq!(loaded.server_url, "");
        assert_eq!(loaded.client_token, "");
        assert_eq!(*vault.writes.borrow(), 0);
    }

    #[test]
    fn saving_moves_the_token_out_of_the_settings_file() {
        let paths = paths();
        let vault = MemoryVault::default();
        save_settings_with(&paths, &vault, &settings()).unwrap();

        assert_eq!(vault.read().unwrap().as_deref(), Some("secret"));
        let persisted = raw(&paths);
        assert!(!persisted.contains("secret"));
        assert!(persisted.contains("gotify.example.test"));
    }

    #[test]
    fn loading_migrates_a_legacy_plaintext_token_once() {
        let paths = paths();
        std::fs::create_dir_all(paths.settings_file.parent().unwrap()).unwrap();
        std::fs::write(
            &paths.settings_file,
            r#"{"server_url":"https://gotify.example.test","client_token":"legacy"}"#,
        )
        .unwrap();

        let vault = MemoryVault::default();
        let loaded = load_settings_with(&paths, &vault).unwrap();
        assert_eq!(loaded.client_token, "legacy");
        assert_eq!(vault.read().unwrap().as_deref(), Some("legacy"));
        assert!(!raw(&paths).contains("legacy"));

        let reloaded = load_settings_with(&paths, &vault).unwrap();
        assert_eq!(reloaded.client_token, "legacy");
        assert_eq!(*vault.writes.borrow(), 1);
    }

    #[test]
    fn failed_migration_keeps_settings_usable_until_retry() {
        let paths = paths();
        std::fs::create_dir_all(paths.settings_file.parent().unwrap()).unwrap();
        std::fs::write(
            &paths.settings_file,
            r#"{"server_url":"https://gotify.example.test","client_token":"legacy"}"#,
        )
        .unwrap();

        let vault = MemoryVault::default();
        *vault.fail_write.borrow_mut() = true;
        let loaded = load_settings_with(&paths, &vault).unwrap();
        assert_eq!(loaded.client_token, "legacy");
        assert_eq!(loaded.server_url, "https://gotify.example.test");

        *vault.fail_write.borrow_mut() = false;
        let migrated = load_settings_with(&paths, &vault).unwrap();
        assert_eq!(migrated.client_token, "legacy");
        assert!(!raw(&paths).contains("legacy"));
    }

    #[test]
    fn failed_settings_write_restores_the_previous_credential() {
        let paths = paths();
        let vault = MemoryVault::default();
        save_settings_with(&paths, &vault, &settings()).unwrap();

        // Replace the settings file with a directory so the atomic rename fails.
        std::fs::remove_file(&paths.settings_file).unwrap();
        std::fs::create_dir_all(&paths.settings_file).unwrap();

        let mut updated = settings();
        updated.client_token = "rotated".into();
        assert!(save_settings_with(&paths, &vault, &updated).is_err());
        assert_eq!(vault.read().unwrap().as_deref(), Some("secret"));
    }

    #[test]
    fn vault_read_failure_keeps_non_secret_settings() {
        struct BrokenVault;

        impl CredentialStore for BrokenVault {
            fn read(&self) -> Result<Option<String>, String> {
                Err("vault locked".into())
            }

            fn write(&self, _: &str) -> Result<(), String> {
                Err("vault locked".into())
            }

            fn delete(&self) -> Result<(), String> {
                Err("vault locked".into())
            }
        }

        let paths = paths();
        std::fs::create_dir_all(paths.settings_file.parent().unwrap()).unwrap();
        std::fs::write(
            &paths.settings_file,
            r#"{"server_url":"https://gotify.example.test"}"#,
        )
        .unwrap();

        let loaded = load_settings_with(&paths, &BrokenVault).unwrap();
        assert_eq!(loaded.server_url, "https://gotify.example.test");
        assert_eq!(loaded.client_token, "");
    }

    #[test]
    fn clearing_the_token_removes_the_stored_credential() {
        let paths = paths();
        let vault = MemoryVault::default();
        save_settings_with(&paths, &vault, &settings()).unwrap();

        let mut cleared = settings();
        cleared.client_token.clear();
        save_settings_with(&paths, &vault, &cleared).unwrap();
        assert_eq!(vault.read().unwrap(), None);
    }
}
