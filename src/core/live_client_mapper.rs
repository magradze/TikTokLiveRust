// live_client_mapper.rs
use crate::core::live_client::TikTokLiveClient;
// Import our new, custom event enum.
use crate::core::live_client_events::TikTokLiveEvent;
// Import the prost Message trait, which gives us the `.decode()` method.
use prost::Message;

// Import all the necessary generated structs directly from our new module.
// We will need these to decode the binary payload of each message.
use crate::generated::{
    BaseProtoMessage, WebcastChatMessage, WebcastGiftMessage, WebcastLikeMessage,
    WebcastMemberMessage, ProtoMessageFetchResult,
};

#[derive(Clone)]
pub struct TikTokLiveMessageMapper {}

impl TikTokLiveMessageMapper {
    /// Handles the main response from TikTok's websocket, which can contain multiple messages.
    pub fn handle_webcast_response(
        &self,
        proto_result: ProtoMessageFetchResult,
        client: &TikTokLiveClient,
    ) {
        // Iterate through each individual message container within the response.
        for message in &proto_result.messages {
            // Pass each message to our new handler function.
            self.handle_single_message(message, client);
        }
    }

    /// Handles a single message from the response, decodes it, and publishes the corresponding event.
    fn handle_single_message(&self, message: &BaseProtoMessage, client: &TikTokLiveClient) {
        // The `r#type` field tells us what kind of event this is (e.g., "WebcastChatMessage").
        let msg_type = &message.r#type;

        // We use a `match` statement to handle only the events we care about.
        // For each event type, we attempt to decode the binary `payload` into the
        // corresponding prost-generated struct.
        match msg_type.as_str() {
            "WebcastChatMessage" => {
                if let Ok(msg) = WebcastChatMessage::decode(message.payload.as_ref()) {
                    client.publish_event(TikTokLiveEvent::OnChatMessage(msg));
                }
            }
            "WebcastGiftMessage" => {
                if let Ok(msg) = WebcastGiftMessage::decode(message.payload.as_ref()) {
                    client.publish_event(TikTokLiveEvent::OnGiftMessage(msg));
                }
            }
            "WebcastLikeMessage" => {
                if let Ok(msg) = WebcastLikeMessage::decode(message.payload.as_ref()) {
                    client.publish_event(TikTokLiveEvent::OnLikeMessage(msg));
                }
            }
            "WebcastMemberMessage" => {
                if let Ok(msg) = WebcastMemberMessage::decode(message.payload.as_ref()) {
                    client.publish_event(TikTokLiveEvent::OnMemberMessage(msg));
                }
            }
            // We ignore all other message types for now.
            _ => {}
        }
    }
}