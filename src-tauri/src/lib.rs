mod downloader;
mod engine;
mod history;
mod models;
mod settings;
mod state;

use tauri::Manager;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            let handle = app.handle();
            let app_state = AppState {
                settings: std::sync::Mutex::new(settings::load(handle)),
                engine: std::sync::Mutex::new(engine::detect(handle)),
                history: std::sync::Mutex::new(history::load(handle)),
                next_id: std::sync::atomic::AtomicU64::new(1),
                ..Default::default()
            };
            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            engine::check_engine,
            engine::setup_engine,
            engine::update_engine,
            downloader::fetch_metadata,
            downloader::start_download,
            downloader::pause_download,
            downloader::resume_download,
            downloader::cancel_download,
            downloader::get_queue,
            downloader::clear_finished,
            history::get_history,
            history::remove_history_item,
            history::clear_history,
            settings::get_settings,
            settings::set_settings,
            settings::get_default_download_dir,
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                downloader::kill_all(app);
            }
        });
}
