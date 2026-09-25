use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

use crate::bookshelf::model::{
    now_ms, AddBookPayload, CachedChapter, ShelfBook, TxtChapterInput, UpdateShelfBookPayload,
};
use crate::bookshelf::storage;
use crate::errors::{CommandError, CommandResult};
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

/// 更新书架条目（局部字段可选）；同时可选同步章节目录。
///
/// JS 入参形状：`{ book: {...}, chapters: [...] | null, createSourceSwitchBackup?: bool }`
/// 当 `createSourceSwitchBackup = true` 时，在写盘前把当前 book + chapters
/// 快照到 `<dataDir>/bookshelf/source_switch_backup/<id>.json`，供
/// `bookshelf_restore_source_switch` 回滚。
#[tauri::command(rename_all = "camelCase")]
pub fn bookshelf_update_book(
    state: State<'_, AppState>,
    book: UpdateShelfBookPayload,
    chapters: Option<Vec<CachedChapter>>,
    create_source_switch_backup: Option<bool>,
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

    // 在写盘前快照：当前 book 是 *更新前* 的状态；chapters 用前端传过来的新目录
    // （保留的是切换前用过的目录）。
    if create_source_switch_backup.unwrap_or(false) {
        let snapshot = {
            let books = state.books.lock().expect("books mutex poisoned");
            let current = storage::find_book(&books, &book.id)?.clone();
            let current_chapters =
                storage::read_chapters(&state.data_dir, &book.id).unwrap_or_default();
            storage::SourceSwitchBackup {
                book: current,
                chapters: current_chapters,
                saved_at: now_ms(),
            }
        };
        storage::write_source_switch_backup(&state.data_dir, &book.id, &snapshot)?;
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

// ── P1 IPC：单本书查询 / 进度更新 / 导出 / 换源回滚 / 在文件管理器中打开 ─────

/// 获取单本书的完整 ShelfBook 记录。
///
/// JS 入参：`{ id: "..." }` → `ShelfBook`
#[tauri::command(rename_all = "camelCase")]
pub fn bookshelf_get(state: State<'_, AppState>, id: String) -> CommandResult<ShelfBook> {
    let books = state.books.lock().expect("books mutex poisoned");
    Ok(storage::find_book(&books, &id)?.clone())
}

/// 更新阅读进度。
///
/// JS 入参：`{ id, chapterIndex, chapterUrl, pageIndex?, scrollRatio?, playbackTime?, readerSettings? }`
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookshelfUpdateProgressArgs {
    pub id: String,
    pub chapter_index: i32,
    pub chapter_url: String,
    #[serde(default)]
    pub page_index: Option<i32>,
    #[serde(default)]
    pub scroll_ratio: Option<f32>,
    #[serde(default)]
    pub playback_time: Option<f32>,
    #[serde(default)]
    pub reader_settings: Option<String>,
}

#[tauri::command(rename_all = "camelCase")]
pub fn bookshelf_update_progress(
    state: State<'_, AppState>,
    args: BookshelfUpdateProgressArgs,
) -> CommandResult<()> {
    let mut books = state.books.lock().expect("books mutex poisoned");
    let target = storage::find_book_mut(&mut books, &args.id)?;
    let now = now_ms();
    target.read_chapter_index = args.chapter_index;
    target.read_chapter_url = Some(args.chapter_url);
    target.last_read_at = now;
    if let Some(p) = args.page_index {
        target.read_page_index = p;
    }
    if let Some(r) = args.scroll_ratio {
        target.read_scroll_ratio = r;
    }
    if let Some(t) = args.playback_time {
        target.read_playback_time = t;
    }
    if let Some(s) = args.reader_settings {
        target.reader_settings = Some(s);
    }
    storage::write_books(&state.data_dir, &books)
}

/// 恢复上一次"整本换源"前的书 + 章节快照（前端点击"恢复换源"时调用）。
/// 无快照时返回 NotFound。
///
/// JS 入参：`{ id: "..." }` → `{ book: ShelfBook, chapters: CachedChapter[] }`
#[tauri::command(rename_all = "camelCase")]
pub fn bookshelf_restore_source_switch(
    state: State<'_, AppState>,
    id: String,
) -> CommandResult<serde_json::Value> {
    let backup = storage::read_source_switch_backup(&state.data_dir, &id)?
        .ok_or_else(|| CommandError::not_found(format!("书 {} 没有换源快照", id)))?;
    Ok(serde_json::json!({
        "book": backup.book,
        "chapters": backup.chapters,
    }))
}

/// 在系统文件管理器中显示某本书的数据目录。
///
/// JS 入参：`{ id: "..." }`。桌面端用 std::process 调系统命令；
/// Android 端暂用 opener 命令（如果 Tauri 注册了），否则 no-op。
#[tauri::command(rename_all = "camelCase")]
pub fn bookshelf_reveal_data_dir(state: State<'_, AppState>, id: String) -> CommandResult<()> {
    let dir = storage::content_dir(&state.data_dir, &id);
    if !dir.exists() {
        return Err(CommandError::not_found(format!("书 {} 数据目录不存在", id)));
    }
    reveal_in_file_manager(&dir);
    Ok(())
}

/// 在系统文件管理器中显示导出文件（并选中它）。
///
/// JS 入参：`{ path: "..." }`
#[tauri::command(rename_all = "camelCase")]
pub fn bookshelf_reveal_export_file(path: String) -> CommandResult<()> {
    let p = std::path::PathBuf::from(&path);
    if !p.exists() {
        return Err(CommandError::not_found(format!("文件不存在: {}", path)));
    }
    reveal_in_file_manager(&p);
    Ok(())
}

// ── P1 IPC：导出（TXT 格式） ───────────────────────────────────────────────

/// 生成导出数据并以 base64 返回（移动端 / 跨端通用）。
///
/// JS 入参：`{ id: "...", format: "txt" | "epub" }` →
///   `{ fileName, mime, base64 }`
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookshelfExportBookDataArgs {
    pub id: String,
    pub format: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookshelfExportDataPayload {
    pub file_name: String,
    pub mime: String,
    pub base64: String,
}

#[tauri::command(rename_all = "camelCase")]
pub fn bookshelf_export_book_data(
    state: State<'_, AppState>,
    args: BookshelfExportBookDataArgs,
) -> CommandResult<BookshelfExportDataPayload> {
    let (book, chapters) = {
        let books = state.books.lock().expect("books mutex poisoned");
        let book = storage::find_book(&books, &args.id)?.clone();
        let chapters = storage::read_chapters(&state.data_dir, &args.id)?;
        (book, chapters)
    };
    let bytes = build_export_bytes(&state.data_dir, &book, &chapters, &args.format)?;
    let mime = match args.format.as_str() {
        "txt" => "text/plain;charset=utf-8",
        "epub" => "application/epub+zip",
        other => {
            return Err(CommandError::invalid(format!("不支持的导出格式: {}", other)));
        }
    };
    let ext = if args.format == "epub" { "epub" } else { "txt" };
    let file_name = sanitize_file_name(&book.name, ext);
    Ok(BookshelfExportDataPayload {
        file_name,
        mime: mime.to_string(),
        base64: bytes_to_base64(&bytes),
    })
}

/// 生成导出数据并直接写入指定路径（桌面端）。
///
/// JS 入参：`{ id: "...", format: "...", savePath: "..." }` → 实际保存路径
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookshelfExportBookArgs {
    pub id: String,
    pub format: String,
    pub save_path: String,
}

#[tauri::command(rename_all = "camelCase")]
pub fn bookshelf_export_book(
    state: State<'_, AppState>,
    args: BookshelfExportBookArgs,
) -> CommandResult<String> {
    let (book, chapters) = {
        let books = state.books.lock().expect("books mutex poisoned");
        let book = storage::find_book(&books, &args.id)?.clone();
        let chapters = storage::read_chapters(&state.data_dir, &args.id)?;
        (book, chapters)
    };
    let bytes = build_export_bytes(&state.data_dir, &book, &chapters, &args.format)?;
    let target = std::path::PathBuf::from(&args.save_path);
    if let Some(parent) = target.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    std::fs::write(&target, &bytes)?;
    Ok(target.to_string_lossy().to_string())
}

// ── 内部工具 ───────────────────────────────────────────────────────────────

/// 拼接纯文本导出：书名/作者 + 每章 `第N章 标题\n\n正文`
fn build_export_bytes(
    data_dir: &Path,
    book: &ShelfBook,
    chapters: &[CachedChapter],
    format: &str,
) -> CommandResult<Vec<u8>> {
    match format {
        "txt" => {
            let mut out = String::new();
            out.push_str(&book.name);
            if !book.author.is_empty() {
                out.push_str("  ");
                out.push_str(&book.author);
            }
            out.push_str("\n\n");
            for (idx, ch) in chapters.iter().enumerate() {
                out.push_str("第");
                out.push_str(&chinese_num(idx + 1));
                out.push_str("章 ");
                out.push_str(&ch.name);
                out.push_str("\n\n");
                let body =
                    std::fs::read_to_string(storage::content_file(data_dir, &book.id, ch.index))
                        .unwrap_or_default();
                out.push_str(body.trim());
                out.push_str("\n\n");
            }
            Ok(out.into_bytes())
        }
        "epub" => Err(CommandError::other(
            "EPUB 导出暂未实现，请先用 TXT 格式".to_string(),
        )),
        other => Err(CommandError::invalid(format!("不支持的导出格式: {}", other))),
    }
}

fn chinese_num(n: usize) -> String {
    let digits = ["零", "一", "二", "三", "四", "五", "六", "七", "八", "九"];
    if n < 10 {
        return digits[n].to_string();
    }
    if n < 20 {
        return format!("十{}", digits[n - 10]);
    }
    if n < 100 {
        let tens = n / 10;
        let ones = n % 10;
        let mut s = format!("{}{}", digits[tens], "十");
        if ones != 0 {
            s.push_str(digits[ones]);
        }
        return s;
    }
    n.to_string()
}

fn sanitize_file_name(name: &str, ext: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect();
    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
        return format!("export.{}", ext);
    }
    format!("{}.{}", trimmed, ext)
}

fn bytes_to_base64(bytes: &[u8]) -> String {
    let mut out = String::with_capacity((bytes.len() + 2) * 4 / 3);
    let chunks = bytes.chunks(3);
    for chunk in chunks {
        let b0 = chunk.first().copied().unwrap_or(0);
        let b1 = chunk.get(1).copied().unwrap_or(0);
        let b2 = chunk.get(2).copied().unwrap_or(0);
        let triple = ((b0 as u32) << 16) | ((b1 as u32) << 8) | (b2 as u32);
        out.push(BASE64_CHARS[((triple >> 18) & 0x3f) as usize] as char);
        out.push(BASE64_CHARS[((triple >> 12) & 0x3f) as usize] as char);
        match chunk.len() {
            3 => {
                out.push(BASE64_CHARS[((triple >> 6) & 0x3f) as usize] as char);
                out.push(BASE64_CHARS[(triple & 0x3f) as usize] as char);
            }
            2 => {
                out.push(BASE64_CHARS[((triple >> 6) & 0x3f) as usize] as char);
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

const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// 跨平台打开文件管理器并定位到目标路径
/// - Windows: explorer /select,"path"
/// - macOS: open -R "path"
/// - Linux: xdg-open "path"（或 dirname(path) 来"包含文件"）
/// - 其他：no-op
fn reveal_in_file_manager(path: &Path) {
    use std::process::Command;
    let result = if cfg!(target_os = "windows") {
        Command::new("explorer").arg(format!("/select,{}", path.display())).spawn()
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg("-R").arg(path).spawn()
    } else if cfg!(target_os = "linux") {
        // xdg-open 不能"选中文件"，只能打开目录；用 dirname 退一步
        let dir = path.parent().unwrap_or(path);
        Command::new("xdg-open").arg(dir).spawn()
    } else {
        // Android / iOS：暂不处理（前端按 isHarmonyNative 走自己的通道）
        return;
    };
    if let Err(err) = result {
        eprintln!("[reveal] 打开文件管理器失败: {}", err);
    }
}