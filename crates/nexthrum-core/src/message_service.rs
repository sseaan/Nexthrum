use crate::{message_policy, AppSettings, GotifyMessage, MessageListItem, MessageSnapshot};
use chrono::{DateTime, Utc};

/// Storage port required by the receive-message use case.
pub trait MessageRepository {
    fn save_message(
        &mut self,
        settings: &AppSettings,
        message: &GotifyMessage,
    ) -> Result<bool, String>;
    fn cleanup_expired(&mut self, settings: &AppSettings) -> Result<(), String>;
    fn unread_count(&self) -> Result<i64, String>;
    fn messages(&self) -> Result<Vec<MessageListItem>, String>;
    fn snapshot(&self) -> Result<MessageSnapshot, String>;
    fn mark_read(&mut self, message_id: i64) -> Result<(), String>;
    fn delete_message(&mut self, message_id: i64) -> Result<(), String>;
    fn clear_messages(&mut self) -> Result<(), String>;
}

pub fn get_message_snapshot<R: MessageRepository>(
    repository: &R,
) -> Result<MessageSnapshot, String> {
    repository.snapshot()
}

pub fn get_messages<R: MessageRepository>(repository: &R) -> Result<Vec<MessageListItem>, String> {
    repository.messages()
}

pub fn get_unread_count<R: MessageRepository>(repository: &R) -> Result<i64, String> {
    repository.unread_count()
}

pub fn mark_message_read<R: MessageRepository>(
    repository: &mut R,
    message_id: i64,
) -> Result<i64, String> {
    repository.mark_read(message_id)?;
    repository.unread_count()
}

pub fn delete_message<R: MessageRepository>(
    repository: &mut R,
    message_id: i64,
) -> Result<i64, String> {
    repository.delete_message(message_id)?;
    repository.unread_count()
}

pub fn clear_messages<R: MessageRepository>(repository: &mut R) -> Result<i64, String> {
    repository.clear_messages()?;
    repository.unread_count()
}

pub fn mark_message_read_snapshot<R: MessageRepository>(
    repository: &mut R,
    message_id: i64,
) -> Result<MessageSnapshot, String> {
    repository.mark_read(message_id)?;
    repository.snapshot()
}

pub fn delete_message_snapshot<R: MessageRepository>(
    repository: &mut R,
    message_id: i64,
) -> Result<MessageSnapshot, String> {
    repository.delete_message(message_id)?;
    repository.snapshot()
}

pub fn clear_messages_snapshot<R: MessageRepository>(
    repository: &mut R,
) -> Result<MessageSnapshot, String> {
    repository.clear_messages()?;
    repository.snapshot()
}

/// Result for a host adapter to publish and optionally notify.
pub struct IncomingMessageResult {
    pub message: GotifyMessage,
    pub unread_count: i64,
    pub should_notify: bool,
}

pub fn receive_message<R: MessageRepository>(
    repository: &mut R,
    settings: &AppSettings,
    message: GotifyMessage,
    received_at: DateTime<Utc>,
    local_minute_of_day: u32,
) -> Result<IncomingMessageResult, String> {
    let prepared = message_policy::prepare_incoming_message(
        message,
        settings,
        received_at,
        local_minute_of_day,
    );
    let is_new = repository.save_message(settings, &prepared.message)?;
    repository.cleanup_expired(settings)?;
    let unread_count = repository.unread_count()?;
    Ok(IncomingMessageResult {
        message: prepared.message,
        unread_count,
        should_notify: prepared.should_notify && is_new,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        clear_messages, clear_messages_snapshot, delete_message, delete_message_snapshot,
        get_message_snapshot, get_messages, get_unread_count, mark_message_read,
        mark_message_read_snapshot, receive_message, MessageRepository,
    };
    use crate::{AppSettings, GotifyMessage};
    use chrono::Utc;

    #[derive(Default)]
    struct FakeRepository {
        saved: Vec<GotifyMessage>,
        cleanup_called: bool,
        unread: i64,
        fail_save: bool,
        is_new: bool,
    }

    impl MessageRepository for FakeRepository {
        fn save_message(
            &mut self,
            _: &AppSettings,
            message: &GotifyMessage,
        ) -> Result<bool, String> {
            if self.fail_save {
                return Err("save failed".into());
            }
            self.saved.push(message.clone());
            Ok(self.is_new)
        }

        fn cleanup_expired(&mut self, _: &AppSettings) -> Result<(), String> {
            self.cleanup_called = true;
            Ok(())
        }

        fn unread_count(&self) -> Result<i64, String> {
            Ok(self.unread)
        }

        fn messages(&self) -> Result<Vec<crate::MessageListItem>, String> {
            Ok(self
                .saved
                .iter()
                .map(|message| crate::MessageListItem {
                    id: message.id,
                    appid: message.appid,
                    message: message.message.clone(),
                    title: message.title.clone(),
                    priority: message.priority,
                    date: message.date.clone(),
                    read: message.read,
                    received_at: message.received_at.clone(),
                })
                .collect())
        }

        fn snapshot(&self) -> Result<crate::MessageSnapshot, String> {
            Ok(crate::MessageSnapshot {
                messages: self.messages()?,
                unread_count: self.unread_count()?,
            })
        }

        fn mark_read(&mut self, _: i64) -> Result<(), String> {
            self.unread = self.unread.saturating_sub(1);
            Ok(())
        }

        fn delete_message(&mut self, _: i64) -> Result<(), String> {
            self.unread = self.unread.saturating_sub(1);
            Ok(())
        }

        fn clear_messages(&mut self) -> Result<(), String> {
            self.unread = 0;
            Ok(())
        }
    }

    fn message() -> GotifyMessage {
        GotifyMessage {
            id: 42,
            appid: 7,
            message: "payload".into(),
            title: "title".into(),
            priority: 1,
            date: "2026-07-03T12:00:00Z".into(),
            extras: None,
            read: false,
            received_at: String::new(),
        }
    }

    #[test]
    fn receive_use_case_persists_and_returns_notification_intent() {
        let settings = AppSettings::default();
        let mut repository = FakeRepository {
            unread: 3,
            is_new: true,
            ..Default::default()
        };
        let received_at = chrono::DateTime::parse_from_rfc3339("2026-07-03T12:30:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let result =
            receive_message(&mut repository, &settings, message(), received_at, 12 * 60).unwrap();
        assert_eq!(repository.saved.len(), 1);
        assert!(repository.cleanup_called);
        assert_eq!(result.unread_count, 3);
        assert!(result.should_notify);
        assert!(!result.message.read);
    }

    #[test]
    fn duplicate_message_is_persisted_without_repeating_notification() {
        let mut repository = FakeRepository::default();
        let result = receive_message(
            &mut repository,
            &AppSettings::default(),
            message(),
            Utc::now(),
            12 * 60,
        )
        .unwrap();
        assert_eq!(repository.saved.len(), 1);
        assert!(!result.should_notify);
    }

    #[test]
    fn query_use_cases_read_messages_and_unread_count_through_repository() {
        let repository = FakeRepository {
            saved: vec![message()],
            unread: 1,
            ..Default::default()
        };
        assert_eq!(get_messages(&repository).unwrap()[0].id, 42);
        assert_eq!(get_unread_count(&repository).unwrap(), 1);
        let snapshot = get_message_snapshot(&repository).unwrap();
        assert_eq!(snapshot.messages.len(), 1);
        assert_eq!(snapshot.unread_count, 1);
    }

    #[test]
    fn message_operations_return_the_post_mutation_unread_snapshot() {
        let mut repository = FakeRepository {
            unread: 3,
            ..Default::default()
        };
        assert_eq!(mark_message_read(&mut repository, 1).unwrap(), 2);
        assert_eq!(delete_message(&mut repository, 2).unwrap(), 1);
        assert_eq!(clear_messages(&mut repository).unwrap(), 0);
    }

    #[test]
    fn mutation_snapshot_use_cases_return_messages_and_unread_count_together() {
        let mut repository = FakeRepository {
            saved: vec![message()],
            unread: 1,
            ..Default::default()
        };
        let read = mark_message_read_snapshot(&mut repository, 42).unwrap();
        assert_eq!(read.messages.len(), 1);
        assert_eq!(read.unread_count, 0);

        repository.unread = 1;
        let deleted = delete_message_snapshot(&mut repository, 42).unwrap();
        assert_eq!(deleted.messages.len(), 1);
        assert_eq!(deleted.unread_count, 0);

        let cleared = clear_messages_snapshot(&mut repository).unwrap();
        assert_eq!(cleared.messages.len(), 1);
        assert_eq!(cleared.unread_count, 0);
    }

    #[test]
    fn receive_use_case_stops_when_persistence_fails() {
        let settings = AppSettings::default();
        let mut repository = FakeRepository {
            fail_save: true,
            ..Default::default()
        };
        let result = receive_message(&mut repository, &settings, message(), Utc::now(), 12 * 60);
        assert!(result.is_err());
        assert!(!repository.cleanup_called);
    }
}
