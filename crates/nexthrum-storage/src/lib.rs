use nexthrum_core::{AppSettings, GotifyMessage, MessageListItem, MessageSnapshot};
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};

/// Host-provided persistence locations. Keeping paths explicit makes the repository portable.
#[derive(Debug, Clone)]
pub struct StoragePaths {
    pub database_file: PathBuf,
    pub settings_file: PathBuf,
}

impl StoragePaths {
    /// Create explicit storage locations supplied by the platform host.
    pub fn new(database_file: PathBuf, settings_file: PathBuf) -> Self {
        Self {
            database_file,
            settings_file,
        }
    }

    pub fn ensure_parent(path: &Path) -> Result<(), String> {
        let parent = path
            .parent()
            .ok_or_else(|| "Storage path has no parent directory".to_string())?;
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create storage directory: {e}"))
    }
}

const SCHEMA_VERSION: i64 = 1;

fn open_db(paths: &StoragePaths) -> Result<Connection, String> {
    StoragePaths::ensure_parent(&paths.database_file)?;
    let mut conn = Connection::open(&paths.database_file)
        .map_err(|e| format!("Failed to open database: {e}"))?;
    migrate_schema(&mut conn)?;
    Ok(conn)
}

fn migrate_schema(conn: &mut Connection) -> Result<(), String> {
    let version: i64 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|e| format!("Failed to read database schema version: {e}"))?;

    if version > SCHEMA_VERSION {
        return Err(format!(
            "Database schema version {version} is newer than supported version {SCHEMA_VERSION}"
        ));
    }
    if version == SCHEMA_VERSION {
        return Ok(());
    }

    let transaction = conn
        .transaction()
        .map_err(|e| format!("Failed to start database migration: {e}"))?;
    if version < 1 {
        // Idempotent on legacy installations where these tables already exist but
        // PRAGMA user_version has not been set by an earlier application release.
        transaction
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS messages (
                    id INTEGER PRIMARY KEY,
                    appid INTEGER NOT NULL,
                    message TEXT NOT NULL,
                    title TEXT NOT NULL,
                    priority INTEGER NOT NULL DEFAULT 0,
                    date TEXT NOT NULL,
                    extras TEXT,
                    read INTEGER NOT NULL DEFAULT 0,
                    received_at TEXT NOT NULL DEFAULT (datetime('now'))
                );",
            )
            .map_err(|e| format!("Failed to apply database migration 1: {e}"))?;
        // Early schemas lack the local-only columns; reconcile before indexing.
        ensure_local_columns(&transaction)?;
        transaction
            .execute_batch(
                "CREATE INDEX IF NOT EXISTS idx_messages_date ON messages(date DESC);
                CREATE INDEX IF NOT EXISTS idx_messages_read ON messages(read);
                PRAGMA user_version = 1;",
            )
            .map_err(|e| format!("Failed to finish database migration 1: {e}"))?;
    }
    transaction
        .commit()
        .map_err(|e| format!("Failed to commit database migration: {e}"))
}

/// Add columns that older schemas may be missing. `ALTER TABLE` cannot use a
/// non-constant default, so `received_at` is backfilled from the server date to
/// keep retention cleanup from deleting legacy messages immediately.
fn ensure_local_columns(conn: &Connection) -> Result<(), String> {
    let mut stmt = conn
        .prepare("PRAGMA table_info(messages)")
        .map_err(|e| format!("Failed to inspect messages table: {e}"))?;
    let existing = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| format!("Failed to read messages columns: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read messages columns: {e}"))?;
    drop(stmt);

    let required = [
        ("priority", "INTEGER NOT NULL DEFAULT 0"),
        ("extras", "TEXT"),
        ("read", "INTEGER NOT NULL DEFAULT 0"),
        ("received_at", "TEXT NOT NULL DEFAULT ''"),
    ];
    for (name, definition) in required {
        if existing.iter().any(|column| column == name) {
            continue;
        }
        conn.execute_batch(&format!(
            "ALTER TABLE messages ADD COLUMN {name} {definition}"
        ))
        .map_err(|e| format!("Failed to add messages column {name}: {e}"))?;
    }

    conn.execute(
        "UPDATE messages SET received_at = date WHERE received_at = ''",
        [],
    )
    .map_err(|e| format!("Failed to backfill received_at: {e}"))?;
    Ok(())
}

pub fn save_message(paths: &StoragePaths, msg: &GotifyMessage) -> Result<bool, String> {
    let mut conn = open_db(paths)?;
    let extras = msg
        .extras
        .as_ref()
        .map(|v| v.to_string())
        .unwrap_or_default();
    let transaction = conn
        .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
        .map_err(|e| format!("Failed to start message transaction: {e}"))?;
    let inserted = transaction
        .execute(
            "INSERT OR IGNORE INTO messages (id, appid, message, title, priority, date, extras, read, received_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![msg.id, msg.appid, msg.message, msg.title, msg.priority, msg.date, extras, msg.read as i32, msg.received_at],
        )
        .map_err(|e| format!("Failed to insert message: {e}"))?;
    if inserted == 0 {
        transaction
            .execute(
                "UPDATE messages SET appid = ?2, message = ?3, title = ?4, priority = ?5,
                 date = ?6, extras = ?7 WHERE id = ?1",
                params![
                    msg.id,
                    msg.appid,
                    msg.message,
                    msg.title,
                    msg.priority,
                    msg.date,
                    extras
                ],
            )
            .map_err(|e| format!("Failed to update duplicate message: {e}"))?;
    }
    transaction
        .commit()
        .map_err(|e| format!("Failed to commit message transaction: {e}"))?;
    Ok(inserted == 1)
}

pub fn get_message_snapshot(paths: &StoragePaths) -> Result<MessageSnapshot, String> {
    let mut conn = open_db(paths)?;
    let transaction = conn
        .transaction()
        .map_err(|e| format!("Failed to start message snapshot: {e}"))?;
    let messages = {
        let mut stmt = transaction
            .prepare(
                "SELECT id, appid, message, title, priority, date, read, received_at \
                 FROM messages ORDER BY date DESC LIMIT 1000",
            )
            .map_err(|e| format!("Failed to prepare message snapshot: {e}"))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(MessageListItem {
                    id: row.get(0)?,
                    appid: row.get(1)?,
                    message: row.get(2)?,
                    title: row.get(3)?,
                    priority: row.get(4)?,
                    date: row.get(5)?,
                    read: row.get::<_, i32>(6)? != 0,
                    received_at: row.get(7)?,
                })
            })
            .map_err(|e| format!("Failed to query message snapshot: {e}"))?;
        rows.map(|row| row.map_err(|e| format!("Failed to read message snapshot row: {e}")))
            .collect::<Result<Vec<_>, _>>()?
    };
    let unread_count = transaction
        .query_row("SELECT COUNT(*) FROM messages WHERE read = 0", [], |row| {
            row.get(0)
        })
        .map_err(|e| format!("Failed to count unread messages: {e}"))?;
    transaction
        .commit()
        .map_err(|e| format!("Failed to finish message snapshot: {e}"))?;
    Ok(MessageSnapshot {
        messages,
        unread_count,
    })
}

/// Case-insensitive substring search over title and body.
///
/// `instr()` is used instead of `LIKE` so `%` and `_` in the query are literal.
/// The unread count is read from the same transaction and stays global.
pub fn search_messages(paths: &StoragePaths, query: &str) -> Result<MessageSnapshot, String> {
    let mut conn = open_db(paths)?;
    let transaction = conn
        .transaction()
        .map_err(|e| format!("Failed to start message search: {e}"))?;

    let messages = {
        let mut stmt = transaction
            .prepare(
                "SELECT id, appid, message, title, priority, date, read, received_at \
                 FROM messages \
                 WHERE instr(lower(title), lower(?1)) > 0 OR instr(lower(message), lower(?1)) > 0 \
                 ORDER BY date DESC LIMIT 500",
            )
            .map_err(|e| format!("Failed to prepare message search: {e}"))?;
        let rows = stmt
            .query_map(params![query], |row| {
                Ok(MessageListItem {
                    id: row.get(0)?,
                    appid: row.get(1)?,
                    message: row.get(2)?,
                    title: row.get(3)?,
                    priority: row.get(4)?,
                    date: row.get(5)?,
                    read: row.get::<_, i32>(6)? != 0,
                    received_at: row.get(7)?,
                })
            })
            .map_err(|e| format!("Failed to run message search: {e}"))?;
        rows.map(|row| row.map_err(|e| format!("Failed to read search row: {e}")))
            .collect::<Result<Vec<_>, _>>()?
    };

    let unread_count = transaction
        .query_row("SELECT COUNT(*) FROM messages WHERE read = 0", [], |row| {
            row.get(0)
        })
        .map_err(|e| format!("Failed to count unread messages: {e}"))?;
    transaction
        .commit()
        .map_err(|e| format!("Failed to finish message search: {e}"))?;
    Ok(MessageSnapshot {
        messages,
        unread_count,
    })
}

pub fn get_messages(paths: &StoragePaths) -> Result<Vec<MessageListItem>, String> {
    let conn = open_db(paths)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, appid, message, title, priority, date, read, received_at \
             FROM messages ORDER BY date DESC LIMIT 1000",
        )
        .map_err(|e| format!("Failed to prepare query: {e}"))?;
    let rows = stmt
        .query_map([], |row| {
            Ok(MessageListItem {
                id: row.get(0)?,
                appid: row.get(1)?,
                message: row.get(2)?,
                title: row.get(3)?,
                priority: row.get(4)?,
                date: row.get(5)?,
                read: row.get::<_, i32>(6)? != 0,
                received_at: row.get(7)?,
            })
        })
        .map_err(|e| format!("Failed to query messages: {e}"))?;
    rows.map(|row| row.map_err(|e| format!("Failed to read row: {e}")))
        .collect()
}

pub fn mark_read(paths: &StoragePaths, message_id: i64) -> Result<(), String> {
    let conn = open_db(paths)?;
    conn.execute(
        "UPDATE messages SET read = 1 WHERE id = ?1",
        params![message_id],
    )
    .map_err(|e| format!("Failed to mark read: {e}"))?;
    Ok(())
}

pub fn delete_message(paths: &StoragePaths, message_id: i64) -> Result<(), String> {
    let conn = open_db(paths)?;
    conn.execute("DELETE FROM messages WHERE id = ?1", params![message_id])
        .map_err(|e| format!("Failed to delete message: {e}"))?;
    Ok(())
}

pub fn clear_messages(paths: &StoragePaths) -> Result<(), String> {
    let conn = open_db(paths)?;
    conn.execute("DELETE FROM messages", [])
        .map_err(|e| format!("Failed to clear messages: {e}"))?;
    Ok(())
}

pub fn get_unread_count(paths: &StoragePaths) -> Result<i64, String> {
    let conn = open_db(paths)?;
    conn.query_row("SELECT COUNT(*) FROM messages WHERE read = 0", [], |row| {
        row.get(0)
    })
    .map_err(|e| format!("Failed to count unread: {e}"))
}

pub fn cleanup_old_messages(paths: &StoragePaths, settings: &AppSettings) -> Result<(), String> {
    let conn = open_db(paths)?;
    conn.execute(
        "DELETE FROM messages WHERE received_at < datetime('now', ?1)",
        params![format!("-{} days", settings.history_retention_days)],
    )
    .map_err(|e| format!("Failed to clean old messages: {e}"))?;
    Ok(())
}

pub fn save_settings(paths: &StoragePaths, settings: &AppSettings) -> Result<(), String> {
    StoragePaths::ensure_parent(&paths.settings_file)?;
    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {e}"))?;
    let temporary = paths
        .settings_file
        .with_extension(format!("json.tmp-{}", std::process::id()));
    std::fs::write(&temporary, json)
        .map_err(|e| format!("Failed to write temporary settings file: {e}"))?;
    if let Err(error) = std::fs::rename(&temporary, &paths.settings_file) {
        let _ = std::fs::remove_file(&temporary);
        return Err(format!("Failed to replace settings file: {error}"));
    }
    Ok(())
}

pub fn load_settings(paths: &StoragePaths) -> Option<AppSettings> {
    let json = std::fs::read_to_string(&paths.settings_file).ok()?;
    serde_json::from_str(&json).ok()
}

/// SQLite adapter implementing the Core message repository port.
pub struct SqliteMessageRepository {
    pub paths: StoragePaths,
}

impl nexthrum_core::message_service::MessageRepository for SqliteMessageRepository {
    fn save_message(&mut self, _: &AppSettings, message: &GotifyMessage) -> Result<bool, String> {
        save_message(&self.paths, message)
    }

    fn cleanup_expired(&mut self, settings: &AppSettings) -> Result<(), String> {
        cleanup_old_messages(&self.paths, settings)
    }

    fn unread_count(&self) -> Result<i64, String> {
        get_unread_count(&self.paths)
    }

    fn messages(&self) -> Result<Vec<MessageListItem>, String> {
        get_messages(&self.paths)
    }

    fn snapshot(&self) -> Result<MessageSnapshot, String> {
        get_message_snapshot(&self.paths)
    }

    fn search(&self, query: &str) -> Result<MessageSnapshot, String> {
        search_messages(&self.paths, query)
    }

    fn mark_read(&mut self, message_id: i64) -> Result<(), String> {
        mark_read(&self.paths, message_id)
    }

    fn delete_message(&mut self, message_id: i64) -> Result<(), String> {
        delete_message(&self.paths, message_id)
    }

    fn clear_messages(&mut self) -> Result<(), String> {
        clear_messages(&self.paths)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        cleanup_old_messages, clear_messages, delete_message, get_message_snapshot, get_messages,
        get_unread_count, load_settings, mark_read, migrate_schema, open_db, save_message,
        save_settings, search_messages, StoragePaths, SCHEMA_VERSION,
    };
    use crate::GotifyMessage;

    fn paths() -> StoragePaths {
        let dir = tempfile::tempdir().unwrap().keep();
        StoragePaths {
            database_file: dir.join("db/messages.sqlite"),
            settings_file: dir.join("prefs/settings.json"),
        }
    }

    fn message() -> GotifyMessage {
        GotifyMessage {
            id: 9,
            appid: 2,
            message: "hello".into(),
            title: "test".into(),
            priority: 1,
            date: "2026-01-01T00:00:00Z".into(),
            extras: None,
            read: false,
            received_at: "2026-01-01T00:00:00Z".into(),
        }
    }

    #[test]
    fn legacy_database_is_migrated_without_losing_messages() {
        let paths = paths();
        StoragePaths::ensure_parent(&paths.database_file).unwrap();
        let legacy = rusqlite::Connection::open(&paths.database_file).unwrap();
        legacy
            .execute_batch(
                "CREATE TABLE messages (
                id INTEGER PRIMARY KEY, appid INTEGER NOT NULL, message TEXT NOT NULL,
                title TEXT NOT NULL, priority INTEGER NOT NULL DEFAULT 0, date TEXT NOT NULL,
                extras TEXT, read INTEGER NOT NULL DEFAULT 0,
                received_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            INSERT INTO messages (id, appid, message, title, priority, date, read)
            VALUES (3, 1, 'legacy', 'legacy title', 0, '2025-01-01', 1);",
            )
            .unwrap();
        drop(legacy);

        let mut migrated = open_db(&paths).unwrap();
        migrate_schema(&mut migrated).unwrap();
        let version: i64 = migrated
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION);
        drop(migrated);

        let messages = get_messages(&paths).unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].id, 3);
        assert_eq!(messages[0].message, "legacy");
        assert!(messages[0].read);
    }

    #[test]
    fn duplicate_delivery_preserves_read_state_and_reports_existing_message() {
        let paths = paths();
        let incoming = message();
        assert!(save_message(&paths, &incoming).unwrap());
        mark_read(&paths, incoming.id).unwrap();

        let mut redelivered = incoming;
        redelivered.message = "updated payload".into();
        redelivered.read = false;
        redelivered.received_at = "2026-01-02T00:00:00Z".into();
        assert!(!save_message(&paths, &redelivered).unwrap());

        let stored = get_messages(&paths).unwrap().remove(0);
        assert_eq!(stored.message, "updated payload");
        assert!(
            stored.read,
            "redelivery must not mark an already-read message unread"
        );
    }

    #[test]
    fn message_snapshot_returns_list_and_unread_count_from_storage() {
        let paths = paths();
        let mut unread = message();
        unread.id = 10;
        save_message(&paths, &message()).unwrap();
        save_message(&paths, &unread).unwrap();
        mark_read(&paths, 9).unwrap();

        let snapshot = get_message_snapshot(&paths).unwrap();
        assert_eq!(snapshot.messages.len(), 2);
        assert_eq!(snapshot.unread_count, 1);
        assert!(snapshot.messages.iter().any(|item| item.id == 10));

        // Snapshot rows must expose server fields instead of placeholders.
        let stored = snapshot.messages.iter().find(|item| item.id == 9).unwrap();
        assert_eq!(stored.priority, 1);
        assert_eq!(stored.received_at, "2026-01-01T00:00:00Z");
    }

    #[test]
    fn search_matches_title_and_body_case_insensitively() {
        let paths = paths();
        let mut deploy = message();
        deploy.title = "Deploy finished".into();
        deploy.message = "production rollout done".into();
        let mut build = message();
        build.id = 11;
        build.title = "Build failed".into();
        build.message = "deploy script exited with 1".into();
        let mut unrelated = message();
        unrelated.id = 12;
        unrelated.title = "Nightly report".into();
        unrelated.message = "all good".into();
        save_message(&paths, &deploy).unwrap();
        save_message(&paths, &build).unwrap();
        save_message(&paths, &unrelated).unwrap();
        mark_read(&paths, 9).unwrap();

        let result = search_messages(&paths, "DEPLOY").unwrap();
        assert_eq!(result.messages.len(), 2, "title and body are both searched");
        assert_eq!(result.unread_count, 2, "unread count stays global");

        let by_title = search_messages(&paths, "nightly").unwrap();
        assert_eq!(by_title.messages.len(), 1);
        assert_eq!(by_title.messages[0].id, 12);

        assert!(search_messages(&paths, "absent")
            .unwrap()
            .messages
            .is_empty());
    }

    #[test]
    fn search_treats_wildcards_as_literal_characters() {
        let paths = paths();
        let mut plain = message();
        plain.title = "100% done".into();
        save_message(&paths, &plain).unwrap();
        let mut other = message();
        other.id = 20;
        other.title = "100 done".into();
        save_message(&paths, &other).unwrap();

        // `%` must not behave as a LIKE wildcard.
        let result = search_messages(&paths, "100%").unwrap();
        assert_eq!(result.messages.len(), 1);
        assert_eq!(result.messages[0].title, "100% done");
    }

    #[test]
    fn delete_and_clear_remove_only_local_messages() {
        let paths = paths();
        let mut second = message();
        second.id = 10;
        save_message(&paths, &message()).unwrap();
        save_message(&paths, &second).unwrap();
        delete_message(&paths, 9).unwrap();
        assert_eq!(get_messages(&paths).unwrap().len(), 1);
        clear_messages(&paths).unwrap();
        assert!(get_messages(&paths).unwrap().is_empty());
        assert_eq!(get_unread_count(&paths).unwrap(), 0);
    }

    #[test]
    fn retention_cleanup_removes_expired_messages() {
        let paths = paths();
        let mut old = message();
        old.received_at = "2020-01-01T00:00:00Z".into();
        save_message(&paths, &old).unwrap();
        let settings = nexthrum_core::AppSettings {
            history_retention_days: 7,
            ..Default::default()
        };
        cleanup_old_messages(&paths, &settings).unwrap();
        assert!(get_messages(&paths).unwrap().is_empty());
    }

    #[test]
    fn settings_round_trip_and_missing_fields_use_defaults() {
        let paths = paths();
        let settings = nexthrum_core::AppSettings {
            server_url: "https://example.test".into(),
            ..Default::default()
        };
        save_settings(&paths, &settings).unwrap();
        assert_eq!(
            load_settings(&paths).unwrap().server_url,
            "https://example.test"
        );

        std::fs::write(
            &paths.settings_file,
            r#"{"server_url":"https://legacy.test"}"#,
        )
        .unwrap();
        let migrated = load_settings(&paths).unwrap();
        assert_eq!(migrated.server_url, "https://legacy.test");
        assert_eq!(migrated.history_retention_days, 7);
        assert!(migrated.minimize_to_tray);
    }
    #[test]
    fn database_with_future_schema_version_is_rejected() {
        let paths = paths();
        StoragePaths::ensure_parent(&paths.database_file).unwrap();
        let conn = rusqlite::Connection::open(&paths.database_file).unwrap();
        conn.pragma_update(None, "user_version", SCHEMA_VERSION + 1)
            .unwrap();
        drop(conn);
        assert!(open_db(&paths)
            .unwrap_err()
            .contains("newer than supported"));
    }

    #[test]
    fn ancient_database_without_local_columns_is_upgraded() {
        let paths = paths();
        StoragePaths::ensure_parent(&paths.database_file).unwrap();
        let ancient = rusqlite::Connection::open(&paths.database_file).unwrap();
        ancient
            .execute_batch(
                "CREATE TABLE messages (
                    id INTEGER PRIMARY KEY, appid INTEGER NOT NULL, message TEXT NOT NULL,
                    title TEXT NOT NULL, date TEXT NOT NULL
                );
                INSERT INTO messages (id, appid, message, title, date)
                VALUES (5, 1, 'ancient', 'old title', '2026-01-02T03:04:05Z');",
            )
            .unwrap();
        drop(ancient);

        // Opens and migrates without losing the legacy row.
        let messages = get_messages(&paths).unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].id, 5);
        assert!(!messages[0].read);
        assert_eq!(get_unread_count(&paths).unwrap(), 1);

        // New writes now work against the upgraded schema.
        save_message(&paths, &message()).unwrap();

        let conn = rusqlite::Connection::open(&paths.database_file).unwrap();
        let received_at: String = conn
            .query_row("SELECT received_at FROM messages WHERE id = 5", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(
            received_at, "2026-01-02T03:04:05Z",
            "legacy rows must be backfilled from the server date"
        );
    }

    #[test]
    fn migration_is_idempotent_and_preserves_data() {
        let paths = paths();
        save_message(&paths, &message()).unwrap();
        for _ in 0..3 {
            let conn = open_db(&paths).unwrap();
            let version: i64 = conn
                .pragma_query_value(None, "user_version", |row| row.get(0))
                .unwrap();
            assert_eq!(version, SCHEMA_VERSION);
        }
        assert_eq!(get_messages(&paths).unwrap().len(), 1);
    }

    #[test]
    fn corrupt_database_reports_an_error_instead_of_panicking() {
        let paths = paths();
        StoragePaths::ensure_parent(&paths.database_file).unwrap();
        std::fs::write(&paths.database_file, b"this is not a sqlite database").unwrap();

        let error = get_message_snapshot(&paths).unwrap_err();
        assert!(!error.is_empty());
        assert!(save_message(&paths, &message()).is_err());
    }

    #[test]
    fn settings_replace_failure_surfaces_and_cleans_up_the_temp_file() {
        let paths = paths();
        // A directory at the settings path makes the atomic rename fail.
        std::fs::create_dir_all(&paths.settings_file).unwrap();
        let error = save_settings(&paths, &nexthrum_core::AppSettings::default()).unwrap_err();
        assert!(error.contains("settings"), "unexpected error: {error}");

        let parent = paths.settings_file.parent().unwrap();
        let leftovers = std::fs::read_dir(parent)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_name().to_string_lossy().contains("tmp-"))
            .count();
        assert_eq!(leftovers, 0, "temporary settings file must be removed");
    }

    #[test]
    fn invalid_settings_json_is_treated_as_missing_configuration() {
        let paths = paths();
        StoragePaths::ensure_parent(&paths.settings_file).unwrap();
        std::fs::write(&paths.settings_file, "not json").unwrap();
        assert!(load_settings(&paths).is_none());
    }

    #[test]
    fn repository_uses_injected_path_and_updates_read_state() {
        let paths = paths();
        save_message(&paths, &message()).unwrap();
        assert_eq!(get_unread_count(&paths).unwrap(), 1);
        mark_read(&paths, 9).unwrap();
        assert_eq!(get_unread_count(&paths).unwrap(), 0);
        assert_eq!(get_messages(&paths).unwrap()[0].id, 9);
    }
}
