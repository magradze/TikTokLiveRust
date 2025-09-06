// English comments for commits
use std::sync::Arc;
use log::{error, info, warn};

use crate::core::live_client_events::{TikTokLiveEvent, TikTokLiveEventObserver};
use crate::core::live_client_http::TikTokLiveHttpClient;
use crate::core::live_client_websocket::TikTokLiveWebsocketClient;
use crate::data::live_common::ConnectionState::{self, CONNECTING, DISCONNECTED};
use crate::data::live_common::{TikTokLiveInfo, TikTokLiveSettings};
use crate::errors::LibError;
use crate::http::http_data::LiveStatus::HostOnline;
use crate::http::http_data::{LiveConnectionDataRequest, LiveDataRequest, LiveUserDataRequest};

pub struct TikTokLiveClient {
    pub settings: TikTokLiveSettings,
    http_client: TikTokLiveHttpClient,
    event_observer: TikTokLiveEventObserver,
    websocket_client: Arc<TikTokLiveWebsocketClient>,
    room_info: TikTokLiveInfo,
}

impl TikTokLiveClient {
    pub(crate) fn new(
        settings: TikTokLiveSettings,
        http_client: TikTokLiveHttpClient,
        event_observer: TikTokLiveEventObserver,
        websocket_client: TikTokLiveWebsocketClient,
        room_info: TikTokLiveInfo,
    ) -> Self {
        TikTokLiveClient {
            settings,
            http_client,
            event_observer,
            websocket_client: Arc::new(websocket_client),
            room_info,
        }
    }

    pub async fn connect(mut self) -> Result<(), LibError> {
        if *self.room_info.connection_state.lock().unwrap() != DISCONNECTED {
            warn!("Client is already connected or connecting.");
            return Ok(());
        }
        self.set_connection_state(CONNECTING);

        info!("Fetching room ID for user '{}'...", &self.settings.host_name);
        let user_data = self.http_client.fetch_live_user_data(LiveUserDataRequest {
            user_name: self.settings.host_name.clone(),
        }).await?;

        info!("Fetching room info for room ID '{}'...", &user_data.room_id);
        let room_data = self.http_client.fetch_live_data(LiveDataRequest {
            room_id: user_data.room_id.clone(),
        }).await?;

        self.room_info.client_data = room_data.json;
        if room_data.live_status != HostOnline {
            error!("Host '{}' is not online. Status: {:?}", &self.settings.host_name, room_data.live_status);
            self.set_connection_state(DISCONNECTED);
            return Err(LibError::HostNotOnline);
        }

        info!("Fetching websocket connection details...");
        let connection_data = self.http_client.fetch_live_connection_data(LiveConnectionDataRequest {
            room_id: user_data.room_id.clone(),
        }).await?;

        // The client needs to be heap-allocated to be shared across threads.
        let client_arc = Arc::new(self);
        
        // Start the websocket client. It will manage its own lifecycle in spawned tasks.
        client_arc.websocket_client.start(connection_data, client_arc.clone()).await?;

        Ok(())
    }

    pub fn disconnect(&self) {
        info!("Disconnect requested by user.");
        self.websocket_client.stop();
        // The connection state will be set to DISCONNECTED by the websocket task itself upon exit.
    }

    pub fn publish_event(&self, event: TikTokLiveEvent) {
        self.event_observer.publish(self, event);
    }

    pub fn get_room_info(&self) -> &String {
        &self.room_info.client_data
    }

    pub fn set_connection_state(&self, state: ConnectionState) {
        let mut data = self.room_info.connection_state.lock().unwrap();
        *data = state;
        info!("Connection state changed to: {:?}", *data);
    }
}