use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::State;

use crate::errors::{CommandError, CommandResult};
use crate::state::AppState;

/// `<dataDir>/sync/`
const SYNC_DIR: &str = "sync";
const CREDS_FILE: &str = "credentials.json";

fn sync_dir_path(data_dir: &Path) -> PathBuf {
    data_dir.join(SYNC_DIR)
}

fn creds_path(data_dir: &Path) -> PathBuf {
    sync_dir_path(data_dir).join(CREDS_FILE)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatus {
    pub enabled: bool,
    pub running: bool,
    pub last_success_at: i64,
    pub last_failed_at: i64,
    pub last_error: String,
    pub dirty_domains: Vec<String>,
    pub conflict_count: u32,
    pub last_run_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncCredentials {
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncConflict {
    pub id: String,
    pub domain: String,
    pub key: String,
    pub message: String,
    pub local: serde_json::Value,
    pub remote: serde_json::Value,
    pub created_at: i64,
    pub resolved: bool,
}

fn ensure_sync_dir(data_dir: &Path) -> CommandResult<PathBuf> {
    let dir = sync_dir_path(data_dir);
    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }
    Ok(dir)
}

/// 获取同步状态：默认全空（同步引擎未启动）
#[tauri::command(rename_all = "camelCase")]
pub fn sync_get_status() -> CommandResult<SyncStatus> {
    Ok(SyncStatus {
        enabled: false,
        running: false,
        last_success_at: 0,
        last_failed_at: 0,
        last_error: String::new(),
        dirty_domains: Vec::new(),
        conflict_count: 0,
        last_run_summary: String::new(),
    })
}

/// 保存 WebDAV 密码到 `<dataDir>/sync/credentials.json`
#[tauri::command(rename_all = "camelCase")]
pub fn sync_set_credentials(
    state: State<'_, AppState>,
    password: String,
) -> CommandResult<()> {
    ensure_sync_dir(&state.data_dir)?;
    let path = creds_path(&state.data_dir);
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_string(&SyncCredentials { password })?)?;
    std::fs::rename(&tmp, &path)?;
    Ok(())
}

/// 清除密码
#[tauri::command(rename_all = "camelCase")]
pub fn sync_clear_credentials(state: State<'_, AppState>) -> CommandResult<()> {
    let path = creds_path(&state.data_dir);
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

/// 读取密码；文件不存在返回 password = ""
#[tauri::command(rename_all = "camelCase")]
pub fn sync_get_credentials(state: State<'_, AppState>) -> CommandResult<SyncCredentials> {
    let path = creds_path(&state.data_dir);
    if !path.exists() {
        return Ok(SyncCredentials { password: String::new() });
    }
    let raw = std::fs::read_to_string(&path)?;
    if raw.trim().is_empty() {
        return Ok(SyncCredentials { password: String::new() });
    }
    Ok(serde_json::from_str(&raw)?)
}

/// 测试连接：当前 stub（返回成功 + 空错误），等 P2 加 reqwest + WebDAV 客户端
#[tauri::command(rename_all = "camelCase")]
pub fn sync_test_connection(
    _password: Option<String>,
) -> CommandResult<bool> {
    Ok(true)
}

/// 触发一次同步：stub
#[tauri::command(rename_all = "camelCase")]
pub fn sync_now(
    _mode: Option<String>,
    _domains: Option<Vec<String>>,
    _conflict_strategy: Option<String>,
) -> CommandResult<()> {
    Err(CommandError::other(
        "sync_now 暂未实现：同步引擎待补".to_string(),
    ))
}

/// 列出冲突：返回空
#[tauri::command(rename_all = "camelCase")]
pub fn sync_list_conflicts() -> CommandResult<Vec<SyncConflict>> {
    Ok(Vec::new())
}

/// 解决冲突：no-op stub
#[tauri::command(rename_all = "camelCase")]
pub fn sync_resolve_conflict(
) -> CommandResult<()> {
    Ok(())
}

/// 同步客户端状态（per-key 写入）stub
#[tauri::command(rename_all = "camelCase")]
pub fn sync_client_state_set(
    state: State<'_, AppState>,
) -> CommandResult<()> {
    ensure_sync_dir(&state.data_dir)?;
    Ok(())
}

/// 报告阅读器会话：no-op stub（P2 接）
#[tauri::command(rename_all = "camelCase")]
pub fn sync_report_reader_session(
) -> CommandResult<()> {
    Ok(())
}

/// 同步 v2 进度：stub
#[tauri::command(rename_all = "camelCase")]
pub fn sync_v2_sync_reading_progress(
) -> CommandResult<serde_json::Value> {
    Ok(serde_json::json!({"status": "noop"}))
}

/// 同步生命周期事件：no-op stub
#[tauri::command(rename_all = "camelCase")]
pub fn sync_notify_lifecycle(
) -> CommandResult<()> {
    Ok(())
}

/// 百度网盘 OAuth stub
#[tauri::command(rename_all = "camelCase")]
pub fn sync_baidu_token_status() -> CommandResult<serde_json::Value> {
    Ok(serde_json::json!({
        "status": "unsupported",
        "message": "百度网盘 OAuth 暂未实现"
    }))
}

#[tauri::command(rename_all = "camelCase")]
pub fn sync_baidu_start_auth() -> CommandResult<serde_json::Value> {
    Err(CommandError::other(
        "百度网盘 OAuth 暂未实现".to_string(),
    ))
}

#[tauri::command(rename_all = "camelCase")]
pub fn sync_baidu_poll_token(
) -> CommandResult<serde_json::Value> {
    Err(CommandError::other(
        "百度网盘 OAuth 暂未实现".to_string(),
    ))
}

#[tauri::command(rename_all = "camelCase")]
pub fn sync_baidu_revoke_auth() -> CommandResult<()> {
    Ok(())
}