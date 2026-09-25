use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use tauri::State;

use crate::errors::{CommandError, CommandResult};
use crate::extensions::model::ExtensionMeta;
use crate::state::AppState;

/// `<dataDir>/extensions/` —— 前端扩展目录
const EXT_DIR: &str = "extensions";
/// 启用标记文件名，与前端约定一致
const ENABLED_SUFFIX: &str = ".enabled";
/// 禁用标记文件名，与前端约定一致
const DISABLED_SUFFIX: &str = ".disabled";

fn extensions_root(data_dir: &Path) -> PathBuf {
    data_dir.join(EXT_DIR)
}

fn file_enabled(data_dir: &Path, file_name: &str) -> bool {
    // 约定：foo.js.enabled 存在 → 启用；foo.js.disabled 存在 → 禁用；都没 → 默认启用
    let disabled = extensions_root(data_dir).join(format!("{}{}", file_name, DISABLED_SUFFIX));
    let enabled = extensions_root(data_dir).join(format!("{}{}", file_name, ENABLED_SUFFIX));
    if disabled.exists() {
        return false;
    }
    if enabled.exists() {
        return true;
    }
    true
}

fn set_file_enabled(data_dir: &Path, file_name: &str, enabled: bool) -> CommandResult<()> {
    let root = extensions_root(data_dir);
    fs::create_dir_all(&root)?;
    let disabled = root.join(format!("{}{}", file_name, DISABLED_SUFFIX));
    let enabled_marker = root.join(format!("{}{}", file_name, ENABLED_SUFFIX));
    // 清理两个标记
    if disabled.exists() {
        fs::remove_file(&disabled)?;
    }
    if enabled_marker.exists() {
        fs::remove_file(&enabled_marker)?;
    }
    if enabled {
        // 启用：创建 .enabled marker（确保存在）
        fs::write(&enabled_marker, b"")?;
    } else {
        // 禁用：创建 .disabled marker
        fs::write(&disabled, b"")?;
    }
    Ok(())
}

fn safe_file_name(input: &str) -> CommandResult<String> {
    if input.is_empty() {
        return Err(CommandError::invalid("扩展文件名不能为空"));
    }
    if input.contains('/') || input.contains('\\') || input.contains("..") {
        return Err(CommandError::invalid(format!("非法文件名: {}", input)));
    }
    Ok(input.to_string())
}

fn scan_extensions(data_dir: &Path) -> CommandResult<Vec<ExtensionMeta>> {
    let root = extensions_root(data_dir);
    if !root.exists() {
        fs::create_dir_all(&root)?;
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(&root)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|s| s.to_str()).map(|s| s.to_string())
        else {
            continue;
        };
        if !file_name.ends_with(".js") {
            continue;
        }
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let modified_at = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        let enabled = file_enabled(data_dir, &file_name);
        out.push(ExtensionMeta {
            file_name: file_name.clone(),
            // 头部解析留给前端 P0，这里先用文件名作为占位
            name: file_name.trim_end_matches(".js").to_string(),
            namespace: String::new(),
            version: String::new(),
            description: String::new(),
            author: String::new(),
            match_patterns: Vec::new(),
            grants: Vec::new(),
            run_at: String::new(),
            category: String::new(),
            enabled,
            file_size: meta.len(),
            modified_at,
        });
    }
    out.sort_by(|a, b| a.file_name.cmp(&b.file_name));
    Ok(out)
}

// ── Tauri 命令 ─────────────────────────────────────────────────────────────

#[tauri::command(rename_all = "camelCase")]
pub fn extension_get_dir(state: State<'_, AppState>) -> CommandResult<String> {
    Ok(extensions_root(&state.data_dir).to_string_lossy().to_string())
}

#[tauri::command(rename_all = "camelCase")]
pub fn extension_list(state: State<'_, AppState>) -> CommandResult<Vec<ExtensionMeta>> {
    scan_extensions(&state.data_dir)
}

#[tauri::command(rename_all = "camelCase")]
pub fn extension_read(state: State<'_, AppState>, file_name: String) -> CommandResult<String> {
    let safe = safe_file_name(&file_name)?;
    let path = extensions_root(&state.data_dir).join(&safe);
    if !path.exists() {
        return Err(CommandError::not_found(format!("扩展文件不存在: {}", safe)));
    }
    Ok(fs::read_to_string(&path)?)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionSaveArgs {
    pub file_name: String,
    pub content: String,
}

#[tauri::command(rename_all = "camelCase")]
pub fn extension_save(state: State<'_, AppState>, file_name: String, content: String) -> CommandResult<()> {
    let safe = safe_file_name(&file_name)?;
    let dir = extensions_root(&state.data_dir);
    fs::create_dir_all(&dir)?;
    let path = dir.join(&safe);
    let tmp = dir.join(format!("{}.tmp", safe));
    fs::write(&tmp, content)?;
    fs::rename(&tmp, &path)?;
    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub fn extension_delete(state: State<'_, AppState>, file_name: String) -> CommandResult<()> {
    let safe = safe_file_name(&file_name)?;
    let dir = extensions_root(&state.data_dir);
    let path = dir.join(&safe);
    if path.exists() {
        fs::remove_file(&path)?;
    }
    // 顺带清理标记文件
    let disabled = dir.join(format!("{}{}", safe, DISABLED_SUFFIX));
    let enabled_marker = dir.join(format!("{}{}", safe, ENABLED_SUFFIX));
    if disabled.exists() {
        fs::remove_file(&disabled)?;
    }
    if enabled_marker.exists() {
        fs::remove_file(&enabled_marker)?;
    }
    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub fn extension_toggle(
    state: State<'_, AppState>,
    file_name: String,
    enabled: bool,
) -> CommandResult<()> {
    let safe = safe_file_name(&file_name)?;
    set_file_enabled(&state.data_dir, &safe, enabled)
}

#[tauri::command(rename_all = "camelCase")]
pub fn extension_open_in_vscode(
    state: State<'_, AppState>,
    file_name: String,
) -> CommandResult<String> {
    let safe = safe_file_name(&file_name)?;
    let path = extensions_root(&state.data_dir).join(&safe);
    Ok(path.to_string_lossy().to_string())
}