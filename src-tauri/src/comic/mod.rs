use std::path::{Path, PathBuf};

use serde::Deserialize;
use tauri::State;

use crate::errors::CommandResult;
use crate::state::AppState;

/// 漫画图片缓存根目录：`<dataDir>/comic_cache/`
const COMIC_ROOT: &str = "comic_cache";

fn cache_root(data_dir: &Path) -> PathBuf {
    data_dir.join(COMIC_ROOT)
}

/// URL 哈希 key（避免反向单依赖/路径分隔符）
fn url_to_path(input: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(input, &mut hasher);
    format!("{:016x}", std::hash::Hasher::finish(&hasher))
}

fn chapter_cache_dir(data_dir: &Path, file_name: &str, chapter_url: &str) -> PathBuf {
    cache_root(data_dir)
        .join(sanitize(file_name))
        .join(url_to_path(chapter_url))
}

/// 文件名清理
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

/// 清理某一本书的缓存（file_name 为 null 时清空全部）
#[tauri::command(rename_all = "camelCase")]
pub fn comic_cache_clear(
    state: State<'_, AppState>,
    file_name: Option<String>,
) -> CommandResult<u64> {
    let root = cache_root(&state.data_dir);
    if !root.exists() {
        return Ok(0);
    }
    let mut freed = 0u64;
    if let Some(name) = file_name {
        let dir = root.join(sanitize(&name));
        if dir.exists() {
            freed += dir_size(&dir)?;
            fs::remove_dir_all(&dir)?;
        }
    } else {
        freed += dir_size(&root)?;
        fs::remove_dir_all(&root)?;
        fs::create_dir_all(&root)?;
    }
    Ok(freed)
}

fn dir_size(p: &Path) -> std::io::Result<u64> {
    let mut total = 0u64;
    if p.is_file() {
        return Ok(p.metadata()?.len());
    }
    for entry in fs::read_dir(p)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            total += dir_size(&path)?;
        } else {
            total += entry.metadata()?.len();
        }
    }
    Ok(total)
}

/// 计算缓存总字节数
#[tauri::command(rename_all = "camelCase")]
pub fn comic_cache_size(state: State<'_, AppState>) -> CommandResult<u64> {
    let root = cache_root(&state.data_dir);
    if !root.exists() {
        return Ok(0);
    }
    Ok(dir_size(&root)?)
}

/// 列出某本书某章已缓存的页索引 + 每页像素尺寸 [w, h]
#[tauri::command(rename_all = "camelCase")]
pub fn comic_get_page_sizes(
    state: State<'_, AppState>,
    file_name: String,
    book_url: String,
    book_name: String,
    chapter_index: i32,
) -> CommandResult<Vec<Option<[u32; 2]>>> {
    let dir = chapter_cache_dir(&state.data_dir, &file_name, &book_url).join(format!("{}", chapter_index));
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let size_path = dir.join(format!("{}.sizes.json", sanitize(&book_name)));
    if !size_path.exists() {
        // 没有 size 文件说明不是本 Rust 写入的，返回空数组
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(&size_path)?;
    if raw.trim().is_empty() {
        return Ok(Vec::new());
    }
    Ok(serde_json::from_str(&raw)?)
}

/// 读取单页缓存图片字节（base64）
#[tauri::command(rename_all = "camelCase")]
pub fn comic_get_cached_page(
    state: State<'_, AppState>,
    file_name: String,
    book_url: String,
    chapter_index: i32,
    page_index: i32,
) -> CommandResult<Option<String>> {
    let dir = chapter_cache_dir(&state.data_dir, &file_name, &book_url).join(format!("{}", chapter_index));
    let path = dir.join(format!("{}.bin", page_index));
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&path)?;
    Ok(Some(base64_encode(&bytes)))
}

/// 清理某一本书某一章的缓存
#[tauri::command(rename_all = "camelCase")]
pub fn comic_cache_clear_chapter(
    state: State<'_, AppState>,
    file_name: String,
    book_url: String,
    chapter_index: i32,
) -> CommandResult<u64> {
    let dir = chapter_cache_dir(&state.data_dir, &file_name, &book_url).join(format!("{}", chapter_index));
    if !dir.exists() {
        return Ok(0);
    }
    let freed = dir_size(&dir)?;
    fs::remove_dir_all(&dir)?;
    Ok(freed)
}

/// 下载章节所有图片（无 HTTP 客户端 stub —— P0 没真正下载）
///
/// **当前状态**：没有 HTTP 客户端 + 没有 booksource 引擎。
/// 返回 0 / 不写文件，让上层知道"没拉到"。
/// 真正实现要等 P2 加 reqwest + 引擎。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComicDownloadImagesArgs {
    pub file_name: String,
    pub book_url: String,
    pub book_name: String,
    pub chapter_url: String,
    pub chapter_index: i32,
    pub image_urls: Vec<String>,
}

#[tauri::command(rename_all = "camelCase")]
pub fn comic_download_images(_args: ComicDownloadImagesArgs) -> CommandResult<u32> {
    // 没有 HTTP 客户端和 JS 引擎，无法实际下载。返回 0（缓存 0 张）
    // 引擎就绪后这里替换为：HTTP 拉图 → 写到 chapter_cache_dir →
    // 写 size json → 返回成功张数
    Ok(0)
}

// ── base64 ──────────────────────────────────────────────────────────────────

fn base64_encode(bytes: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((bytes.len() + 2) * 4 / 3);
    for chunk in bytes.chunks(3) {
        let b0 = chunk.first().copied().unwrap_or(0);
        let b1 = chunk.get(1).copied().unwrap_or(0);
        let b2 = chunk.get(2).copied().unwrap_or(0);
        let triple = ((b0 as u32) << 16) | ((b1 as u32) << 8) | (b2 as u32);
        out.push(CHARS[((triple >> 18) & 0x3f) as usize] as char);
        out.push(CHARS[((triple >> 12) & 0x3f) as usize] as char);
        match chunk.len() {
            3 => {
                out.push(CHARS[((triple >> 6) & 0x3f) as usize] as char);
                out.push(CHARS[(triple & 0x3f) as usize] as char);
            }
            2 => {
                out.push(CHARS[((triple >> 6) & 0x3f) as usize] as char);
                out.push('=');
            }
            1 => {
                out.push('=');
                out.push('=');
            }
            _ => {}
        }
    }
    out
}

use std::fs;