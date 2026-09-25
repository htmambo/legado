use std::fs;
use std::path::{Path, PathBuf};

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
fn all_dirs(data_dir: &Path) -> CommandResult<Vec<String>> {
    let primary = primary_booksource_dir(data_dir).to_string_lossy().to_string();
    let extras = read_extra_dirs(data_dir)?;
    let mut all = Vec::with_capacity(1 + extras.len());
    all.push(primary);
    for e in extras {
        if !all.contains(&e) {
            all.push(e);
        }
    }
    Ok(all)
}

/// 返回完整目录列表：主目录在前，去重
#[tauri::command(rename_all = "camelCase")]
pub fn booksource_get_dirs(state: State<'_, AppState>) -> CommandResult<Vec<String>> {
    all_dirs(&state.data_dir)
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

/// 流式版本：扫描结果分批通过 `booksource:batch` 事件推给前端。
/// payload: { requestId, items, done, total?, error? }
///
/// 必须立即返回、后台线程扫描再推送：前端的 `listen()` 注册是异步 IPC，
/// 若在 invoke 内同步 emit，事件可能先于 JS 侧 listener 落位到达而被丢弃，
/// 前端 Promise 永远等不到 done → 书源管理页一直转圈。
#[tauri::command(rename_all = "camelCase")]
pub fn booksource_list_streaming(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    request_id: String,
) -> CommandResult<()> {
    use tauri::Emitter;

    const BATCH_SIZE: usize = 50;

    let data_dir = state.data_dir.clone();
    std::thread::spawn(move || {
        let emit = |payload: serde_json::Value| {
            if let Err(e) = app.emit("booksource:batch", payload) {
                log::warn!("booksource:batch emit 失败: {}", e);
            }
        };

        match all_dirs(&data_dir).and_then(|dirs| scan_dirs(&dirs)) {
            Ok(items) => {
                let total = items.len();
                for chunk in items.chunks(BATCH_SIZE) {
                    emit(serde_json::json!({
                        "requestId": request_id,
                        "items": chunk,
                        "done": false,
                        "total": total,
                    }));
                }
                emit(serde_json::json!({
                    "requestId": request_id,
                    "items": Vec::<BookSourceMeta>::new(),
                    "done": true,
                    "total": total,
                }));
            }
            Err(e) => {
                emit(serde_json::json!({
                    "requestId": request_id,
                    "items": Vec::<BookSourceMeta>::new(),
                    "done": true,
                    "error": e.to_string(),
                }));
            }
        }
    });
    Ok(())
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

// ── 单文件 CRUD（无 JS 引擎，纯文件 IO） ─────────────────────────────────────

fn safe_file_name(input: &str) -> CommandResult<String> {
    if input.is_empty() {
        return Err(CommandError::invalid("fileName 不能为空"));
    }
    if input.contains('/') || input.contains('\\') || input.contains("..") {
        return Err(CommandError::invalid(format!("非法 fileName: {}", input)));
    }
    Ok(input.to_string())
}

fn resolve_booksource_path(data_dir: &Path, file_name: &str, source_dir: Option<&str>) -> CommandResult<PathBuf> {
    let safe = safe_file_name(file_name)?;
    let dir = if let Some(sd) = source_dir {
        let p = Path::new(sd);
        if !p.is_absolute() {
            return Err(CommandError::invalid(format!("sourceDir 必须是绝对路径: {}", sd)));
        }
        PathBuf::from(sd)
    } else {
        primary_booksource_dir(data_dir)
    };
    Ok(dir.join(safe))
}

/// 读取书源 .js 文件内容
#[tauri::command(rename_all = "camelCase")]
pub fn booksource_read(
    state: State<'_, AppState>,
    file_name: String,
    source_dir: Option<String>,
) -> CommandResult<String> {
    let path = resolve_booksource_path(&state.data_dir, &file_name, source_dir.as_deref())?;
    if !path.exists() {
        return Err(CommandError::not_found(format!("书源文件不存在: {}", file_name)));
    }
    Ok(fs::read_to_string(&path)?)
}

/// 写入书源 .js 文件（覆盖）
#[tauri::command(rename_all = "camelCase")]
pub fn booksource_save(
    state: State<'_, AppState>,
    file_name: String,
    content: String,
    source_dir: Option<String>,
) -> CommandResult<()> {
    let path = resolve_booksource_path(&state.data_dir, &file_name, source_dir.as_deref())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("js.tmp");
    fs::write(&tmp, content)?;
    fs::rename(&tmp, &path)?;
    Ok(())
}

/// 删除书源 .js 文件
#[tauri::command(rename_all = "camelCase")]
pub fn booksource_delete(
    state: State<'_, AppState>,
    file_name: String,
    source_dir: Option<String>,
) -> CommandResult<()> {
    let path = resolve_booksource_path(&state.data_dir, &file_name, source_dir.as_deref())?;
    if path.exists() {
        fs::remove_file(&path)?;
    }
    Ok(())
}

/// 启用 / 禁用书源（用同名 marker 文件标记）
#[tauri::command(rename_all = "camelCase")]
pub fn booksource_toggle(
    state: State<'_, AppState>,
    file_name: String,
    enabled: bool,
    source_dir: Option<String>,
) -> CommandResult<()> {
    let dir = if let Some(ref sd) = source_dir {
        PathBuf::from(sd)
    } else {
        primary_booksource_dir(&state.data_dir)
    };
    let safe = safe_file_name(&file_name)?;
    fs::create_dir_all(&dir)?;
    let disabled = dir.join(format!("{}.disabled", safe));
    let enabled_marker = dir.join(format!("{}.enabled", safe));
    if disabled.exists() {
        fs::remove_file(&disabled)?;
    }
    if enabled_marker.exists() {
        fs::remove_file(&enabled_marker)?;
    }
    if enabled {
        fs::write(&enabled_marker, b"")?;
    } else {
        fs::write(&disabled, b"")?;
    }
    Ok(())
}

/// 返回书源文件绝对路径（不读内容，用于外部编辑器打开）
#[tauri::command(rename_all = "camelCase")]
pub fn booksource_resolve_path(
    state: State<'_, AppState>,
    file_name: String,
    source_dir: Option<String>,
) -> CommandResult<String> {
    let path = resolve_booksource_path(&state.data_dir, &file_name, source_dir.as_deref())?;
    Ok(path.to_string_lossy().to_string())
}

/// 在 VSCode 中打开书源文件（与 extension_open_in_vscode 同策略：返回绝对路径，
/// 由前端决定如何拉起编辑器）
#[tauri::command(rename_all = "camelCase")]
pub fn booksource_open_in_vscode(
    state: State<'_, AppState>,
    file_name: String,
    source_dir: Option<String>,
) -> CommandResult<String> {
    booksource_resolve_path(state, file_name, source_dir)
}

// ── 草稿（AI 辅助生成书源时的中间产物） ────────────────────────────────────

fn draft_path(data_dir: &Path, file_name: &str) -> CommandResult<PathBuf> {
    let safe = safe_file_name(file_name)?;
    let dir = data_dir.join("booksource_drafts");
    fs::create_dir_all(&dir)?;
    Ok(dir.join(safe))
}

/// 保存 AI 草稿
#[tauri::command(rename_all = "camelCase")]
pub fn booksource_save_draft(
    state: State<'_, AppState>,
    file_name: String,
    content: String,
) -> CommandResult<()> {
    let path = draft_path(&state.data_dir, &file_name)?;
    let tmp = path.with_extension("js.tmp");
    fs::write(&tmp, content)?;
    fs::rename(&tmp, &path)?;
    Ok(())
}

/// 删除 AI 草稿
#[tauri::command(rename_all = "camelCase")]
pub fn booksource_delete_draft(
    state: State<'_, AppState>,
    file_name: String,
) -> CommandResult<()> {
    let path = draft_path(&state.data_dir, &file_name)?;
    if path.exists() {
        fs::remove_file(&path)?;
    }
    Ok(())
}