mod app_config;
mod bookshelf;
mod booksource;
mod comic;
mod cover;
mod errors;
mod extensions;
mod font;
mod frontend_log;
mod frontend_storage;
mod platform;
mod prefetch;
mod state;
mod stubs;
mod sync;
mod video;
mod webserver;

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
            booksource::commands::booksource_list_streaming,
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
            booksource::commands::booksource_read,
            booksource::commands::booksource_save,
            booksource::commands::booksource_delete,
            booksource::commands::booksource_toggle,
            booksource::commands::booksource_resolve_path,
            booksource::commands::booksource_save_draft,
            booksource::commands::booksource_delete_draft,
            comic::comic_download_images,
            comic::comic_get_page_sizes,
            comic::comic_get_cached_page,
            comic::comic_cache_clear_chapter,
            comic::comic_cache_clear,
            comic::comic_cache_size,
            cover::cover_resolve_cache,
            cover::cover_cache_size,
            cover::cover_cache_clear,
            sync::sync_get_status,
            sync::sync_set_credentials,
            sync::sync_clear_credentials,
            sync::sync_get_credentials,
            sync::sync_test_connection,
            sync::sync_now,
            sync::sync_list_conflicts,
            sync::sync_resolve_conflict,
            sync::sync_client_state_set,
            sync::sync_report_reader_session,
            sync::sync_v2_sync_reading_progress,
            sync::sync_notify_lifecycle,
            sync::sync_baidu_token_status,
            sync::sync_baidu_start_auth,
            sync::sync_baidu_poll_token,
            sync::sync_baidu_revoke_auth,
            webserver::web_server_status,
            webserver::web_server_start,
            webserver::web_server_stop,
            webserver::web_server_pick_dist_dir,
            font::list_user_fonts,
            font::upload_user_font,
            font::delete_user_font,
            font::rename_user_font,
            font::list_system_fonts,
            video::start_video_proxy,
            video::stop_video_proxy,
            stubs::repository_fetch,
            stubs::repository_install,
            stubs::repository_preview_source,
            stubs::repository_check_source_sync,
            stubs::booksource_eval,
            stubs::booksource_search,
            stubs::booksource_book_info,
            stubs::booksource_chapter_list,
            stubs::booksource_chapter_content,
            stubs::booksource_explore,
            stubs::booksource_call_fn,
            stubs::booksource_http_proxy,
            stubs::booksource_check_update,
            stubs::booksource_apply_update,
            stubs::booksource_run_tests,
            stubs::booksource_test,
            stubs::booksource_compile_to_installed,
            stubs::booksource_analyze_url,
            stubs::booksource_pick_dir,
            stubs::config_read,
            stubs::config_write,
            stubs::config_write_json,
            stubs::config_delete_key,
            stubs::config_read_all,
            stubs::config_read_json,
            stubs::config_clear,
            stubs::config_read_bytes,
            stubs::config_write_bytes,
            stubs::config_list_scopes,
            stubs::config_dump_scope,
            stubs::js_eval,
            stubs::script_dialog_result,
            stubs::script_repl_eval,
            stubs::explore_clear_cache,
            stubs::tts_synthesize,
            stubs::tts_piper_synthesize,
            stubs::frontend_plugin_http_request,
            stubs::storage_debug_dump,
            stubs::get_local_ips,
            stubs::open_dir_in_explorer,
            stubs::export_save_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}