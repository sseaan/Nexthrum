use crate::{AppSettings, GotifyMessage};
use chrono::{DateTime, Timelike};

/// Result of applying local receive-time policy to a Gotify payload.
pub struct PreparedMessage {
    pub message: GotifyMessage,
    pub should_notify: bool,
}

/// Apply local-only receive metadata and notification policy.
///
/// This function is independent of Tauri, storage, and system notification APIs so its
/// behavior can be reused by other platform adapters.
pub fn prepare_incoming_message(
    mut message: GotifyMessage,
    settings: &AppSettings,
    received_at: DateTime<chrono::Utc>,
    local_minute_of_day: u32,
) -> PreparedMessage {
    message.received_at = received_at.to_rfc3339();
    message.read = false;

    PreparedMessage {
        message,
        should_notify: !is_in_dnd_period(settings, local_minute_of_day),
    }
}

/// Check whether a minute-of-day falls within the configured DND interval.
pub fn is_in_dnd_period(settings: &AppSettings, current_minutes: u32) -> bool {
    let start = parse_time_to_minutes(&settings.do_not_disturb_start);
    let end = parse_time_to_minutes(&settings.do_not_disturb_end);

    if start <= end {
        current_minutes >= start && current_minutes < end
    } else {
        current_minutes >= start || current_minutes < end
    }
}

fn parse_time_to_minutes(time: &str) -> u32 {
    let parts: Vec<&str> = time.split(':').collect();
    if parts.len() == 2 {
        if let (Ok(hours), Ok(minutes)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
            if hours < 24 && minutes < 60 {
                return hours * 60 + minutes;
            }
        }
    }
    0
}

pub fn local_minute_of_day(now: DateTime<chrono::Local>) -> u32 {
    now.hour() * 60 + now.minute()
}

#[cfg(test)]
mod tests {
    use super::{is_in_dnd_period, prepare_incoming_message};
    use crate::{AppSettings, GotifyMessage};

    fn settings(start: &str, end: &str) -> AppSettings {
        AppSettings {
            do_not_disturb_start: start.to_string(),
            do_not_disturb_end: end.to_string(),
            ..AppSettings::default()
        }
    }

    fn message() -> GotifyMessage {
        GotifyMessage {
            id: 42,
            appid: 7,
            message: "payload".into(),
            title: "title".into(),
            priority: 0,
            date: "2026-07-03T12:00:00Z".into(),
            extras: None,
            read: true,
            received_at: String::new(),
        }
    }

    #[test]
    fn dnd_interval_crossing_midnight_is_handled() {
        let config = settings("22:00", "08:00");
        assert!(is_in_dnd_period(&config, 22 * 60));
        assert!(is_in_dnd_period(&config, 7 * 60 + 59));
        assert!(!is_in_dnd_period(&config, 8 * 60));
        assert!(!is_in_dnd_period(&config, 12 * 60));
    }

    #[test]
    fn normal_dnd_interval_is_start_inclusive_and_end_exclusive() {
        let config = settings("09:30", "17:00");
        assert!(!is_in_dnd_period(&config, 9 * 60 + 29));
        assert!(is_in_dnd_period(&config, 9 * 60 + 30));
        assert!(!is_in_dnd_period(&config, 17 * 60));
    }

    #[test]
    fn preparing_message_sets_local_fields_and_notification_policy() {
        let config = settings("22:00", "08:00");
        let received = chrono::DateTime::parse_from_rfc3339("2026-07-03T12:30:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        let prepared = prepare_incoming_message(message(), &config, received, 12 * 60);
        assert!(!prepared.message.read);
        assert_eq!(prepared.message.received_at, "2026-07-03T12:30:00+00:00");
        assert!(prepared.should_notify);
    }
}
