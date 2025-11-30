// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use log::info;

mod commands;
mod db;
mod error;
mod indexer;
mod search;
mod state;

use state::AppState;

#[tokio::main]
async fn main() {
    env_logger::init();
    info!("Starting Cortex...");

    // Initialize app state
    let app_state = AppState::new().await.expect("Failed to initialize app state");

    tauri::Builder::default()
        .manage(app_state)
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::indexing::start_indexing,
            commands::indexing::stop_indexing,
            commands::indexing::get_index_status,
            commands::search::search_files,
            commands::search::get_file_detail,
            commands::search::get_search_stats,
            // AI Commands (Phase 2)
            commands::ai_commands::get_embedding_status,
            commands::ai_commands::generate_embeddings,
            commands::ai_commands::generate_all_embeddings,
            commands::ai_commands::semantic_search,
            commands::ai_commands::find_similar_files,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
