use std::collections::HashMap;

use serde::Deserialize;
use tauri::State;
use uuid::Uuid;

use crate::bookshelf::model::{
    now_ms, AddBookPayload, CachedChapter, ShelfBook, TxtChapterInput, UpdateShelfBookPayload,
};
use crate::bookshelf::storage;
use crate::errors::CommandResult;
use crate::state::AppState;

// 注意：所有 #[tauri::command] 都加了 `rename_all = "camelCase"`——
// 配合前端发来的 camelCase JSON（`book` / `fileName` / `sourceName` 等），
// 同时保留 Rust 端惯用的 snake_case 形参名。

// ── 命令 ───────────────────────────────────────────────────────────────────

/// 列出所有已加入书架的书
#[tauri::command(rename_all = "camelCase")]
pub fn bookshelf_list(state: State<'_, AppState>) -> CommandResult<Vec<ShelfBook>> {
    let books = state.books.lock().expect("books mutex poisoned");
    Ok(books.clone())
}

/// 加入书架：分配 UUID + 初始化进度字段 + 落盘 + 内存缓存
///
/// JS 入参形状：`{ book: {...}, fileName: "x", sourceName: "y" }`
#[tauri::command(rename_all = "camelCase")]
pub fn bookshelf_add(
    state: State<'_, AppState>,
    book: AddBookPayload,
    file_name: String,
    source_name: String,
) -> CommandResult<ShelfBook> {
    let now = now_ms();
    let shelf_book = ShelfBook {
        id: Uuid::new_v4().to_string(),
        name: book.name,
        author: book.author.unwrap_or_default(),
        cover_url: book.cover_url,
        cover_referer: None,
        intro: book.intro,
        kind: book.kind,
        group_id: book.group_id,
        book_url: book.book_url,
        file_name,
        source_name,
        last_chapter: book.last_chapter,
        added_at: now,
        last_read_at: 0,
        read_chapter_index: -1,
        read_chapter_url: None,
        total_chapters: 0,
        source_type: book.source_type.unwrap_or_else(|| "novel".to_string()),
        read_page_index: -1,
        read_scroll_ratio: -1.0,
        read_playback_time: -1.0,
        reader_settings: None,
        is_private: false,
    };

    // 内存 + 落盘
    {
        let mut books = state.books.lock().expect("books mutex poisoned");
        books.push(shelf_book.clone());
        storage::write_books(&state.data_dir, &books)?;
    }

    // 为这本书预建章节目录文件（空数组起步）
    storage::write_chapters(&state.data_dir, &shelf_book.id, &[])?;

    Ok(shelf_book)
}

/// 覆盖式保存单本书的章节目录
///
/// JS 入参形状：`{ id: "...", chapters: [...] }`
#[tauri::command(rename_all = "camelCase")]
pub fn bookshelf_save_chapters(
    state: State<'_, AppState>,
    id: String,
    chapters: Vec<CachedChapter>,
) -> CommandResult<()> {
    {
        let books = state.books.lock().expect("books mutex poisoned");
        storage::find_book(&books, &id)?;
    }
    storage::write_chapters(&state.data_dir, &id, &chapters)
}

/// 批量写入章节正文（TXT 导入专用）
///
/// JS 入参形状：`{ id: "...", chapters: [{ index, content }, ...] }`
#[tauri::command(rename_all = "camelCase")]
pub fn bookshelf_save_txt_chapters(
    state: State<'_, AppState>,
    id: String,
    chapters: Vec<TxtChapterInput>,
) -> CommandResult<()> {
    {
        let books = state.books.lock().expect("books mutex poisoned");
        storage::find_book(&books, &id)?;
    }
    for ch in &chapters {
        storage::write_chapter_content(&state.data_dir, &id, ch.index, &ch.content)?;
    }
    Ok(())
}

/// 更新书架条目（局部字段可选）；同时可选同步章节目录
///
/// JS 入参形状：`{ book: {...}, chapters: [...] | null }`
#[tauri::command(rename_all = "camelCase")]
pub fn bookshelf_update_book(
    state: State<'_, AppState>,
    book: UpdateShelfBookPayload,
    chapters: Option<Vec<CachedChapter>>,
) -> CommandResult<()> {
    {
        let mut books = state.books.lock().expect("books mutex poisoned");
        let target = storage::find_book_mut(&mut books, &book.id)?;

        // 必填字段：直接覆盖（前端传回的就是当前值，幂等）
        target.book_url = book.book_url.clone();
        target.file_name = book.file_name.clone();
        target.source_name = book.source_name.clone();
        target.source_type = book.source_type.clone();

        // 仅覆盖前端实际传入的可选字段；缺省保留旧值
        if let Some(v) = book.name.clone() {
            target.name = v;
        }
        if let Some(v) = book.author.clone() {
            target.author = v;
        }
        if let Some(v) = book.cover_url.clone() {
            target.cover_url = Some(v);
        }
        if let Some(v) = book.intro.clone() {
            target.intro = Some(v);
        }
        if let Some(v) = book.kind.clone() {
            target.kind = Some(v);
        }
        if let Some(v) = book.group_id.clone() {
            target.group_id = Some(v);
        }
        if let Some(v) = book.last_chapter.clone() {
            target.last_chapter = Some(v);
        }
        if let Some(v) = book.total_chapters {
            target.total_chapters = v;
        }
        if let Some(v) = book.read_chapter_index {
            target.read_chapter_index = v;
        }
        if let Some(v) = book.read_chapter_url.clone() {
            target.read_chapter_url = Some(v);
        }
        if let Some(v) = book.added_at {
            target.added_at = v;
        }
        if let Some(v) = book.last_read_at {
            target.last_read_at = v;
        }
        if let Some(v) = book.read_page_index {
            target.read_page_index = v;
        }
        if let Some(v) = book.read_scroll_ratio {
            target.read_scroll_ratio = v;
        }
        if let Some(v) = book.read_playback_time {
            target.read_playback_time = v;
        }
        if let Some(v) = book.reader_settings.clone() {
            target.reader_settings = Some(v);
        }
        if let Some(v) = book.is_private {
            target.is_private = v;
        }

        storage::write_books(&state.data_dir, &books)?;
    }

    if let Some(chapters) = chapters.as_ref() {
        storage::write_chapters(&state.data_dir, &book.id, chapters)?;
    }
    Ok(())
}

// ── 单章正文缓存 ───────────────────────────────────────────────────────────

/// 读取单本书的章节目录（书架打开书籍时第一时间调用）
///
/// JS 入参：`{ id: "..." }`
#[tauri::command(rename_all = "camelCase")]
pub fn bookshelf_get_chapters(
    state: State<'_, AppState>,
    id: String,
) -> CommandResult<Vec<CachedChapter>> {
    // 这里不强制要求书在 books.json 里——某些书源可能先缓存章节再登记。
    storage::read_chapters(&state.data_dir, &id)
}

/// 读取单章缓存正文；文件不存在返回 Ok(None)。
///
/// JS 入参：`{ id: "...", chapterIndex: 0 }`
#[tauri::command(rename_all = "camelCase")]
pub fn bookshelf_get_content(
    state: State<'_, AppState>,
    id: String,
    chapter_index: i32,
) -> CommandResult<Option<String>> {
    storage::read_chapter_content(&state.data_dir, &id, chapter_index)
}

/// 写入单章缓存正文（覆盖）。
///
/// JS 入参：`{ id: "...", chapterIndex: 0, content: "..." }`
#[tauri::command(rename_all = "camelCase")]
pub fn bookshelf_save_content(
    state: State<'_, AppState>,
    id: String,
    chapter_index: i32,
    content: String,
) -> CommandResult<()> {
    storage::write_chapter_content(&state.data_dir, &id, chapter_index, &content)
}

/// 删除单章缓存正文（幂等）。
///
/// JS 入参：`{ id: "...", chapterIndex: 0 }`
#[tauri::command(rename_all = "camelCase")]
pub fn bookshelf_delete_content(
    state: State<'_, AppState>,
    id: String,
    chapter_index: i32,
) -> CommandResult<()> {
    storage::delete_chapter_content(&state.data_dir, &id, chapter_index)
}

/// 列出已缓存正文的章节索引集合。
///
/// JS 入参：`{ id: "..." }` → `number[]`
#[tauri::command(rename_all = "camelCase")]
pub fn bookshelf_get_cached_indices(
    state: State<'_, AppState>,
    id: String,
) -> CommandResult<Vec<i32>> {
    storage::list_cached_indices(&state.data_dir, &id)
}

// ── 集级播放进度（视频 / 有声书） ──────────────────────────────────────────

/// 读取单本书的集级播放进度；文件不存在返回 Ok({})
///
/// JS 入参：`{ id: "..." }` → `Record<chapterUrl, EpisodeProgress>`
#[tauri::command(rename_all = "camelCase")]
pub fn bookshelf_get_episode_progress(
    state: State<'_, AppState>,
    id: String,
) -> CommandResult<HashMap<String, crate::bookshelf::model::EpisodeProgress>> {
    // EpisodeProgress 是结构化对象；但底层 JSON 仍按 JSON Value 读写以便后续扩展字段。
    let raw: HashMap<String, serde_json::Value> = storage::read_episode_progress(&state.data_dir, &id)?;
    let mut out = HashMap::new();
    for (k, v) in raw {
        match serde_json::from_value::<crate::bookshelf::model::EpisodeProgress>(v) {
            Ok(ep) => {
                out.insert(k, ep);
            }
            Err(_) => {
                // 忽略无法反序列化的旧条目，向前兼容
            }
        }
    }
    Ok(out)
}

/// 保存单集播放进度（合并写入整个进度 map）。
///
/// JS 入参：`{ id: "...", chapterUrl: "...", time: 12.3, duration: 600 }`
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookshelfSaveEpisodeProgressArgs {
    pub id: String,
    pub chapter_url: String,
    pub time: f64,
    pub duration: f64,
}

#[tauri::command(rename_all = "camelCase")]
pub fn bookshelf_save_episode_progress(
    state: State<'_, AppState>,
    id: String,
    chapter_url: String,
    time: f64,
    duration: f64,
) -> CommandResult<()> {
    let mut map = storage::read_episode_progress(&state.data_dir, &id)?;
    let ep = crate::bookshelf::model::EpisodeProgress {
        time,
        duration,
        last_played_at: now_ms(),
    };
    map.insert(chapter_url, serde_json::to_value(ep)?);
    storage::write_episode_progress(&state.data_dir, &id, &map)
}