mod app_config;
mod bookshelf;
mod booksource;
mod errors;
mod extensions;
mod frontend_log;
mod frontend_storage;
mod platform;
mod prefetch;
mod state;

use tauri::Manager;

use crate::state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // 计算数据根目录并加载书架内存缓存
            let data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("无法解析 app_data_dir: {}", e))?;
            let app_state = AppState::load(data_dir)
                .map_err(|e| format!("AppState 初始化失败: {}", e))?;
            app.manage(app_state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            platform::get_platform,
            frontend_log::frontend_log,
            frontend_storage::frontend_storage_list_namespaces,
            frontend_storage::frontend_storage_list,
            frontend_storage::frontend_storage_set,
            frontend_storage::frontend_storage_remove,
            booksource::commands::booksource_get_dirs,
            booksource::commands::booksource_get_dir,
            booksource::commands::booksource_add_dir,
            booksource::commands::booksource_remove_dir,
            booksource::commands::booksource_list,
            extensions::commands::extension_get_dir,
            extensions::commands::extension_list,
            extensions::commands::extension_read,
            extensions::commands::extension_save,
            extensions::commands::extension_delete,
            extensions::commands::extension_toggle,
            extensions::commands::extension_open_in_vscode,
            app_config::app_config_get_all,
            app_config::app_config_set,
            app_config::app_config_reset,
            bookshelf::commands::bookshelf_list,
            bookshelf::commands::bookshelf_add,
            bookshelf::commands::bookshelf_save_chapters,
            bookshelf::commands::bookshelf_save_txt_chapters,
            bookshelf::commands::bookshelf_update_book,
            bookshelf::commands::bookshelf_get_chapters,
            bookshelf::commands::bookshelf_get_content,
            bookshelf::commands::bookshelf_save_content,
            bookshelf::commands::bookshelf_delete_content,
            bookshelf::commands::bookshelf_get_cached_indices,
            bookshelf::commands::bookshelf_get_episode_progress,
            bookshelf::commands::bookshelf_save_episode_progress,
            bookshelf::commands::bookshelf_get,
            bookshelf::commands::bookshelf_update_progress,
            bookshelf::commands::bookshelf_restore_source_switch,
            bookshelf::commands::bookshelf_export_book_data,
            bookshelf::commands::bookshelf_export_book,
            bookshelf::commands::bookshelf_reveal_data_dir,
            bookshelf::commands::bookshelf_reveal_export_file,
            prefetch::bookshelf_prefetch_chapters,
            prefetch::booksource_cancel,
            prefetch::bookshelf_pick_save_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}