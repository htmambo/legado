use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use crate::errors::{CommandError, CommandResult};
use crate::state::AppState;

// ── 前端 PrefetchPayload 类型映射 ────────────────────────────────────────────
//
// 与 src/stores/prefetch.ts:PrefetchPayload 字段一一对应。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrefetchChapterInfo {
    pub index: i32,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrefetchPayload {
    pub id: String,
    pub file_name: String,
    pub book_url: String,
    pub book_name: String,
    pub source_type: String,
    pub chapters: Vec<PrefetchChapterInfo>,
    pub start_index: i32,
    pub count: i32,
    #[serde(default)]
    pub concurrency: Option<u32>,
    pub task_id: String,
}

// ── 前端 PrefetchProgressPayload 映射（用于 emit 事件） ───────────────────────

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PrefetchProgressPayload {
    task_id: String,
    done: u32,
    total: u32,
    chapter_index: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PrefetchDonePayload {
    task_id: String,
}

// ── Tauri 命令 ─────────────────────────────────────────────────────────────

/// 批量预取章节正文。
///
/// **当前状态**：P0 阶段**没有 booksource JS 引擎**，无法实际跑 `bookInfo` /
/// `chapterList` 脚本拉取章节正文。本命令接受任务，立即 emit 一个 `done=0` 的
/// `shelf:prefetch-progress` 事件，再 emit `shelf:prefetch-done`，让前端 UI
/// 干净退出"缓存中"状态。等 booksource 引擎就绪后（P2 时）替换为真实现。
///
/// JS 入参：`{ payload: PrefetchPayload }`
#[tauri::command(rename_all = "camelCase")]
pub fn bookshelf_prefetch_chapters(
    app: AppHandle,
    state: State<'_, AppState>,
    payload: PrefetchPayload,
) -> CommandResult<()> {
    let total = payload.chapters.len() as u32;

    // 立刻 emit 一次 0/total 的进度事件，再 emit done，让前端 prefetch 状态归零
    let _ = app.emit(
        "shelf:prefetch-progress",
        PrefetchProgressPayload {
            task_id: payload.task_id.clone(),
            done: 0,
            total,
            chapter_index: payload.start_index,
            error: Some("booksource 引擎尚未实现，本次未缓存任何章节".to_string()),
        },
    );
    let _ = app.emit(
        "shelf:prefetch-done",
        PrefetchDonePayload {
            task_id: payload.task_id,
        },
    );

    // 数据已经在本地导入阶段或 bookshelf_get_content 走过了；这里只触发事件。
    let _ = state; // 当前实现没用 State，但保留参数以保持命令签名稳定
    Ok(())
}

/// 取消一个预取任务。
///
/// **当前状态**：bookshelf_prefetch_chapters 立即完成 / 不会真跑后台任务，
/// 所以这里实质是 no-op。保留这个命令供真实引擎就绪后接入任务取消逻辑。
///
/// JS 入参：`{ taskId: "..." }`
#[tauri::command(rename_all = "camelCase")]
pub fn booksource_cancel(task_id: String) -> CommandResult<()> {
    // 当前没有运行中的任务池。引擎就绪后这里需要从全局 TaskMap 中移除 task_id。
    let _ = task_id;
    Ok(())
}

// ── 鸿蒙专用：导出文件选择 ───────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PickSavePathArgs {
    pub default_name: String,
    #[serde(default)]
    pub filter_name: Option<String>,
    #[serde(default)]
    pub filter_exts: Vec<String>,
}

/// 弹一个系统保存路径选择器，返回用户选定的绝对路径；取消返回 null。
///
/// **当前状态**：桌面端前端走 `@tauri-apps/plugin-dialog.save()` 直连，本命令
/// 不会被调到。鸿蒙 native (`isHarmonyNative`) 才走这里。鸿蒙集成暂未实现，
/// 返回明确错误而不是 unhandledrejection。
///
/// JS 入参：`{ defaultName, filterName?, filterExts: string[] }`
#[tauri::command(rename_all = "camelCase")]
pub fn bookshelf_pick_save_path(args: PickSavePathArgs) -> CommandResult<Option<String>> {
    let _ = args;
    Err(CommandError::other(
        "bookshelf_pick_save_path 暂未实现：Harmony 系统选择器集成待补".to_string(),
    ))
}