// live_client_http.rs
// Replace the old `protobuf` import with the new `prost` import.
use prost::Message;

use crate::data::live_common::TikTokLiveSettings;
use crate::errors::LibError;
// Import the `WebcastResponse` struct directly from our new `generated` module.
// The old path `...::messages::webcast::` is no longer valid.
use crate::generated::ProtoMessageFetchResult;
use crate::http::http_data::{
    LiveConnectionDataRequest, LiveConnectionDataResponse, LiveDataRequest, LiveDataResponse,
    LiveUserDataRequest, LiveUserDataResponse,
};
use crate::http::http_data_mappers::{
    map_live_data_response, map_live_user_data_response, map_sign_server_response,
};
use crate::http::http_request_builder::HttpRequestFactory;

pub struct TikTokLiveHttpClient {
    pub(crate) settings: TikTokLiveSettings,
    pub(crate) factory: HttpRequestFactory,
}

pub const TIKTOK_URL_WEB: &str = "https://www.tiktok.com/";
pub const TIKTOK_URL_WEBCAST: &str = "https://webcast.tiktok.com/webcast/";
// This external signing API might be outdated or unavailable.
// For now, we leave it as is, but it's a potential point of failure.
pub const TIKTOK_SIGN_API: &str = "https://tiktok.eulerstream.com/webcast/sign_url";

impl TikTokLiveHttpClient {
    pub async fn fetch_live_user_data(
        &self,
        request: LiveUserDataRequest,
    ) -> Result<LiveUserDataResponse, LibError> {
        let url = format!("{}{}", TIKTOK_URL_WEB, "api-live/user/room");
        let option = self
            .factory
            .request()
            .with_url(url.as_str())
            .with_param("uniqueId", &request.user_name)
            .with_param("sourceType", "54")
            .as_json()
            .await;

        let json = option.ok_or(LibError::HttpRequestFailed)?;
        map_live_user_data_response(json)
    }

    pub async fn fetch_live_data(
        &self,
        request: LiveDataRequest,
    ) -> Result<LiveDataResponse, LibError> {
        let url = format!("{}{}", TIKTOK_URL_WEBCAST, "room/info");
        let option = self
            .factory
            .request()
            .with_url(url.as_str())
            .with_param("room_id", &request.room_id)
            .as_json()
            .await;

        let json = option.ok_or(LibError::HttpRequestFailed)?;
        map_live_data_response(json)
    }

    pub async fn fetch_live_connection_data(
        &self,
        request: LiveConnectionDataRequest,
    ) -> Result<LiveConnectionDataResponse, LibError> {
        // Preparing URL to sign
        let url_to_sign = self
            .factory
            .request()
            .with_url(&format!("{}{}", TIKTOK_URL_WEBCAST, "im/fetch"))
            .with_param("room_id", &request.room_id)
            .as_url();

        // Signing URL
        let option = self
            .factory
            .request()
            .with_url(TIKTOK_SIGN_API)
            .with_param("client", "ttlive-rust")
            .with_param("uuc", "1")
            .with_param("url", &url_to_sign)
            .with_param("apiKey", &self.settings.sign_api_key)
            .as_json()
            .await;

        let json = option.ok_or(LibError::UrlSigningFailed)?;
        let sign_server_response = map_sign_server_response(json);

        // Getting credentials for connection to websocket
        let response = self
            .factory
            .request()
            .with_reset()
            .with_time_out(self.settings.http_data.time_out)
            .with_url(&sign_server_response.signed_url)
            .build_get_request()
            .send()
            .await
            .map_err(|_| LibError::HttpRequestFailed)?;

        let optional_header = response.headers().get("set-cookie");
        let header_value = optional_header
            .ok_or(LibError::HeaderNotReceived)?
            .to_str()
            .map_err(|_| LibError::HeaderNotReceived)?
            .to_string();

        let protocol_buffer_message = response.bytes().await.map_err(|_| LibError::BytesParseError)?;

        // --- Refactoring Step: Use prost::Message::decode ---
        // Use ProtoMessageFetchResult instead of WebcastResponse
        let proto_result = ProtoMessageFetchResult::decode(protocol_buffer_message.as_ref())
            .map_err(|e| {
                eprintln!("Failed to decode ProtoMessageFetchResult: {:?}", e);
                LibError::BytesParseError
            })?;

        // ws_url არის საჭირო ველი
        let web_socket_url = proto_result.ws_url.clone();
        let url = url::Url::parse(&web_socket_url).map_err(|_| LibError::InvalidHost)?;
        Ok(LiveConnectionDataResponse {
            web_socket_timeout: self.settings.http_data.time_out,
            web_socket_cookies: header_value,
            web_socket_url: url,
        })
    }
}