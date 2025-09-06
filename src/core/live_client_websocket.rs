use futures_util::{SinkExt, StreamExt};
use log::info;
use prost::Message;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, timeout, Duration};
use tokio_tungstenite::tungstenite::handshake::client::Request;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage};

use crate::core::live_client::TikTokLiveClient;
use crate::core::live_client_mapper::TikTokLiveMessageMapper;
use crate::data::live_common::ConnectionState::{self, CONNECTED, DISCONNECTED};
use crate::errors::LibError;
use crate::core::live_client_events::TikTokLiveEvent;
use crate::http::http_data::LiveConnectionDataResponse;
use crate::generated::{ProtoMessageFetchResult, WebcastPushFrame};

pub struct TikTokLiveWebsocketClient {
    pub(crate) message_mapper: TikTokLiveMessageMapper,
    pub(crate) running: Arc<AtomicBool>,
}

impl TikTokLiveWebsocketClient {
    pub fn new(message_mapper: TikTokLiveMessageMapper) -> Self {
        TikTokLiveWebsocketClient {
            message_mapper,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn start(
        &self,
        response: LiveConnectionDataResponse,
        client: Arc<TikTokLiveClient>,
    ) -> Result<(), LibError> {
        let host = response
            .web_socket_url
            .host_str()
            .ok_or(LibError::InvalidHost)?;

        let request = Request::builder()
            .method("GET")
            .uri(response.web_socket_url.to_string())
            .header("Host", host)
            .header("Upgrade", "websocket")
            .header("Connection", "keep-alive")
            .header("Cache-Control", "max-age=0")
            .header("Accept", "text/html,application/json,application/protobuf")
            .header("Sec-Websocket-Key", "asd")
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.90 Safari/537.36")
            .header("Referer", "https://www.tiktok.com/")
            .header("Origin", "https://www.tiktok.com")
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Accept-Encoding", "gzip, deflate")
            .header("Cookie", response.web_socket_cookies)
            .header("Sec-Websocket-Version", "13")
            .body(())
            .map_err(|_| LibError::ParamsError)?;

        let (ws_stream, _) = connect_async(request)
            .await
            .map_err(|_| LibError::WebSocketConnectFailed)?;
        let (write, mut read) = ws_stream.split();
        let write = Arc::new(Mutex::new(write));

        client.set_connection_state(CONNECTED);
        client.publish_event(TikTokLiveEvent::OnConnected);

        let running = self.running.clone();
        running.store(true, Ordering::SeqCst);

        let message_mapper = self.message_mapper.clone();
        let client_clone = client.clone();
        let write_clone = write.clone();
        let running_clone = running.clone();

        tokio::spawn(async move {
            info!("Websocket connected");
            while running_clone.load(Ordering::SeqCst) {
                if let Some(Ok(message)) = read.next().await {
                    if let WsMessage::Binary(buffer) = message {
                        let push_frame = match WebcastPushFrame::decode(buffer.as_ref()) {
                            Ok(frame) => frame,
                            Err(_) => continue,
                        };

                        let proto_result = match ProtoMessageFetchResult::decode(push_frame.payload.as_ref()) {
                            Ok(result) => result,
                            Err(_) => continue,
                        };

                        if proto_result.needs_ack {
                            let push_frame_ack = WebcastPushFrame {
                                payload_type: "ack".to_string(),
                                log_id: push_frame.log_id,
                                payload: proto_result.internal_ext.clone().into_bytes(),
                                ..Default::default()
                            };
                            
                            // Using encode_to_vec() is a more robust way to get bytes from a prost message.
                            let binary = push_frame_ack.encode_to_vec();
                            let message = WsMessage::Binary(binary);
                            if write_clone.lock().await.send(message).await.is_err() {
                                continue;
                            }
                        }

                        // This was already correct in your code.
                        message_mapper.handle_webcast_response(proto_result, client_clone.as_ref());
                    }
                } else {
                    break; // Stream ended
                }
            }
            // Logic to handle disconnection
            running_clone.store(false, Ordering::SeqCst);
            client_clone.set_connection_state(DISCONNECTED);
            client_clone.publish_event(TikTokLiveEvent::OnDisconnected);
            info!("Websocket listener stopped.");
        });

        let write_clone = write.clone();
        let running_clone = running.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(9));
            while running_clone.load(Ordering::SeqCst) {
                interval.tick().await;

                let heartbeat_message = WsMessage::Binary(vec![0x3a, 0x02, 0x68, 0x62]);

                if timeout(Duration::from_secs(5), write_clone.lock().await.send(heartbeat_message)).await.is_err() {
                     log::error!("Heartbeat send timed out or failed");
                     break;
                }
            }
            log::info!("Heartbeat task stopped");
        });
        Ok(())
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}