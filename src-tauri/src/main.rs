// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use cortex_lib::{commands, state};
use log::info;
use state::AppState;
use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

const SPLASH_HTML: &str = include_str!("../splash.html");

#[tokio::main]
async fn main() {
    env_logger::init();
    info!("Starting Cortex...");

    tauri::Builder::default()
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
            // Export Commands (Phase 3)
            commands::export::export_vscode_context,
            commands::export::export_rake_package,
            commands::export::get_export_preview,
            commands::export::get_rake_export_preview,
            commands::export::list_prompt_templates,
            commands::export::get_export_stats,
        ])
        .setup(|app| {
            let handle = app.handle().clone();

            // Create splash screen window with inline HTML
            let html_content = format!(
                "data:text/html;base64,{}",
                base64::Engine::encode(&base64::engine::general_purpose::STANDARD, SPLASH_HTML)
            );

            let splash_window = WebviewWindowBuilder::new(
                app,
                "splashscreen",
                WebviewUrl::External(html_content.parse().unwrap())
            )
            .title("Cortex")
            .inner_size(500.0, 400.0)
            .center()
            .resizable(false)
            .decorations(false)
            .transparent(true)
            .always_on_top(true)
            .skip_taskbar(true)
            .build();

            if let Err(e) = splash_window {
                log::warn!("Failed to create splash window: {}", e);
            }

            // Spawn initialization in background
            tauri::async_runtime::spawn(async move {
                // Get windows
                let splash_window = handle.get_webview_window("splashscreen");
                let main_window = handle.get_webview_window("main");

                // Emit status update
                if let Some(splash) = &splash_window {
                    let _ = splash.emit("splash-status", "Initializing database...");
                }

                // Short delay to ensure splash is visible
                tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

                // Initialize app state
                if let Some(splash) = &splash_window {
                    let _ = splash.emit("splash-status", "Loading database...");
                }

                match AppState::new().await {
                    Ok(app_state) => {
                        info!("App state initialized successfully");

                        // Store app state
                        handle.manage(app_state);

                        if let Some(splash) = &splash_window {
                            let _ = splash.emit("splash-status", "Checking AI models...");
                        }
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                        if let Some(splash) = &splash_window {
                            let _ = splash.emit("splash-status", "Ready!");
                        }
                        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

                        // Close splash and show main window
                        if let Some(splash) = splash_window {
                            let _ = splash.close();
                        }

                        if let Some(main) = main_window {
                            let _ = main.show();
                            let _ = main.set_focus();
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to initialize app state: {}", e);

                        if let Some(splash) = &splash_window {
                            let _ = splash.emit("splash-status", "Initialization failed");
                        }

                        // Show error and exit
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                        std::process::exit(1);
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
