//! 引擎依赖型命令的统一 stub。
//!
//! 这些命令前端调用会 throw "Command not found" 之前；本模块把所有 stub
//! 注册成具名函数，统一返回 "xxx 暂未实现：需要 yyy" 的明确错误，
//! 至少消掉 unhandledrejection。真正实现等 booksource JS 引擎、HTTP
//! 客户端、TTS 引擎、扩展 HTTP 等基础组件就绪后逐项替换。

use serde::Deserialize;
use tauri::{AppHandle, State};

use crate::errors::{CommandError, CommandResult};
use crate::state::AppState;

fn not_implemented(what: &str, needs: &str) -> CommandError {
    CommandError::other(format!("{} 暂未实现：{}", what, needs))
}

// ── 书源仓库（依赖网络 + JS 引擎） ──────────────────────────────────────────

#[tauri::command(rename_all = "camelCase")]
pub fn repository_fetch(_url: String) -> CommandResult<serde_json::Value> {
    Err(not_implemented(
        "repository_fetch",
        "需要 HTTP 客户端 + 书源 JSON 协议解析",
    ))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryInstallArgs {
    #[serde(default)]
    pub download_url: Option<String>,
    #[serde(default)]
    pub file_name: Option<String>,
    #[serde(default)]
    pub expected_uuid: Option<String>,
}

#[tauri::command(rename_all = "camelCase")]
pub fn repository_install(_args: RepositoryInstallArgs) -> CommandResult<()> {
    Err(not_implemented(
        "repository_install",
        "需要 HTTP 下载 + 书源安装逻辑",
    ))
}

#[tauri::command(rename_all = "camelCase")]
pub fn repository_preview_source(
) -> CommandResult<serde_json::Value> {
    Err(not_implemented(
        "repository_preview_source",
        "需要 HTTP 下载 + JS 引擎解析",
    ))
}

#[tauri::command(rename_all = "camelCase")]
pub fn repository_check_source_sync(
) -> CommandResult<serde_json::Value> {
    Err(not_implemented(
        "repository_check_source_sync",
        "需要 JS 引擎对比本地 / 远程书源",
    ))
}

// ── 书源脚本执行（依赖 JS 引擎） ──────────────────────────────────────────

#[tauri::command(rename_all = "camelCase")]
pub fn booksource_eval(
) -> CommandResult<String> {
    Err(not_implemented(
        "booksource_eval",
        "需要 Boa / QuickJS 等 JS 引擎嵌入",
    ))
}

#[tauri::command(rename_all = "camelCase")]
pub fn booksource_search(
) -> CommandResult<serde_json::Value> {
    Err(not_implemented("booksource_search", "需要 JS 引擎 + HTTP"))
}

#[tauri::command(rename_all = "camelCase")]
pub fn booksource_book_info(
) -> CommandResult<serde_json::Value> {
    Err(not_implemented("booksource_book_info", "需要 JS 引擎 + HTTP"))
}

#[tauri::command(rename_all = "camelCase")]
pub fn booksource_chapter_list(
) -> CommandResult<serde_json::Value> {
    Err(not_implemented("booksource_chapter_list", "需要 JS 引擎 + HTTP"))
}

#[tauri::command(rename_all = "camelCase")]
pub fn booksource_chapter_content(
) -> CommandResult<String> {
    Err(not_implemented(
        "booksource_chapter_content",
        "需要 JS 引擎 + HTTP",
    ))
}

#[tauri::command(rename_all = "camelCase")]
pub fn booksource_explore(
) -> CommandResult<serde_json::Value> {
    Err(not_implemented("booksource_explore", "需要 JS 引擎 + HTTP"))
}

#[tauri::command(rename_all = "camelCase")]
pub fn booksource_call_fn(
) -> CommandResult<serde_json::Value> {
    Err(not_implemented("booksource_call_fn", "需要 JS 引擎"))
}

#[tauri::command(rename_all = "camelCase")]
pub fn booksource_http_proxy(
) -> CommandResult<serde_json::Value> {
    Err(not_implemented("booksource_http_proxy", "需要 HTTP 客户端"))
}

#[tauri::command(rename_all = "camelCase")]
pub fn booksource_check_update(
) -> CommandResult<serde_json::Value> {
    Err(not_implemented(
        "booksource_check_update",
        "需要 JS 引擎 + HTTP",
    ))
}

#[tauri::command(rename_all = "camelCase")]
pub fn booksource_apply_update(
) -> CommandResult<()> {
    Err(not_implemented(
        "booksource_apply_update",
        "需要 JS 引擎 + HTTP",
    ))
}

#[tauri::command(rename_all = "camelCase")]
pub fn booksource_run_tests(
) -> CommandResult<serde_json::Value> {
    Err(not_implemented("booksource_run_tests", "需要 JS 引擎"))
}

#[tauri::command(rename_all = "camelCase")]
pub fn booksource_test(
) -> CommandResult<serde_json::Value> {
    Err(not_implemented("booksource_test", "需要 JS 引擎"))
}

#[tauri::command(rename_all = "camelCase")]
pub fn booksource_compile_to_installed(
) -> CommandResult<()> {
    Err(not_implemented(
        "booksource_compile_to_installed",
        "需要 JS 引擎",
    ))
}

#[tauri::command(rename_all = "camelCase")]
pub fn booksource_analyze_url(
) -> CommandResult<serde_json::Value> {
    Err(not_implemented(
        "booksource_analyze_url",
        "需要 HTTP 抓取 + JS 引擎",
    ))
}

#[tauri::command(rename_all = "camelCase")]
pub fn booksource_pick_dir() -> CommandResult<String> {
    Err(not_implemented(
        "booksource_pick_dir",
        "桌面端走 Tauri dialog plugin 直连；Harmony 选择器待补",
    ))
}

// ── 脚本配置 / 调试 ──────────────────────────────────────────────────────────

#[tauri::command(rename_all = "camelCase")]
pub fn config_read(
) -> CommandResult<String> {
    Err(not_implemented("config_read", "脚本 KV 待补"))
}

#[tauri::command(rename_all = "camelCase")]
pub fn config_write(
) -> CommandResult<()> {
    Err(not_implemented("config_write", "脚本 KV 待补"))
}

#[tauri::command(rename_all = "camelCase")]
pub fn config_write_json(
) -> CommandResult<()> {
    Err(not_implemented("config_write_json", "脚本 KV 待补"))
}

#[tauri::command(rename_all = "camelCase")]
pub fn config_delete_key(
) -> CommandResult<()> {
    Err(not_implemented("config_delete_key", "脚本 KV 待补"))
}

#[tauri::command(rename_all = "camelCase")]
pub fn config_read_all(
) -> CommandResult<serde_json::Value> {
    Err(not_implemented("config_read_all", "脚本 KV 待补"))
}

#[tauri::command(rename_all = "camelCase")]
pub fn config_read_json(
) -> CommandResult<serde_json::Value> {
    Err(not_implemented("config_read_json", "脚本 KV 待补"))
}

#[tauri::command(rename_all = "camelCase")]
pub fn config_clear(_scope: String) -> CommandResult<()> {
    Err(not_implemented("config_clear", "脚本 KV 待补"))
}

#[tauri::command(rename_all = "camelCase")]
pub fn config_read_bytes(
) -> CommandResult<Vec<u8>> {
    Err(not_implemented("config_read_bytes", "脚本 KV 待补"))
}

#[tauri::command(rename_all = "camelCase")]
pub fn config_write_bytes(
) -> CommandResult<()> {
    Err(not_implemented("config_write_bytes", "脚本 KV 待补"))
}

#[tauri::command(rename_all = "camelCase")]
pub fn config_list_scopes() -> CommandResult<Vec<String>> {
    Err(not_implemented("config_list_scopes", "脚本 KV 待补"))
}

#[tauri::command(rename_all = "camelCase")]
pub fn config_dump_scope(
) -> CommandResult<serde_json::Value> {
    Err(not_implemented("config_dump_scope", "脚本 KV 待补"))
}

// ── JS eval / 调试 ──────────────────────────────────────────────────────────

#[tauri::command(rename_all = "camelCase")]
pub fn js_eval(_code: String) -> CommandResult<String> {
    Err(not_implemented("js_eval", "需要 JS 引擎"))
}

#[tauri::command(rename_all = "camelCase")]
pub fn script_dialog_result(
) -> CommandResult<()> {
    Err(not_implemented("script_dialog_result", "需要 JS 引擎事件循环"))
}

#[tauri::command(rename_all = "camelCase")]
pub fn script_repl_eval(
) -> CommandResult<serde_json::Value> {
    Err(not_implemented("script_repl_eval", "需要 JS 引擎"))
}

#[tauri::command(rename_all = "camelCase")]
pub fn explore_clear_cache(
) -> CommandResult<()> {
    Ok(())
}

// ── TTS ──────────────────────────────────────────────────────────────────────

#[tauri::command(rename_all = "camelCase")]
pub fn tts_synthesize(
) -> CommandResult<serde_json::Value> {
    Err(not_implemented("tts_synthesize", "需要 TTS 引擎（如 sherpa-onnx）"))
}

#[tauri::command(rename_all = "camelCase")]
pub fn tts_piper_synthesize(
) -> CommandResult<serde_json::Value> {
    Err(not_implemented("tts_piper_synthesize", "需要 piper-tts 引擎"))
}

// ── 前端插件 HTTP 请求（沙箱内代理 fetch） ──────────────────────────────────

#[tauri::command(rename_all = "camelCase")]
pub fn frontend_plugin_http_request(
) -> CommandResult<serde_json::Value> {
    Err(not_implemented(
        "frontend_plugin_http_request",
        "需要 HTTP 客户端 + 沙箱",
    ))
}

// ── 调试存储转储 ────────────────────────────────────────────────────────────

#[tauri::command(rename_all = "camelCase")]
pub fn storage_debug_dump(state: State<'_, AppState>) -> CommandResult<serde_json::Value> {
    // 粗略 dump：frontend = frontend_storage 下所有命名空间的扁平 map
    let storage_dir = state.data_dir.join("frontend_storage");
    let mut frontend = serde_json::Map::new();
    if storage_dir.exists() {
        for entry in std::fs::read_dir(&storage_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let ns = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string();
            let raw = std::fs::read_to_string(&path)?;
            let map: serde_json::Value = if raw.trim().is_empty() {
                serde_json::json!({})
            } else {
                serde_json::from_str(&raw).unwrap_or(serde_json::json!({}))
            };
            frontend.insert(ns, map);
        }
    }
    let overrides_path = state.data_dir.join("app-config-overrides.json");
    let app_config = if overrides_path.exists() {
        let raw = std::fs::read_to_string(&overrides_path)?;
        serde_json::from_str(&raw).unwrap_or(serde_json::Value::Null)
    } else {
        serde_json::Value::Null
    };
    Ok(serde_json::json!({
        "appConfig": app_config,
        "frontend": frontend,
        "scriptJson": {},
        "scriptBytes": {},
        "clientStates": {},
        "appStatePath": overrides_path.to_string_lossy(),
        "bookshelfPath": state.bookshelf_dir().join("books.json").to_string_lossy(),
    }))
}

// ── 网络 / 系统 ──────────────────────────────────────────────────────────────

#[tauri::command(rename_all = "camelCase")]
pub fn get_local_ips() -> CommandResult<Vec<String>> {
    // 简单实现：列举本机网卡
    use if_addrs::IfAddr;
    let mut out = Vec::new();
    if let Ok(ifaces) = if_addrs::get_if_addrs() {
        for iface in ifaces {
            if let IfAddr::V4(addr) = iface.addr {
                if !addr.ip.is_loopback() {
                    out.push(addr.ip.to_string());
                }
            }
        }
    }
    Ok(out)
}

#[tauri::command(rename_all = "camelCase")]
pub fn open_dir_in_explorer(_path: String) -> CommandResult<()> {
    Err(not_implemented(
        "open_dir_in_explorer",
        "桌面端用 std::process；Android/Harmony 用平台 Intent",
    ))
}

#[tauri::command(rename_all = "camelCase")]
pub fn export_save_file(
) -> CommandResult<Option<String>> {
    Err(not_implemented(
        "export_save_file",
        "Harmony 系统保存器集成待补；桌面端走 Tauri dialog + fs plugin",
    ))
}

// ── 浏览器探测（内嵌 WebView 会话，书源调试页用） ────────────────────────────
//
// 真实现需要 Tauri 多 WebView / 窗口管理 + CDP 或 JS 桥，整套是独立 feature。
// 目前全部 stub：只在书源调试页用户手动点"浏览器探测"时触发，报错信息明确。

#[tauri::command(rename_all = "camelCase")]
pub fn browser_probe_create(_options: Option<serde_json::Value>) -> CommandResult<String> {
    Err(not_implemented(
        "browser_probe_create",
        "需要内嵌浏览器会话管理（独立 WebView + JS 桥）",
    ))
}

#[tauri::command(rename_all = "camelCase")]
pub fn browser_probe_navigate(
    _session_id: String,
    _url: String,
    _options: Option<serde_json::Value>,
) -> CommandResult<()> {
    Err(not_implemented("browser_probe_navigate", "需要内嵌浏览器会话"))
}

#[tauri::command(rename_all = "camelCase")]
pub fn browser_probe_eval(
    _session_id: String,
    _code: String,
    _options: Option<serde_json::Value>,
) -> CommandResult<serde_json::Value> {
    Err(not_implemented("browser_probe_eval", "需要内嵌浏览器会话"))
}

#[tauri::command(rename_all = "camelCase")]
pub fn browser_probe_run(
    _url: String,
    _code: String,
    _options: Option<serde_json::Value>,
) -> CommandResult<serde_json::Value> {
    Err(not_implemented("browser_probe_run", "需要内嵌浏览器会话"))
}

#[tauri::command(rename_all = "camelCase")]
pub fn browser_probe_get_cookies(_url: Option<String>) -> CommandResult<serde_json::Value> {
    Err(not_implemented("browser_probe_get_cookies", "需要内嵌浏览器会话"))
}

#[tauri::command(rename_all = "camelCase")]
pub fn browser_probe_set_cookie(
    _url: String,
    _cookie: serde_json::Value,
) -> CommandResult<()> {
    Err(not_implemented("browser_probe_set_cookie", "需要内嵌浏览器会话"))
}

#[tauri::command(rename_all = "camelCase")]
pub fn browser_probe_set_user_agent(_user_agent: String) -> CommandResult<()> {
    Err(not_implemented(
        "browser_probe_set_user_agent",
        "需要内嵌浏览器会话",
    ))
}

#[tauri::command(rename_all = "camelCase")]
pub fn browser_probe_clear_data() -> CommandResult<()> {
    Err(not_implemented("browser_probe_clear_data", "需要内嵌浏览器会话"))
}

#[tauri::command(rename_all = "camelCase")]
pub fn browser_probe_show(_session_id: String) -> CommandResult<()> {
    Err(not_implemented("browser_probe_show", "需要内嵌浏览器会话"))
}

#[tauri::command(rename_all = "camelCase")]
pub fn browser_probe_hide(_session_id: String) -> CommandResult<()> {
    Err(not_implemented("browser_probe_hide", "需要内嵌浏览器会话"))
}

#[tauri::command(rename_all = "camelCase")]
pub fn browser_probe_close(_session_id: String) -> CommandResult<()> {
    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub fn browser_probe_close_all() -> CommandResult<()> {
    Ok(())
}

// ── 音频缓存代理（音乐播放器下载受 Referer 限制的音频） ─────────────────────
//
// 前端已做兜底：本命令失败会回退直接播放，所以 stub 不会卡死播放。

#[tauri::command(rename_all = "camelCase")]
pub fn audio_resolve_cache(_request: serde_json::Value) -> CommandResult<serde_json::Value> {
    Err(not_implemented(
        "audio_resolve_cache",
        "需要 HTTP 客户端（带 Referer 下载 + 本地缓存）",
    ))
}
