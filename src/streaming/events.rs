//! Event handling system for MessageStream.
//!
//! This module defines the event types and handlers used by the MessageStream
//! to manage different types of callbacks and event dispatching.

use crate::types::{AnthropicError, Message, MessageStreamEvent};

// Type aliases to simplify complex callback trait objects
// These aliases improve readability and satisfy clippy::type_complexity
#[allow(clippy::type_complexity)]
pub type StreamEventCb = dyn Fn(&MessageStreamEvent, &Message) + Send + Sync;
#[allow(clippy::type_complexity)]
pub type TextCb = dyn Fn(&str, &str) + Send + Sync;
#[allow(clippy::type_complexity)]
pub type MessageCb = dyn Fn(&Message) + Send + Sync;
#[allow(clippy::type_complexity)]
pub type ErrorCb = dyn Fn(&AnthropicError) + Send + Sync;
#[allow(clippy::type_complexity)]
pub type VoidCb = dyn Fn() + Send + Sync;

/// Types of events that can be handled by MessageStream.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EventType {
    /// Raw stream events with message snapshots
    StreamEvent,
    /// Text delta updates
    Text,
    /// Complete message received
    Message,
    /// Final message when stream ends
    FinalMessage,
    /// Error occurred
    Error,
    /// Stream ended
    End,
    /// Stream connected
    Connect,
    /// Stream aborted
    Abort,
}

/// Event handlers for different types of events.
///
/// This enum encapsulates the different callback types that can be registered
/// with MessageStream for handling various events during streaming.
pub enum EventHandler {
    /// Handler for stream events - receives event and current message snapshot
    StreamEvent(Box<StreamEventCb>),

    /// Handler for text deltas - receives delta text and current accumulated text
    Text(Box<TextCb>),

    /// Handler for complete messages
    Message(Box<MessageCb>),

    /// Handler for the final message when stream completes
    FinalMessage(Box<MessageCb>),

    /// Handler for errors
    Error(Box<ErrorCb>),

    /// Handler for stream end
    End(Box<VoidCb>),

    /// Handler for connection established
    Connect(Box<VoidCb>),

    /// Handler for stream abort
    Abort(Box<ErrorCb>),
}

impl std::fmt::Debug for EventHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StreamEvent(_) => f.debug_tuple("StreamEvent").field(&"<callback>").finish(),
            Self::Text(_) => f.debug_tuple("Text").field(&"<callback>").finish(),
            Self::Message(_) => f.debug_tuple("Message").field(&"<callback>").finish(),
            Self::FinalMessage(_) => f.debug_tuple("FinalMessage").field(&"<callback>").finish(),
            Self::Error(_) => f.debug_tuple("Error").field(&"<callback>").finish(),
            Self::End(_) => f.debug_tuple("End").field(&"<callback>").finish(),
            Self::Connect(_) => f.debug_tuple("Connect").field(&"<callback>").finish(),
            Self::Abort(_) => f.debug_tuple("Abort").field(&"<callback>").finish(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_event_type_equality() {
        assert_eq!(EventType::Text, EventType::Text);
        assert_ne!(EventType::Text, EventType::Error);
    }

    #[test]
    fn test_event_type_as_hash_key() {
        let mut map: HashMap<EventType, Vec<String>> = HashMap::new();
        map.insert(EventType::Text, vec!["handler1".to_string()]);
        map.insert(EventType::Error, vec!["handler2".to_string()]);

        assert!(map.contains_key(&EventType::Text));
        assert!(map.contains_key(&EventType::Error));
        assert!(!map.contains_key(&EventType::Connect));
    }

    #[test]
    fn test_event_handler_debug() {
        let handler = EventHandler::Text(Box::new(|_, _| {}));
        let debug_str = format!("{:?}", handler);
        assert!(debug_str.contains("Text"));
        assert!(debug_str.contains("<callback>"));
    }
}
