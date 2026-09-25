use std::io::Write;
use std::path::Path;

use tauri::State;

use crate::bookshelf::model::now_ms;
use crate::state::AppState;

/// 前端调用：把用户在 UI 上的提示/错误转发到 Rust 侧日志
///
/// 写入 `<data_dir>/frontend.log`，每行一条 JSON（避免多线程写入交叠）。
/// 写入失败时静默吞掉：日志通道挂了不能反过来阻塞 UI。
///
/// JS 入参形状：`{ level: "info" | "error" | ..., message: "..." }`
#[tauri::command(rename_all = "camelCase")]
pub fn frontend_log(state: State<'_, AppState>, level: String, message: String) {
    if let Err(err) = append_log(&state.data_dir, &level, &message) {
        eprintln!("[frontend_log] 写入失败: {}", err);
    }
}

fn append_log(data_dir: &Path, level: &str, message: &str) -> std::io::Result<()> {
    let path = data_dir.join("frontend.log");
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    let ts = now_ms();
    // 单行 JSON：避免换行/转义污染日志结构
    let escaped = message
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r");
    let line = format!(
        "{{\"ts\":{},\"level\":\"{}\",\"message\":\"{}\"}}\n",
        ts,
        level.replace('"', "\\\""),
        escaped,
    );
    file.write_all(line.as_bytes())?;
    Ok(())
}