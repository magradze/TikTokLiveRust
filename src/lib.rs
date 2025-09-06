
// Declare a 'generated' module.
pub mod generated {
    // === THIS IS THE FINAL FIX ===
    // We are now including the exact file that prost-build creates.
    // The name `tik_tok.rs` is derived from the proto package name with underscores.
    include!(concat!(env!("OUT_DIR"), "/tik_tok.rs"));
}

// Declare the other existing modules of the library.
pub mod core;
pub mod data;
pub mod errors;
pub mod http;

// ...existing code...

// The main struct for the library's public API.
pub struct TikTokLive {}

impl TikTokLive {
    /// Returns a builder for creating a new TikTokLiveClient.
    pub fn new_client(user_name: &str) -> core::live_client_builder::TikTokLiveBuilder {
        core::live_client_builder::TikTokLiveBuilder::new(user_name)
    }
}