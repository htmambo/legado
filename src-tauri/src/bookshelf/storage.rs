use std::path::{Path, PathBuf};

use crate::bookshelf::model::{CachedChapter, ShelfBook};
use crate::errors::{CommandError, CommandResult};

/// 书架目录名（位于 `<data_dir>/bookshelf/`）
pub const BOOKSHELF_DIR: &str = "bookshelf";
pub const BOOKS_FILE: &str = "books.json";
pub const CHAPTERS_SUBDIR: &str = "chapters";
pub const CONTENT_SUBDIR: &str = "content";

/// 工具函数：把所有路径都基于 data_dir 拼接
pub fn bookshelf_root(data_dir: &Path) -> PathBuf {
    data_dir.join(BOOKSHELF_DIR)
}

pub fn chapters_dir(data_dir: &Path) -> PathBuf {
    bookshelf_root(data_dir).join(CHAPTERS_SUBDIR)
}

pub fn chapters_file(data_dir: &Path, book_id: &str) -> PathBuf {
    chapters_dir(data_dir).join(format!("{}.json", sanitize_id(book_id)))
}

pub fn content_dir(data_dir: &Path, book_id: &str) -> PathBuf {
    bookshelf_root(data_dir).join(CONTENT_SUBDIR).join(sanitize_id(book_id))
}

pub fn content_file(data_dir: &Path, book_id: &str, index: i32) -> PathBuf {
    content_dir(data_dir, book_id).join(format!("{}.txt", index))
}

/// 书 id 在文件系统上要安全 —— 防止 `../../etc/passwd`
pub fn sanitize_id(id: &str) -> String {
    id.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

/// 读取 `books.json`，缺失或为空 → 返回空 Vec
pub fn read_books(data_dir: &Path) -> CommandResult<Vec<ShelfBook>> {
    let path = bookshelf_root(data_dir).join(BOOKS_FILE);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = std::fs::read_to_string(&path)?;
    if raw.trim().is_empty() {
        return Ok(Vec::new());
    }
    Ok(serde_json::from_str(&raw)?)
}

/// 原子写 `books.json`：先写临时文件再 rename，避免半截写入
pub fn write_books(data_dir: &Path, books: &[ShelfBook]) -> CommandResult<()> {
    let dir = bookshelf_root(data_dir);
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(BOOKS_FILE);
    let tmp = dir.join(format!("{}.tmp", BOOKS_FILE));
    let json = serde_json::to_string_pretty(books)?;
    std::fs::write(&tmp, json)?;
    std::fs::rename(&tmp, &path)?;
    Ok(())
}

/// 读取单本书的章节目录，缺失或空 → 返回空 Vec
pub fn read_chapters(data_dir: &Path, book_id: &str) -> CommandResult<Vec<CachedChapter>> {
    let path = chapters_file(data_dir, book_id);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = std::fs::read_to_string(&path)?;
    if raw.trim().is_empty() {
        return Ok(Vec::new());
    }
    Ok(serde_json::from_str(&raw)?)
}

/// 原子写单本书的章节目录
pub fn write_chapters(
    data_dir: &Path,
    book_id: &str,
    chapters: &[CachedChapter],
) -> CommandResult<()> {
    let dir = chapters_dir(data_dir);
    std::fs::create_dir_all(&dir)?;
    let path = chapters_file(data_dir, book_id);
    let tmp = dir.join(format!("{}.{}.tmp", sanitize_id(book_id), "chapters"));
    let json = serde_json::to_string_pretty(chapters)?;
    std::fs::write(&tmp, json)?;
    std::fs::rename(&tmp, &path)?;
    Ok(())
}

/// 单章正文写入（覆盖）。同时确保 content 目录存在。
pub fn write_chapter_content(
    data_dir: &Path,
    book_id: &str,
    index: i32,
    content: &str,
) -> CommandResult<()> {
    let dir = content_dir(data_dir, book_id);
    std::fs::create_dir_all(&dir)?;
    let path = content_file(data_dir, book_id, index);
    let tmp = dir.join(format!("{}.{}.tmp", index, "content"));
    std::fs::write(&tmp, content)?;
    std::fs::rename(&tmp, &path)?;
    Ok(())
}

/// 在书架内存缓存 + 磁盘上查找并返回指定 id 的书，找不到返回 NotFound
pub fn find_book<'a>(books: &'a [ShelfBook], id: &str) -> CommandResult<&'a ShelfBook> {
    books
        .iter()
        .find(|b| b.id == id)
        .ok_or_else(|| CommandError::not_found(format!("书 id {} 不存在", id)))
}

/// 在书架内存缓存 + 磁盘上查找并返回指定 id 的书的可变引用
pub fn find_book_mut<'a>(books: &'a mut [ShelfBook], id: &str) -> CommandResult<&'a mut ShelfBook> {
    books
        .iter_mut()
        .find(|b| b.id == id)
        .ok_or_else(|| CommandError::not_found(format!("书 id {} 不存在", id)))
}

// ── 单章正文 ───────────────────────────────────────────────────────────────

/// 读取单章正文；文件不存在返回 Ok(None)。
/// `local-txt://` 导入的章节正文就是用这个存的。
pub fn read_chapter_content(data_dir: &Path, book_id: &str, index: i32) -> CommandResult<Option<String>> {
    let path = content_file(data_dir, book_id, index);
    if !path.exists() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(&path)?;
    if raw.is_empty() {
        return Ok(None);
    }
    Ok(Some(raw))
}

/// 列举一本书已缓存的章节索引（content/<id>/*.txt）
pub fn list_cached_indices(data_dir: &Path, book_id: &str) -> CommandResult<Vec<i32>> {
    let dir = content_dir(data_dir, book_id);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if !name.ends_with(".txt") {
            continue;
        }
        if let Some(stem) = name.strip_suffix(".txt") {
            if let Ok(n) = stem.parse::<i32>() {
                out.push(n);
            }
        }
    }
    out.sort();
    Ok(out)
}

/// 删除单章正文缓存；文件不存在视为成功（幂等）。
pub fn delete_chapter_content(data_dir: &Path, book_id: &str, index: i32) -> CommandResult<()> {
    let path = content_file(data_dir, book_id, index);
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

// ── 集级播放进度（视频 / 有声书） ──────────────────────────────────────────

/// 集级进度 JSON 路径
pub fn episode_progress_file(data_dir: &Path, book_id: &str) -> std::path::PathBuf {
    bookshelf_root(data_dir)
        .join("meta")
        .join("episode_progress")
        .join(format!("{}.json", sanitize_id(book_id)))
}

/// 读取集级进度；文件不存在返回 Ok({})
pub fn read_episode_progress(
    data_dir: &Path,
    book_id: &str,
) -> CommandResult<std::collections::HashMap<String, serde_json::Value>> {
    let path = episode_progress_file(data_dir, book_id);
    if !path.exists() {
        return Ok(std::collections::HashMap::new());
    }
    let raw = std::fs::read_to_string(&path)?;
    if raw.trim().is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    Ok(serde_json::from_str(&raw)?)
}

/// 原子写集级进度
pub fn write_episode_progress(
    data_dir: &Path,
    book_id: &str,
    map: &std::collections::HashMap<String, serde_json::Value>,
) -> CommandResult<()> {
    let path = episode_progress_file(data_dir, book_id);
    let dir = path
        .parent()
        .ok_or_else(|| CommandError::other("episode_progress 路径无效"))?;
    std::fs::create_dir_all(dir)?;
    let tmp = dir.join(format!("{}.tmp", sanitize_id(book_id)));
    let json = serde_json::to_string_pretty(map)?;
    std::fs::write(&tmp, json)?;
    std::fs::rename(&tmp, &path)?;
    Ok(())
}