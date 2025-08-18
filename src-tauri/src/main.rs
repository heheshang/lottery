// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use tauri::Manager;

mod lottery;
use lottery::api::LotteryAppState;

#[tokio::main]
async fn main() -> Result<()> {
    tauri::Builder::default()
        .manage(LotteryAppState::new())
        .invoke_handler(tauri::generate_handler![
            lottery::api::predict_numbers,
            lottery::api::train_algorithms,
            lottery::api::compare_algorithms,
            lottery::api::get_available_algorithms,
            lottery::api::get_algorithm_metadata,
            lottery::api::collect_and_update_data,
            lottery::api::get_recent_drawings,
            lottery::api::get_algorithm_rankings,
            lottery::api::recommend_algorithms,
        ])
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    
    Ok(())
}
