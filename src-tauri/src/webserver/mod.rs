use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;

use crate::errors::{CommandError, CommandResult};
use crate::state::AppState;

const STATE_FILE: &str = "webserver-state.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WebServerState {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub port: u32,
    #[serde(default)]
    pub dist_path: String,
    #[serde(default)]
    pub running: bool,
    #[serde(default)]
    pub last_error: String,
}

fn webserver_state_path(data_dir: &PathBuf) -> PathBuf {
    data_dir.join(STATE_FILE)
}

fn read_state(data_dir: &PathBuf) -> WebServerState {
    let p = webserver_state_path(data_dir);
    if !p.exists() {
        return WebServerState {
            port: 7688,
            ..WebServerState::default()
        };
    }
    let raw = std::fs::read_to_string(&p).unwrap_or_default();
    serde_json::from_str(&raw).unwrap_or_else(|_| WebServerState {
        port: 7688,
        ..WebServerState::default()
    })
}

/// 获取 Web 服务器状态（启用 / 端口 / 路径 / 运行中）
#[tauri::command(rename_all = "camelCase")]
pub fn web_server_status(state: State<'_, AppState>) -> CommandResult<WebServerState> {
    Ok(read_state(&state.data_dir))
}

/// 启动 Web 服务器：当前 stub（HTTP 服务器未嵌入）
#[tauri::command(rename_all = "camelCase")]
pub fn web_server_start(state: State<'_, AppState>) -> CommandResult<WebServerState> {
    let mut s = read_state(&state.data_dir);
    s.running = false;
    s.last_error = "Web 服务器暂未实现：HTTP server 待补".to_string();
    std::fs::write(
        webserver_state_path(&state.data_dir),
        serde_json::to_string_pretty(&s)?,
    )?;
    Ok(s)
}

/// 停止 Web 服务器：stub
#[tauri::command(rename_all = "camelCase")]
pub fn web_server_stop(state: State<'_, AppState>) -> CommandResult<WebServerState> {
    let mut s = read_state(&state.data_dir);
    s.running = false;
    s.last_error.clear();
    std::fs::write(
        webserver_state_path(&state.data_dir),
        serde_json::to_string_pretty(&s)?,
    )?;
    Ok(s)
}

/// 选择静态文件目录：stub（实际打开系统目录选择器由前端 dialog plugin 处理）
#[tauri::command(rename_all = "camelCase")]
pub fn web_server_pick_dist_dir() -> CommandResult<String> {
    Err(CommandError::other(
        "web_server_pick_dist_dir 暂未实现：Harmony 选择器集成待补".to_string(),
    ))
}