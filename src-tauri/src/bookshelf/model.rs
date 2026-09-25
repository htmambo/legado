use serde::{Deserialize, Serialize};

/// 一本已加入书架的书（与前端 `ShelfBook` 字段一一对应，camelCase）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShelfBook {
    pub id: String,
    pub name: String,
    pub author: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_referer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intro: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    pub book_url: String,
    pub file_name: String,
    pub source_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_chapter: Option<String>,
    pub added_at: i64,
    pub last_read_at: i64,
    pub read_chapter_index: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_chapter_url: Option<String>,
    pub total_chapters: i32,
    pub source_type: String,
    pub read_page_index: i32,
    pub read_scroll_ratio: f32,
    pub read_playback_time: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reader_settings: Option<String>,
    pub is_private: bool,
}

/// 加入书架时的入参（无 id、addedAt、进度等运行态字段）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddBookPayload {
    pub name: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub cover_url: Option<String>,
    #[serde(default)]
    pub intro: Option<String>,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub group_id: Option<String>,
    pub book_url: String,
    #[serde(default)]
    pub last_chapter: Option<String>,
    #[serde(default)]
    pub source_type: Option<String>,
}

/// 更新书架条目时的入参（id 必填，其余可选）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateShelfBookPayload {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub cover_url: Option<String>,
    #[serde(default)]
    pub intro: Option<String>,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub group_id: Option<String>,
    pub book_url: String,
    pub file_name: String,
    pub source_name: String,
    #[serde(default)]
    pub last_chapter: Option<String>,
    #[serde(default)]
    pub total_chapters: Option<i32>,
    #[serde(default)]
    pub read_chapter_index: Option<i32>,
    #[serde(default)]
    pub read_chapter_url: Option<String>,
    pub source_type: String,
    #[serde(default)]
    pub added_at: Option<i64>,
    #[serde(default)]
    pub last_read_at: Option<i64>,
    #[serde(default)]
    pub read_page_index: Option<i32>,
    #[serde(default)]
    pub read_scroll_ratio: Option<f32>,
    #[serde(default)]
    pub read_playback_time: Option<f32>,
    #[serde(default)]
    pub reader_settings: Option<String>,
    #[serde(default)]
    pub is_private: Option<bool>,
}

/// 单章节的目录项（与前端 `CachedChapter` 一致）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CachedChapter {
    pub index: i32,
    pub name: String,
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
}

/// `bookshelf_save_txt_chapters` 的入参元素
#[derive(Debug, Clone, Deserialize)]
pub struct TxtChapterInput {
    pub index: i32,
    pub content: String,
}

/// 集级播放进度（视频 / 有声书）。与前端 `EpisodeProgress` 字段一一对应。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpisodeProgress {
    pub time: f64,
    pub duration: f64,
    pub last_played_at: i64,
}

/// 现在时间（毫秒）—— 简单封装，避免引入 chrono 依赖
pub fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}