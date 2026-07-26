mod commands;
mod llm;
mod models;
mod storage;

use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            analyze_lyrics,
            save_track,
            list_tracks,
            get_track,
            delete_track,
            generate_lyrics,
            save_generation,
            list_generations,
            check_ollama_status,
            get_settings,
            save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
