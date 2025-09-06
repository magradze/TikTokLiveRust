// English comments for commits
// This file is mostly correct and doesn't need many changes.
// It orchestrates the creation of the client's components.
use crate::core::live_client::TikTokLiveClient;
use crate::core::live_client_events::{TikTokEventHandler, TikTokLiveEventObserver};
use crate::core::live_client_http::TikTokLiveHttpClient;
use crate::core::live_client_mapper::TikTokLiveMessageMapper;
use crate::core::live_client_websocket::TikTokLiveWebsocketClient;
use crate::data::create_default_settings;
use crate::data::live_common::{TikTokLiveInfo, TikTokLiveSettings};
use crate::http::http_request_builder::HttpRequestFactory;

pub struct TikTokLiveBuilder {
    settings: TikTokLiveSettings,
    pub(crate) event_observer: TikTokLiveEventObserver,
}

impl TikTokLiveBuilder {
    /// Creates a new builder for a TikTok LIVE client.
    /// `user_name`: The TikTok user (`@username`) to connect to.
    pub fn new(user_name: &str) -> Self {
        Self {
            settings: create_default_settings(user_name),
            event_observer: TikTokLiveEventObserver::new(),
        }
    }

    /// Allows custom configuration of the client settings.
    pub fn configure<F>(&mut self, on_configure: F) -> &mut Self
    where
        F: FnOnce(&mut TikTokLiveSettings),
    {
        on_configure(&mut self.settings);
        self
    }

    /// Subscribes an event handler to be called for every TikTok LIVE event.
    pub fn on_event(&mut self, on_event: TikTokEventHandler) -> &mut Self {
        self.event_observer.subscribe(on_event);
        self
    }

    /// Builds the final `TikTokLiveClient` instance.
    pub fn build(&self) -> TikTokLiveClient {
        let settings = self.settings.clone();
        let observer = self.event_observer.clone();
        let mapper = TikTokLiveMessageMapper {};
        let websocket_client = TikTokLiveWebsocketClient::new(mapper);
        let http_factory = HttpRequestFactory {
            settings: settings.clone(),
        };
        let http_client = TikTokLiveHttpClient {
            settings: settings.clone(),
            factory: http_factory,
        };

        TikTokLiveClient::new(
            settings,
            http_client,
            observer,
            websocket_client,
            TikTokLiveInfo::default(),
        )
    }
}