mod ollama;
mod storage;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let directory = app.path().app_data_dir()?;
            std::fs::create_dir_all(&directory)?;
            app.manage(storage::DatabasePath(directory.join("stories.sqlite3")));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ollama::list_models,
            ollama::generate,
            storage::initialize_library,
            storage::list_stories,
            storage::load_story,
            storage::create_story,
            storage::save_story,
            storage::delete_story
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
