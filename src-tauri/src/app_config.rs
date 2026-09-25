use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::State;

use crate::errors::{CommandError, CommandResult};
use crate::state::AppState;

/// `<appDataDir>/app-config-overrides.json`
/// 只存用户**修改过**的键（不存完整 config，避免跟前端默认表双源数据）。
/// 读取时用 `AppConfig::default()` 兜底再 overlay 即可。
const OVERRIDES_FILE: &str = "app-config-overrides.json";

fn overrides_path(data_dir: &Path) -> PathBuf {
    data_dir.join(OVERRIDES_FILE)
}

fn read_overrides(data_dir: &Path) -> CommandResult<BTreeMap<String, serde_json::Value>> {
    let path = overrides_path(data_dir);
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    let raw = std::fs::read_to_string(&path)?;
    if raw.trim().is_empty() {
        return Ok(BTreeMap::new());
    }
    // 用户覆盖是松散结构（任意 key → 任意 JSON value），不用严格 schema
    let v: serde_json::Value = serde_json::from_str(&raw)?;
    let obj = v
        .as_object()
        .cloned()
        .ok_or_else(|| CommandError::other("app-config-overrides.json 顶层必须是对象"))?;
    Ok(obj.into_iter().collect())
}

fn write_overrides(
    data_dir: &Path,
    map: &BTreeMap<String, serde_json::Value>,
) -> CommandResult<()> {
    let path = overrides_path(data_dir);
    let tmp = data_dir.join(format!("{}.tmp", OVERRIDES_FILE));
    let json = serde_json::to_string_pretty(map)?;
    std::fs::write(&tmp, json)?;
    std::fs::rename(&tmp, &path)?;
    Ok(())
}

fn default_config_value() -> serde_json::Value {
    serde_json::to_value(AppConfig::default()).expect("AppConfig::default() 一定可序列化")
}

fn merge_with_defaults(
    overrides: &BTreeMap<String, serde_json::Value>,
) -> serde_json::Value {
    let mut base = default_config_value();
    if let Some(obj) = base.as_object_mut() {
        for (k, v) in overrides {
            // 用户存的就是它要的最终值；不校验类型，避免误把字符串 "true" 之类拒掉
            obj.insert(k.clone(), v.clone());
        }
    }
    base
}

// ── 与前端 AppConfig 对齐的默认值 ────────────────────────────────────────────
// 字段顺序、命名、默认值与 src/composables/useAppConfig.ts + src/stores/appConfig.ts
// 保持一致。前端用 snake_case，所以这里 #[serde(rename_all = "snake_case")]。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
struct AppConfig {
    http_user_agent: String,
    http_follow_redirects: bool,
    http_connect_timeout_secs: u32,
    http_ignore_tls_errors: bool,
    http_doh_server: String,
    proxy_mode: String,
    proxy_type: String,
    proxy_host: String,
    proxy_port: u32,
    proxy_username: String,
    proxy_password: String,
    engine_timeout_secs: u32,
    booksource_watcher_enabled: bool,
    browser_probe_enabled: bool,
    browser_probe_user_agent: String,
    browser_probe_timeout_secs: u32,
    browser_probe_visible_by_default: bool,
    browser_probe_force_visible: bool,
    browser_probe_persist_profile: bool,
    comic_cache_enabled: bool,
    ui_theme: String,
    ui_density: String,
    ui_enable_aplus_tracking: bool,
    video_player_type: String,
    video_default_rate: f32,
    video_auto_next: bool,
    video_quality_prefer: String,
    video_remember_progress: bool,
    video_seek_step_secs: u32,
    video_vjs_preload: String,
    video_vjs_pip: bool,
    video_xg_download: bool,
    video_dp_danmaku: bool,
    video_dp_theme: String,
    video_autoplay: bool,
    web_server_enabled: bool,
    web_server_port: u32,
    web_server_dist_path: String,
    request_min_delay_ms: u32,
    cache_prefetch_count: i32,
    cache_prefetch_concurrency: u32,
    export_prefetch_concurrency: u32,
    sync_enabled: bool,
    sync_provider: String,
    sync_profile_id: String,
    sync_webdav_url: String,
    sync_webdav_username: String,
    sync_webdav_root_dir: String,
    sync_webdav_allow_http: bool,
    sync_trigger_enabled: bool,
    sync_timer_enabled: bool,
    sync_timer_interval_secs: u32,
    sync_trigger_on_startup: bool,
    sync_trigger_on_resume: bool,
    sync_trigger_on_unlock_resume: bool,
    sync_trigger_on_bookshelf_change: bool,
    sync_trigger_on_booksource_change: bool,
    sync_trigger_on_settings_change: bool,
    sync_scope_bookshelf: bool,
    sync_scope_reading_progress: bool,
    sync_scope_booksources: bool,
    sync_scope_reader_settings: bool,
    sync_scope_app_settings: bool,
    sync_scope_source_flags: bool,
    sync_scope_extensions: bool,
    sync_scope_script_config: bool,
    sync_mobile_foreground_only: bool,
    sync_mobile_screen_on_only: bool,
    sync_mobile_wifi_only: bool,
    sync_mobile_pause_on_low_battery: bool,
    sync_mobile_startup_delay_ms: u32,
    sync_mobile_resume_delay_ms: u32,
    sync_baidu_app_name: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        // 与 src/stores/appConfig.ts:10-84 的 DEFAULT_CONFIG 保持一致
        // （这是 useAppConfigStore 用的，App.vue 启动加载走它）
        const BUILTIN_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
        Self {
            http_user_agent: BUILTIN_USER_AGENT.to_string(),
            http_follow_redirects: true,
            http_connect_timeout_secs: 10,
            http_ignore_tls_errors: true,
            http_doh_server: "none".to_string(),
            proxy_mode: "system".to_string(),
            proxy_type: "http".to_string(),
            proxy_host: String::new(),
            proxy_port: 0,
            proxy_username: String::new(),
            proxy_password: String::new(),
            engine_timeout_secs: 30,
            booksource_watcher_enabled: false,
            browser_probe_enabled: true,
            browser_probe_user_agent: String::new(),
            browser_probe_timeout_secs: 0,
            browser_probe_visible_by_default: false,
            browser_probe_force_visible: false,
            browser_probe_persist_profile: true,
            comic_cache_enabled: true,
            ui_theme: "auto".to_string(),
            ui_density: "standard".to_string(),
            ui_enable_aplus_tracking: true,
            video_player_type: "xgplayer".to_string(),
            video_default_rate: 1.0,
            video_auto_next: true,
            video_quality_prefer: "auto".to_string(),
            video_remember_progress: true,
            video_seek_step_secs: 10,
            video_vjs_preload: "auto".to_string(),
            video_vjs_pip: true,
            video_xg_download: false,
            video_dp_danmaku: false,
            video_dp_theme: "#00b1ff".to_string(),
            video_autoplay: true,
            web_server_enabled: false,
            web_server_port: 7688,
            web_server_dist_path: String::new(),
            request_min_delay_ms: 300,
            cache_prefetch_count: 3,
            cache_prefetch_concurrency: 2,
            export_prefetch_concurrency: 3,
            sync_enabled: false,
            sync_provider: "webdav".to_string(),
            sync_profile_id: "default".to_string(),
            sync_webdav_url: String::new(),
            sync_webdav_username: String::new(),
            sync_webdav_root_dir: "legado-sync".to_string(),
            sync_webdav_allow_http: false,
            sync_trigger_enabled: true,
            sync_timer_enabled: false,
            sync_timer_interval_secs: 900,
            sync_trigger_on_startup: true,
            sync_trigger_on_resume: true,
            sync_trigger_on_unlock_resume: true,
            sync_trigger_on_bookshelf_change: false,
            sync_trigger_on_booksource_change: false,
            sync_trigger_on_settings_change: false,
            sync_scope_bookshelf: true,
            sync_scope_reading_progress: true,
            sync_scope_booksources: true,
            sync_scope_reader_settings: true,
            sync_scope_app_settings: true,
            sync_scope_source_flags: false,
            sync_scope_extensions: false,
            sync_scope_script_config: false,
            sync_mobile_foreground_only: true,
            sync_mobile_screen_on_only: true,
            sync_mobile_wifi_only: true,
            sync_mobile_pause_on_low_battery: true,
            sync_mobile_startup_delay_ms: 5000,
            sync_mobile_resume_delay_ms: 1500,
            sync_baidu_app_name: "legado-tauri".to_string(),
        }
    }
}

// ── Tauri 命令 ─────────────────────────────────────────────────────────────

/// 返回完整配置：默认值 + 用户覆盖。
///
/// 序列化成普通 JSON 对象（`{ http_user_agent: "...", ... }`），前端直接当 AppConfig 用。
#[tauri::command(rename_all = "camelCase")]
pub fn app_config_get_all(state: State<'_, AppState>) -> CommandResult<serde_json::Value> {
    let overrides = read_overrides(&state.data_dir)?;
    Ok(merge_with_defaults(&overrides))
}

/// 设置单个 key；value 是任意 JSON 值（前端用 `unknown` 类型传过来）。
#[tauri::command(rename_all = "camelCase")]
pub fn app_config_set(
    state: State<'_, AppState>,
    key: String,
    value: serde_json::Value,
) -> CommandResult<()> {
    if key.is_empty() {
        return Err(CommandError::invalid("app_config_set 不允许空 key"));
    }
    let mut overrides = read_overrides(&state.data_dir)?;
    overrides.insert(key, value);
    write_overrides(&state.data_dir, &overrides)
}

/// 把单个 key 从用户覆盖中移除，下次 get_all 时该字段回退到默认值。
/// key 不存在视为成功（幂等）。
#[tauri::command(rename_all = "camelCase")]
pub fn app_config_reset(state: State<'_, AppState>, key: String) -> CommandResult<()> {
    let mut overrides = read_overrides(&state.data_dir)?;
    overrides.remove(&key);
    write_overrides(&state.data_dir, &overrides)
}