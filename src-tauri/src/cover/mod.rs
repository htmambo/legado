use std::path::{Path, PathBuf};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::errors::{CommandError, CommandResult};
use crate::state::AppState;

/// `<dataDir>/cover_cache/`
const COVER_ROOT: &str = "cover_cache";

fn cover_root(data_dir: &Path) -> PathBuf {
    data_dir.join(COVER_ROOT)
}

fn url_key(input: &str) -> String {
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// 封面缓存查询入参
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverRequest {
    pub url: String,
    #[serde(default)]
    pub referer: Option<String>,
    #[serde(default)]
    pub headers: Vec<(String, String)>,
}

/// 封面缓存返回
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverResolveResult {
    pub mime: String,
    pub base64: String,
}

/// 查询封面缓存：命中返回 base64，未命中返回 NotFound
#[tauri::command(rename_all = "camelCase")]
pub fn cover_resolve_cache(
    state: State<'_, AppState>,
    request: CoverRequest,
) -> CommandResult<CoverResolveResult> {
    let key = url_key(&request.url);
    let path = cover_root(&state.data_dir).join(format!("{}.bin", key));
    if !path.exists() {
        return Err(CommandError::not_found(format!("封面缓存未命中: {}", request.url)));
    }
    let bytes = std::fs::read(&path)?;
    let mime = std::fs::read_to_string(path.with_extension("mime")).unwrap_or_else(|_| "image/jpeg".to_string());
    Ok(CoverResolveResult {
        mime,
        base64: base64_encode(&bytes),
    })
}

/// 计算封面缓存总字节数
#[tauri::command(rename_all = "camelCase")]
pub fn cover_cache_size(state: State<'_, AppState>) -> CommandResult<u64> {
    let root = cover_root(&state.data_dir);
    if !root.exists() {
        return Ok(0);
    }
    Ok(dir_size(&root)?)
}

/// 清理封面缓存
#[tauri::command(rename_all = "camelCase")]
pub fn cover_cache_clear(state: State<'_, AppState>) -> CommandResult<u64> {
    let root = cover_root(&state.data_dir);
    if !root.exists() {
        return Ok(0);
    }
    let freed = dir_size(&root)?;
    std::fs::remove_dir_all(&root)?;
    std::fs::create_dir_all(&root)?;
    Ok(freed)
}

fn dir_size(p: &Path) -> std::io::Result<u64> {
    let mut total = 0u64;
    if p.is_file() {
        return Ok(p.metadata()?.len());
    }
    for entry in std::fs::read_dir(p)? {
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