use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;

use crate::errors::{CommandError, CommandResult};
use crate::state::AppState;

const FONT_DIR: &str = "user_fonts";
const FONT_INDEX: &str = "user_fonts-index.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserFont {
    pub id: String,
    pub file_name: String,
    pub display_name: String,
    pub file_size: u64,
    pub uploaded_at: i64,
}

fn font_dir(data_dir: &PathBuf) -> PathBuf {
    data_dir.join(FONT_DIR)
}

fn font_index_path(data_dir: &PathBuf) -> PathBuf {
    data_dir.join(FONT_INDEX)
}

fn read_index(data_dir: &PathBuf) -> Vec<UserFont> {
    let p = font_index_path(data_dir);
    if !p.exists() {
        return Vec::new();
    }
    let raw = std::fs::read_to_string(&p).unwrap_or_default();
    serde_json::from_str(&raw).unwrap_or_default()
}

fn write_index(data_dir: &PathBuf, list: &[UserFont]) -> CommandResult<()> {
    let p = font_index_path(data_dir);
    let tmp = p.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_string_pretty(list)?)?;
    std::fs::rename(&tmp, &p)?;
    Ok(())
}

/// 列出已上传的用户字体
#[tauri::command(rename_all = "camelCase")]
pub fn list_user_fonts(state: State<'_, AppState>) -> CommandResult<Vec<UserFont>> {
    Ok(read_index(&state.data_dir))
}

/// 上传字体文件
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadUserFontArgs {
    pub file_name: String,
    pub data: Vec<u8>,
}

#[tauri::command(rename_all = "camelCase")]
pub fn upload_user_font(
    state: State<'_, AppState>,
    args: UploadUserFontArgs,
) -> CommandResult<UserFont> {
    let dir = font_dir(&state.data_dir);
    std::fs::create_dir_all(&dir)?;
    let safe = sanitize(&args.file_name);
    let path = dir.join(&safe);
    std::fs::write(&path, &args.data)?;
    let id = format!("{:016x}", md5_like(&safe));
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    let font = UserFont {
        id,
        file_name: safe,
        display_name: args.file_name,
        file_size: args.data.len() as u64,
        uploaded_at: now,
    };
    let mut list = read_index(&state.data_dir);
    list.retain(|f| f.id != font.id);
    list.push(font.clone());
    write_index(&state.data_dir, &list)?;
    Ok(font)
}

/// 删除字体
#[tauri::command(rename_all = "camelCase")]
pub fn delete_user_font(state: State<'_, AppState>, id: String) -> CommandResult<()> {
    let mut list = read_index(&state.data_dir);
    let Some(pos) = list.iter().position(|f| f.id == id) else {
        return Err(CommandError::not_found(format!("字体 {} 不存在", id)));
    };
    let removed = list.remove(pos);
    let path = font_dir(&state.data_dir).join(&removed.file_name);
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    write_index(&state.data_dir, &list)
        .map_err(|e| CommandError::other(format!("索引写入失败: {}", e)))
}

/// 重命名字体
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameUserFontArgs {
    pub id: String,
    pub display_name: String,
}

#[tauri::command(rename_all = "camelCase")]
pub fn rename_user_font(
    state: State<'_, AppState>,
    args: RenameUserFontArgs,
) -> CommandResult<()> {
    let mut list = read_index(&state.data_dir);
    let font = list.iter_mut().find(|f| f.id == args.id).ok_or_else(|| {
        CommandError::not_found(format!("字体 {} 不存在", args.id))
    })?;
    font.display_name = args.display_name;
    write_index(&state.data_dir, &list)
        .map_err(|e| CommandError::other(format!("索引写入失败: {}", e)))
}

/// 列出系统字体（由前端用 JS 接口读取，这里返回空数组）
#[tauri::command(rename_all = "camelCase")]
pub fn list_system_fonts() -> CommandResult<Vec<String>> {
    // 系统字体枚举各平台实现差异大，前端有 npx `enumerateFonts()` 备用
    Ok(Vec::new())
}

fn sanitize(input: &str) -> String {
    if input.is_empty() {
        return "_empty_".to_string();
    }
    input
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn md5_like(input: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    input.hash(&mut h);
    h.finish()
}