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
    pub headers: Option<std::collections::HashMap<String, String>>,
}

/// 封面缓存返回（与前端 BookCoverImg 的契约一致）
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverResolveResult {
    /// 缓存文件绝对路径（前端 toFileSrcSync 转 asset:// 显示）
    pub local_path: String,
    /// `local://` 引用（可回写 coverUrl，BookCoverImg 识别为本地文件）
    pub local_ref: String,
}

fn ext_from_mime(mime: &str) -> &str {
    match mime.split(';').next().unwrap_or("").trim() {
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/svg+xml" => "svg",
        "image/avif" => "avif",
        "image/bmp" => "bmp",
        _ => "jpg",
    }
}

/// 把字节写入封面缓存，返回 (localPath, localRef)
fn write_cache(data_dir: &Path, key: &str, bytes: &[u8], mime: &str) -> CommandResult<(String, String)> {
    let root = cover_root(data_dir);
    std::fs::create_dir_all(&root)?;
    let path = root.join(format!("{}.{}", key, ext_from_mime(mime)));
    std::fs::write(&path, bytes)?;
    let local_path = path.to_string_lossy().to_string();
    Ok((local_path.clone(), format!("local://{}", local_path)))
}

/// 查找已缓存的封面文件（任意扩展名）
fn find_cached(data_dir: &Path, key: &str) -> Option<PathBuf> {
    let root = cover_root(data_dir);
    for ext in ["jpg", "png", "gif", "webp", "svg", "avif", "bmp", "bin"] {
        let p = root.join(format!("{}.{}", key, ext));
        if p.exists() {
            return Some(p);
        }
    }
    None
}

fn base64_decode(input: &str) -> CommandResult<Vec<u8>> {
    let table = |c: u8| -> Option<u8> {
        match c {
            b'A'..=b'Z' => Some(c - b'A'),
            b'a'..=b'z' => Some(c - b'a' + 26),
            b'0'..=b'9' => Some(c - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    };
    let mut out = Vec::with_capacity(input.len() * 3 / 4);
    let mut acc: u32 = 0;
    let mut nbits = 0u32;
    for &b in input.as_bytes() {
        if b == b'=' || b == b'\n' || b == b'\r' {
            continue;
        }
        let Some(v) = table(b) else {
            return Err(CommandError::invalid("data: URL base64 解码失败"));
        };
        acc = (acc << 6) | v as u32;
        nbits += 6;
        if nbits >= 8 {
            nbits -= 8;
            out.push((acc >> nbits) as u8);
        }
    }
    Ok(out)
}

fn hex_val(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

fn percent_decode(input: &str) -> CommandResult<Vec<u8>> {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            if i + 2 >= bytes.len() {
                return Err(CommandError::invalid("data: URL percent 解码失败"));
            }
            let hi = hex_val(bytes[i + 1])
                .ok_or_else(|| CommandError::invalid("data: URL percent 解码失败"))?;
            let lo = hex_val(bytes[i + 2])
                .ok_or_else(|| CommandError::invalid("data: URL percent 解码失败"))?;
            out.push((hi << 4) | lo);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    Ok(out)
}

/// 查询/获取封面缓存：
/// - data: URL（生成封面）：解码后直接写缓存返回
/// - http(s)：命中直接返回；未命中带 Referer/headers 下载（reqwest，绕开 CORS）
#[tauri::command(rename_all = "camelCase")]
pub async fn cover_resolve_cache(
    state: State<'_, AppState>,
    request: CoverRequest,
) -> CommandResult<CoverResolveResult> {
    let key = url_key(&request.url);

    // data: URL —— 生成封面走这里
    if let Some(data_part) = request.url.strip_prefix("data:") {
        let (meta, payload) = data_part
            .split_once(',')
            .ok_or_else(|| CommandError::invalid("非法 data: URL"))?;
        let mime = meta.trim_end_matches(";base64").trim();
        let mime = if mime.is_empty() { "image/png" } else { mime };
        let bytes = if meta.ends_with(";base64") {
            base64_decode(payload)?
        } else {
            percent_decode(payload)?
        };
        let (local_path, local_ref) = write_cache(&state.data_dir, &key, &bytes, mime)?;
        return Ok(CoverResolveResult { local_path, local_ref });
    }

    // 缓存命中
    if let Some(path) = find_cached(&state.data_dir, &key) {
        let local_path = path.to_string_lossy().to_string();
        return Ok(CoverResolveResult {
            local_ref: format!("local://{}", local_path),
            local_path,
        });
    }

    // 缓存未命中：下载
    if !request.url.starts_with("http://") && !request.url.starts_with("https://") {
        return Err(CommandError::invalid(format!(
            "不支持的封面 URL: {}",
            &request.url.chars().take(80).collect::<String>()
        )));
    }
    let client = crate::booksource::engine::http_client();
    let mut builder = client.get(&request.url);
    if let Some(referer) = request.referer.as_deref().filter(|s| !s.is_empty()) {
        builder = builder.header(reqwest::header::REFERER, referer);
    }
    if let Some(headers) = request.headers.as_ref() {
        for (k, v) in headers {
            builder = builder.header(k, v);
        }
    }
    let resp = builder
        .send()
        .await
        .map_err(|e| CommandError::other(format!("封面下载失败: {}", e)))?;
    if !resp.status().is_success() {
        return Err(CommandError::other(format!("封面下载 HTTP {}", resp.status())));
    }
    let mime = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("image/jpeg")
        .to_string();
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| CommandError::other(format!("封面读取失败: {}", e)))?;
    let (local_path, local_ref) = write_cache(&state.data_dir, &key, &bytes, &mime)?;
    Ok(CoverResolveResult { local_path, local_ref })
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