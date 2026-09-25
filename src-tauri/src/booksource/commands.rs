use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use tauri::State;

use crate::booksource::model::BookSourceMeta;
use crate::errors::{CommandError, CommandResult};
use crate::state::AppState;

/// `<dataDir>/booksource/` —— 用户/示例书源目录
const PRIMARY_DIR: &str = "booksource";
/// `<dataDir>/external-booksource-dirs.json` —— 用户额外添加的外部目录
const EXTRA_DIRS_FILE: &str = "external-booksource-dirs.json";

/// 主目录路径
pub fn primary_booksource_dir(data_dir: &Path) -> PathBuf {
    data_dir.join(PRIMARY_DIR)
}

/// 返回完整目录列表：主目录在前，去重
#[tauri::command(rename_all = "camelCase")]
pub fn booksource_get_dirs(state: State<'_, AppState>) -> CommandResult<Vec<String>> {
    let primary = primary_booksource_dir(&state.data_dir).to_string_lossy().to_string();
    let extras = read_extra_dirs(&state.data_dir)?;
    let mut all = Vec::with_capacity(1 + extras.len());
    all.push(primary);
    for e in extras {
        if !all.contains(&e) {
            all.push(e);
        }
    }
    Ok(all)
}

/// 返回主书源目录绝对路径
#[tauri::command(rename_all = "camelCase")]
pub fn booksource_get_dir(state: State<'_, AppState>) -> CommandResult<String> {
    Ok(primary_booksource_dir(&state.data_dir).to_string_lossy().to_string())
}

/// 添加外部书源目录
#[tauri::command(rename_all = "camelCase")]
pub fn booksource_add_dir(
    state: State<'_, AppState>,
    dir_path: String,
) -> CommandResult<()> {
    if dir_path.trim().is_empty() {
        return Err(CommandError::invalid("外部书源目录不能为空"));
    }
    let normalized = normalize_dir(&dir_path)?;
    let mut extras = read_extra_dirs(&state.data_dir)?;
    if !extras.iter().any(|d| d == &normalized) {
        extras.push(normalized);
        write_extra_dirs(&state.data_dir, &extras)?;
    }
    Ok(())
}

/// 移除外部书源目录
#[tauri::command(rename_all = "camelCase")]
pub fn booksource_remove_dir(
    state: State<'_, AppState>,
    dir_path: String,
) -> CommandResult<()> {
    let normalized = normalize_dir(&dir_path)?;
    let mut extras = read_extra_dirs(&state.data_dir)?;
    let before = extras.len();
    extras.retain(|d| d != &normalized);
    if extras.len() != before {
        write_extra_dirs(&state.data_dir, &extras)?;
    }
    Ok(())
}

/// 扫描主目录（或全部目录）的所有书源文件，返回元数据列表
#[tauri::command(rename_all = "camelCase")]
pub fn booksource_list(state: State<'_, AppState>) -> CommandResult<Vec<BookSourceMeta>> {
    let dirs = booksource_get_dirs(state.clone())?;
    scan_dirs(&dirs)
}

// ── 内部工具 ───────────────────────────────────────────────────────────────

fn normalize_dir(input: &str) -> CommandResult<String> {
    let p = Path::new(input);
    if !p.exists() {
        return Err(CommandError::invalid(format!("目录不存在: {}", input)));
    }
    if !p.is_dir() {
        return Err(CommandError::invalid(format!("不是目录: {}", input)));
    }
    // 规范化为绝对路径
    let abs = p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
    Ok(abs.to_string_lossy().to_string())
}

fn extra_dirs_path(data_dir: &Path) -> PathBuf {
    data_dir.join(EXTRA_DIRS_FILE)
}

fn read_extra_dirs(data_dir: &Path) -> CommandResult<Vec<String>> {
    let path = extra_dirs_path(data_dir);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(&path)?;
    if raw.trim().is_empty() {
        return Ok(Vec::new());
    }
    Ok(serde_json::from_str(&raw)?)
}

fn write_extra_dirs(data_dir: &Path, dirs: &[String]) -> CommandResult<()> {
    let path = extra_dirs_path(data_dir);
    let tmp = data_dir.join(format!("{}.tmp", EXTRA_DIRS_FILE));
    let json = serde_json::to_string_pretty(dirs)?;
    fs::write(&tmp, json)?;
    fs::rename(&tmp, &path)?;
    Ok(())
}

fn scan_dirs(dirs: &[String]) -> CommandResult<Vec<BookSourceMeta>> {
    let mut out = Vec::new();
    for dir in dirs {
        let p = Path::new(dir);
        if !p.is_dir() {
            continue;
        }
        let entries = match fs::read_dir(p) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("js") {
                continue;
            }
            let Some(file_name) = path.file_name().and_then(|s| s.to_str()).map(|s| s.to_string())
            else {
                continue;
            };
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
            out.push(BookSourceMeta {
                source_key: None,
                uuid: None,
                file_name: file_name.clone(),
                name: file_name.trim_end_matches(".js").to_string(),
                author: None,
                logo: None,
                description: None,
                enabled: true,
                file_size: meta.len(),
                modified_at,
                source_dir: p.to_string_lossy().to_string(),
                source_type: "novel".to_string(),
            });
        }
    }
    // 按文件名排序，前端展示稳定
    out.sort_by(|a, b| a.file_name.cmp(&b.file_name));
    Ok(out)
}