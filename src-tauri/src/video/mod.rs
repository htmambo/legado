use serde::{Deserialize, Serialize};
use tauri::State;

use crate::errors::{CommandError, CommandResult};
use crate::state::AppState;

const PROXIES_FILE: &str = "video_proxies.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct VideoProxyHandle {
    pub id: String,
    pub port: u32,
    pub url: String,
}

/// 当前活动的代理列表
fn read_proxies(data_dir: &Path) -> Vec<VideoProxyHandle> {
    let p = data_dir.join(PROXIES_FILE);
    if !p.exists() {
        return Vec::new();
    }
    let raw = std::fs::read_to_string(&p).unwrap_or_default();
    serde_json::from_str(&raw).unwrap_or_default()
}

fn write_proxies(data_dir: &Path, list: &[VideoProxyHandle]) -> CommandResult<()> {
    let p = data_dir.join(PROXIES_FILE);
    let tmp = p.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_string_pretty(list)?)?;
    std::fs::rename(&tmp, &p)?;
    Ok(())
}

use std::path::Path;

/// 启动视频本地代理：stub（HTTP 代理服务待嵌入）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartVideoProxyArgs {
    pub url: String,
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub concurrency: Option<u32>,
}

#[tauri::command(rename_all = "camelCase")]
pub fn start_video_proxy(
    _state: State<'_, AppState>,
    args: StartVideoProxyArgs,
) -> CommandResult<VideoProxyHandle> {
    let _ = args;
    Err(CommandError::other(
        "start_video_proxy 暂未实现：视频本地代理待补".to_string(),
    ))
}

/// 停止视频代理：按 port 找并移除
#[tauri::command(rename_all = "camelCase")]
pub fn stop_video_proxy(state: State<'_, AppState>, port: u32) -> CommandResult<()> {
    let mut list = read_proxies(&state.data_dir);
    list.retain(|p| p.port != port);
    write_proxies(&state.data_dir, &list)
}