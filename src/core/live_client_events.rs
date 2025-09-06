// live_client_events.rs
// English comments for the commit

use crate::core::live_client::TikTokLiveClient;
// Import the new, prost-generated structs.
// We import the specific message types we will handle.
// NOTE: The exact names (e.g., `WebcastChatMessage`, `WebcastGiftMessage`) come directly
// from the .proto files. We are effectively re-mapping the old event system to the new one.
use crate::generated::{
    WebcastChatMessage, WebcastGiftMessage, WebcastLikeMessage, WebcastMemberMessage,
};

// --- Refactoring Step 1: Create a new Event Enum ---
// This enum will be our new "main event type". It wraps the raw, prost-generated structs,
// providing a clean, unified interface for the rest of the application.
// This replaces the old, non-existent `TikTokLiveEvent`.
#[derive(Debug, Clone)]
pub enum TikTokLiveEvent {
    OnChatMessage(WebcastChatMessage),
    OnGiftMessage(WebcastGiftMessage),
    OnLikeMessage(WebcastLikeMessage),
    OnMemberMessage(WebcastMemberMessage),
    // We can add other events here later as we implement them.
    // e.g., OnSocialMessage(WebcastSocialMessage),
    OnConnected,
    OnDisconnected,
}

// --- Refactoring Step 2: Update the EventHandler Type Alias ---
// The type alias now uses our new `TikTokLiveEvent` enum.
pub type TikTokEventHandler =
    fn(client: &TikTokLiveClient, event: &TikTokLiveEvent);

// The observer struct itself doesn't need to change its structure.
#[derive(Clone)]
pub struct TikTokLiveEventObserver {
    pub events: Vec<TikTokEventHandler>,
}

impl TikTokLiveEventObserver {
    pub fn new() -> Self {
        TikTokLiveEventObserver { events: vec![] }
    }

    pub fn subscribe(&mut self, handler: TikTokEventHandler) {
        self.events.push(handler);
    }

    // --- Refactoring Step 3: Update the `publish` method signature ---
    // This method now accepts our new `TikTokLiveEvent` enum.
    pub fn publish(&self, client: &TikTokLiveClient, event: TikTokLiveEvent) {
        for handler in &self.events {
            // The logic inside remains the same, but the type is now correct.
            handler(client, &event);
        }
    }
}