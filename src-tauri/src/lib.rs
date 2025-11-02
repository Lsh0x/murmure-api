mod audio;
pub mod config;
pub mod dictionary;
mod engine;
pub mod model;
pub mod transcription;
pub mod server;

// Server-only modules (always compiled)
// Tauri-specific modules (conditionally compiled)
#[cfg(feature = "tauri")]
mod clipboard;
#[cfg(feature = "tauri")]
mod commands;
#[cfg(feature = "tauri")]
mod history;
#[cfg(feature = "tauri")]
mod http_api;
#[cfg(feature = "tauri")]
mod overlay;
#[cfg(feature = "tauri")]
mod settings;
#[cfg(feature = "tauri")]
mod shortcuts;
#[cfg(feature = "tauri")]
mod tray_icon;

// Re-export public types for server usage
pub use config::ServerConfig;
pub use dictionary::Dictionary;
pub use model::Model;
pub use transcription::TranscriptionService;
pub use server::TranscriptionServiceImpl;

// Tauri-specific code is conditionally compiled
// Uncomment and add #[cfg(feature = "tauri")] when needed for desktop app
/*
#[cfg(feature = "tauri")]
fn show_main_window(app: &tauri::AppHandle) {
    // Tauri-specific window management
}

#[cfg(feature = "tauri")]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Tauri app initialization
}
*/
