use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::State;

use crate::errors::{CommandError, CommandResult};
use crate::state::AppState;

/// 目录：`<dataDir>/frontend_storage/`
const STORAGE_DIR: &str = "frontend_storage";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendStorageEntry {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendStorageNamespaceSummary {
    pub namespace: String,
    pub count: usize,
}

// ── 路径工具 ───────────────────────────────────────────────────────────────

fn storage_dir(data_dir: &Path) -> PathBuf {
    data_dir.join(STORAGE_DIR)
}

fn namespace_file(data_dir: &Path, namespace: &str) -> PathBuf {
    storage_dir(data_dir).join(format!("{}.json", sanitize_namespace(namespace)))
}

/// namespace 路径安全化：只允许字母数字 + `-` `_` `.`
fn sanitize_namespace(ns: &str) -> String {
    if ns.is_empty() {
        return "_empty_".to_string();
    }
    ns.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

// ── 文件读写 ───────────────────────────────────────────────────────────────

fn read_namespace(
    data_dir: &Path,
    namespace: &str,
) -> CommandResult<BTreeMap<String, String>> {
    let path = namespace_file(data_dir, namespace);
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    let raw = std::fs::read_to_string(&path)?;
    if raw.trim().is_empty() {
        return Ok(BTreeMap::new());
    }
    // 文件存的就是 `{key: value}` 的扁平 map；serde 接受任意顺序。
    Ok(serde_json::from_str(&raw)?)
}

fn write_namespace(
    data_dir: &Path,
    namespace: &str,
    map: &BTreeMap<String, String>,
) -> CommandResult<()> {
    let dir = storage_dir(data_dir);
    std::fs::create_dir_all(&dir)?;
    let path = namespace_file(data_dir, namespace);
    let tmp = dir.join(format!("{}.tmp", sanitize_namespace(namespace)));
    let json = serde_json::to_string_pretty(map)?;
    std::fs::write(&tmp, json)?;
    std::fs::rename(&tmp, &path)?;
    Ok(())
}

// ── Tauri 命令 ─────────────────────────────────────────────────────────────

/// 列出所有已有命名空间（每个命名空间下有多少条记录）。
#[tauri::command(rename_all = "camelCase")]
pub fn frontend_storage_list_namespaces(
    state: State<'_, AppState>,
) -> CommandResult<Vec<FrontendStorageNamespaceSummary>> {
    let dir = storage_dir(&state.data_dir);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut out: Vec<FrontendStorageNamespaceSummary> = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        // 跳过 .tmp 文件
        if stem.ends_with(".tmp") {
            continue;
        }
        let raw = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let count = if raw.trim().is_empty() {
            0
        } else {
            serde_json::from_str::<serde_json::Value>(&raw)
                .ok()
                .and_then(|v| v.as_object().map(|o| o.len()))
                .unwrap_or(0)
        };
        out.push(FrontendStorageNamespaceSummary {
            namespace: stem.to_string(),
            count,
        });
    }
    // 按 namespace 名字稳定排序，方便前端展示
    out.sort_by(|a, b| a.namespace.cmp(&b.namespace));
    Ok(out)
}

/// 读取某个命名空间下的所有 key-value。
#[tauri::command(rename_all = "camelCase")]
pub fn frontend_storage_list(
    state: State<'_, AppState>,
    namespace: String,
) -> CommandResult<Vec<FrontendStorageEntry>> {
    let map = read_namespace(&state.data_dir, &namespace)?;
    Ok(map
        .into_iter()
        .map(|(key, value)| FrontendStorageEntry { key, value })
        .collect())
}

/// 设置某个 namespace 下的某个 key 的 value（覆盖写入）。
#[tauri::command(rename_all = "camelCase")]
pub fn frontend_storage_set(
    state: State<'_, AppState>,
    namespace: String,
    key: String,
    value: String,
) -> CommandResult<()> {
    if key.is_empty() {
        return Err(CommandError::invalid("frontend_storage_set 不允许空 key"));
    }
    let mut map = read_namespace(&state.data_dir, &namespace)?;
    map.insert(key, value);
    write_namespace(&state.data_dir, &namespace, &map)
}

/// 删除某个 namespace 下的某个 key；key 不存在视为成功（幂等）。
#[tauri::command(rename_all = "camelCase")]
pub fn frontend_storage_remove(
    state: State<'_, AppState>,
    namespace: String,
    key: String,
) -> CommandResult<()> {
    let mut map = read_namespace(&state.data_dir, &namespace)?;
    map.remove(&key);
    write_namespace(&state.data_dir, &namespace, &map)
}