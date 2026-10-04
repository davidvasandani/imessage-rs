/// Webhook dispatch service: fire-and-forget HTTP POST to registered webhooks.
///
/// When events occur (new message, typing, etc.), the webhook service POSTs
/// a JSON payload to each registered webhook URL whose event filter matches.
pub mod event_cache;
pub mod service;

use imessage_core::config::WebhookConfigEntry;

/// A webhook target: URL + event filter.
#[derive(Debug, Clone)]
pub struct WebhookTarget {
    pub url: String,
    /// Event names this webhook subscribes to (e.g. `["*"]` or `["new-message"]`).
    pub events: Vec<String>,
    /// Whether reaction messages (non-null `associatedMessageType`) are
    /// delivered. True by default, matching BlueBubbles.
    pub include_reactions: bool,
}

impl From<&WebhookConfigEntry> for WebhookTarget {
    fn from(entry: &WebhookConfigEntry) -> Self {
        match entry {
            WebhookConfigEntry::Simple(url) => Self {
                url: url.clone(),
                events: vec!["*".to_string()],
                include_reactions: true,
            },
            WebhookConfigEntry::Detailed {
                url,
                events,
                include_reactions,
            } => Self {
                url: url.clone(),
                events: events.clone().unwrap_or_else(|| vec!["*".to_string()]),
                include_reactions: include_reactions.unwrap_or(true),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_entry_subscribes_to_everything_including_reactions() {
        let t = WebhookTarget::from(&WebhookConfigEntry::Simple("http://a".into()));
        assert_eq!(t.events, vec!["*"]);
        assert!(t.include_reactions);
    }

    #[test]
    fn detailed_entry_defaults_and_overrides() {
        let t = WebhookTarget::from(&WebhookConfigEntry::Detailed {
            url: "http://b".into(),
            events: None,
            include_reactions: None,
        });
        assert_eq!(t.events, vec!["*"]);
        assert!(t.include_reactions);

        let t = WebhookTarget::from(&WebhookConfigEntry::Detailed {
            url: "http://c".into(),
            events: Some(vec!["new-message".into()]),
            include_reactions: Some(false),
        });
        assert_eq!(t.events, vec!["new-message"]);
        assert!(!t.include_reactions);
    }
}
