//! 书源 JS 执行引擎（轻量版）。
//!
//! 不引入 Rust JS 引擎：把书源源码注入**主窗口**执行——
//! - 主窗口自带 `__TAURI_INTERNALS__`，注入的 `legado.http` shim 直接回调
//!   Rust 的 `booksource_http_proxy`（reqwest），绕开浏览器 CORS；
//! - 每个书源编译成一个模块缓存在 `window.__bsModules[fileName]`（IIFE
//!   隔离，避免不同书源的全局变量互相污染）；
//! - 结果走 `window.__probeResults[reqId]` + 轮询（复用 browser_probe 的
//!   eval 桥），异步 / await 天然支持。

use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State, WebviewWindow};
use uuid::Uuid;

use crate::booksource::commands::resolve_booksource_path;
use crate::browser_probe::{eval_value, poll_result};
use crate::errors::{CommandError, CommandResult};
use crate::state::AppState;

const BUILTIN_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

fn main_window(app: &AppHandle) -> CommandResult<WebviewWindow> {
    app.get_webview_window("main")
        .ok_or_else(|| CommandError::other("主窗口不存在".to_string()))
}

/// 确保书源模块已加载到主窗口，然后调用指定函数（candidates 取第一个存在的），
/// 轮询拿结果。fn_args 会序列化成 JS 数组字面量按顺序展开传入。
async fn run_source_fn(
    app: &AppHandle,
    state: &State<'_, AppState>,
    file_name: &str,
    source_dir: Option<&str>,
    fn_candidates: &[&str],
    fn_args: Vec<serde_json::Value>,
    timeout: Duration,
) -> CommandResult<serde_json::Value> {
    let candidates = serde_json::to_string(fn_candidates)
        .map_err(|e| CommandError::other(e.to_string()))?;
    let args =
        serde_json::to_string(&fn_args).map_err(|e| CommandError::other(e.to_string()))?;
    let tail = format!(
        r#"const candidates = {candidates};
    let fn;
    for (const n of candidates) {{ if (typeof mod[n] === 'function') {{ fn = mod[n]; break; }} }}
    if (!fn) throw new Error('书源缺少可用函数: ' + candidates.join('/'));
    const value = await fn.apply(mod, {args});
    window.__probeResults[REQ] = {{ ok: true, value: value === undefined ? null : value }};"#,
        candidates = candidates,
        args = args
    );
    run_in_source(app, state, file_name, source_dir, &tail, timeout).await
}

/// 确保书源模块已加载，然后在模块上下文里执行 tail_js（可用变量：REQ / fileName / mod），
/// 轮询结果槽位并解包 ok/error。
async fn run_in_source(
    app: &AppHandle,
    state: &State<'_, AppState>,
    file_name: &str,
    source_dir: Option<&str>,
    tail_js: &str,
    timeout: Duration,
) -> CommandResult<serde_json::Value> {
    let path = resolve_booksource_path(&state.data_dir, file_name, source_dir)?;
    if !path.exists() {
        return Err(CommandError::not_found(format!("书源文件不存在: {}", file_name)));
    }
    let source_code = std::fs::read_to_string(&path)?;

    let window = main_window(app)?;
    let req_id = Uuid::new_v4().simple().to_string();
    let bootstrap = build_bootstrap(&req_id, file_name, &source_code, tail_js)?;
    window
        .eval(bootstrap)
        .map_err(|e| CommandError::other(format!("注入书源脚本失败: {}", e)))?;

    let result = poll_result(&window, &req_id, timeout).await?;
    match result.get("ok").and_then(|v| v.as_bool()) {
        Some(true) => Ok(result.get("value").cloned().unwrap_or(serde_json::Value::Null)),
        _ => Err(CommandError::other(
            result
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("书源脚本执行失败")
                .to_string(),
        )),
    }
}

/// 生成"确保模块 + 执行 tail_js + 异常落结果槽位"的注入脚本。
/// tail_js 运行在 async 函数体内，可用变量：REQ（结果槽位 key）、fileName、mod。
fn build_bootstrap(
    req_id: &str,
    file_name: &str,
    source_code: &str,
    tail_js: &str,
) -> CommandResult<String> {
    let req = serde_json::to_string(req_id).map_err(|e| CommandError::other(e.to_string()))?;
    let file = serde_json::to_string(file_name).map_err(|e| CommandError::other(e.to_string()))?;
    let code =
        serde_json::to_string(source_code).map_err(|e| CommandError::other(e.to_string()))?;

    // 模块工厂：new Function('legado', 源码 + return 导出表)
    // —— IIFE 隔离全局；导出表覆盖书源约定的全部函数名。
    Ok(format!(
        r#"window.__probeResults = window.__probeResults || {{}};
window.__bsModules = window.__bsModules || {{}};
(async function() {{
  const REQ = {req};
  try {{
    const fileName = {file};
    if (!window.__bsModules[fileName]) {{
      const __invokeRaw = window.__TAURI_INTERNALS__.invoke;
      const __invoke = (cmd, args) => __invokeRaw(cmd, args).catch(e => {{
        throw new Error(e && e.message ? e.message : String(e));
      }});
      const legado = {{
        http: {{
          get: (url, headers) => __invoke('booksource_http_proxy', {{ request: {{ url, method: 'GET', headers: headers || {{}} }} }}).then(r => r.body),
          post: (url, body, headers) => __invoke('booksource_http_proxy', {{ request: {{ url, method: 'POST', body: body ?? null, headers: headers || {{}} }} }}).then(r => r.body),
          request: (request) => __invoke('booksource_http_proxy', {{ request }}),
        }},
      }};
      const factory = new Function('legado', {code} + '\n;return {{\n  search: typeof search === "function" ? search : undefined,\n  bookInfo: typeof bookInfo === "function" ? bookInfo : undefined,\n  toc: typeof toc === "function" ? toc : undefined,\n  chapterList: typeof chapterList === "function" ? chapterList : undefined,\n  content: typeof content === "function" ? content : undefined,\n  chapterContent: typeof chapterContent === "function" ? chapterContent : undefined,\n  explore: typeof explore === "function" ? explore : undefined\n}};');
      window.__bsModules[fileName] = factory(legado);
    }}
    const mod = window.__bsModules[fileName];
    {tail}
  }} catch (e) {{
    window.__probeResults[REQ] = {{ ok: false, error: String(e && (e.stack || e.message) || e) }};
  }}
}})();
"started""#,
        req = req,
        file = file,
        code = code,
        tail = tail_js
    ))
}

// ── 命令 ───────────────────────────────────────────────────────────────────

#[tauri::command(rename_all = "camelCase")]
pub async fn booksource_search(
    app: AppHandle,
    state: State<'_, AppState>,
    file_name: String,
    keyword: String,
    page: Option<i64>,
    source_dir: Option<String>,
) -> CommandResult<serde_json::Value> {
    run_source_fn(
        &app,
        &state,
        &file_name,
        source_dir.as_deref(),
        &["search"],
        vec![
            serde_json::Value::String(keyword),
            serde_json::json!(page.unwrap_or(1)),
        ],
        Duration::from_secs(40),
    )
    .await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn booksource_book_info(
    app: AppHandle,
    state: State<'_, AppState>,
    file_name: String,
    book_url: String,
    source_dir: Option<String>,
) -> CommandResult<serde_json::Value> {
    run_source_fn(
        &app,
        &state,
        &file_name,
        source_dir.as_deref(),
        &["bookInfo"],
        vec![serde_json::Value::String(book_url)],
        Duration::from_secs(40),
    )
    .await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn booksource_chapter_list(
    app: AppHandle,
    state: State<'_, AppState>,
    file_name: String,
    book_url: String,
    task_id: Option<String>,
    source_dir: Option<String>,
) -> CommandResult<serde_json::Value> {
    let _ = task_id; // 取消信号暂未实现
    run_source_fn(
        &app,
        &state,
        &file_name,
        source_dir.as_deref(),
        &["toc", "chapterList"],
        vec![serde_json::Value::String(book_url)],
        Duration::from_secs(120),
    )
    .await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn booksource_chapter_content(
    app: AppHandle,
    state: State<'_, AppState>,
    file_name: String,
    chapter_url: String,
    source_dir: Option<String>,
    category_params: Option<serde_json::Value>,
) -> CommandResult<serde_json::Value> {
    run_source_fn(
        &app,
        &state,
        &file_name,
        source_dir.as_deref(),
        &["content", "chapterContent"],
        vec![
            serde_json::Value::String(chapter_url),
            category_params.unwrap_or(serde_json::Value::Null),
        ],
        Duration::from_secs(40),
    )
    .await
}

#[tauri::command(rename_all = "camelCase")]
pub async fn booksource_explore(
    app: AppHandle,
    state: State<'_, AppState>,
    file_name: String,
    page: Option<i64>,
    category: Option<String>,
    no_cache: Option<bool>,
    source_dir: Option<String>,
) -> CommandResult<serde_json::Value> {
    let _ = no_cache; // 探索缓存暂未实现
    run_source_fn(
        &app,
        &state,
        &file_name,
        source_dir.as_deref(),
        &["explore"],
        vec![
            serde_json::json!(category.unwrap_or_default()),
            serde_json::json!(page.unwrap_or(1)),
        ],
        Duration::from_secs(40),
    )
    .await
}

/// 书源 eval（能力检测 / 调试）：
/// - entryCode 为空：返回书源定义的函数名列表（逗号分隔），
///   前端 detectCapabilities 用它判定 search/explore 能力；
/// - entryCode 非空：在书源模块作用域（with(mod)）内执行，返回结果字符串。
#[tauri::command(rename_all = "camelCase")]
pub async fn booksource_eval(
    app: AppHandle,
    state: State<'_, AppState>,
    file_name: String,
    entry_code: Option<String>,
    source_dir: Option<String>,
) -> CommandResult<String> {
    let tail = match &entry_code {
        None => {
            "window.__probeResults[REQ] = { ok: true, value: Object.keys(mod).filter(k => typeof mod[k] === 'function').join(',') };".to_string()
        }
        Some(code) => {
            let entry = serde_json::to_string(code)
                .map_err(|e| CommandError::other(e.to_string()))?;
            format!(
                "const __fn = new Function('module', 'with(module){{\\n' + {entry} + '\\n}}');\n    const __v = await __fn(mod);\n    window.__probeResults[REQ] = {{ ok: true, value: __v === undefined || __v === null ? '' : (typeof __v === 'string' ? __v : JSON.stringify(__v)) }};",
                entry = entry
            )
        }
    };
    let value = run_in_source(
        &app,
        &state,
        &file_name,
        source_dir.as_deref(),
        &tail,
        Duration::from_secs(18),
    )
    .await?;
    Ok(match value {
        serde_json::Value::String(s) => s,
        other => other.to_string(),
    })
}

// ── HTTP 代理（书源 JS 的 legado.http 走后端，绕开 CORS） ────────────────────
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpProxyRequest {
    pub url: String,
    pub method: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpProxyResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

pub(crate) fn http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .user_agent(BUILTIN_USER_AGENT)
            // 小说站证书质量参差，放宽校验换兼容性（与原版 legado 一致）
            .danger_accept_invalid_certs(true)
            .connect_timeout(Duration::from_secs(15))
            .timeout(Duration::from_secs(30))
            .build()
            .expect("reqwest client build")
    })
}

/// 按 content-type charset 解码；无 charset 时 UTF-8，失败回退 GBK（中文站常见）
fn decode_body(bytes: &[u8], content_type: Option<&str>) -> String {
    let charset = content_type.and_then(|ct| {
        ct.split(';').find_map(|part| {
            let part = part.trim();
            part.strip_prefix("charset=").map(|c| c.trim_matches('"'))
        })
    });
    if let Some(label) = charset {
        if let Some(enc) = encoding_rs::Encoding::for_label(label.as_bytes()) {
            let (text, _, _) = enc.decode(bytes);
            return text.into_owned();
        }
    }
    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => {
            let (text, _, _) = encoding_rs::GBK.decode(bytes);
            text.into_owned()
        }
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn booksource_http_proxy(request: HttpProxyRequest) -> CommandResult<HttpProxyResponse> {
    let method = request
        .method
        .as_deref()
        .unwrap_or("GET")
        .to_ascii_uppercase();
    let client = http_client();
    let mut builder = client.request(
        reqwest::Method::from_bytes(method.as_bytes())
            .map_err(|e| CommandError::invalid(format!("非法 HTTP 方法 {}: {}", method, e)))?,
        &request.url,
    );
    if let Some(headers) = request.headers {
        for (k, v) in headers {
            builder = builder.header(k, v);
        }
    }
    if let Some(body) = request.body {
        builder = builder.body(body);
    }
    let resp = builder
        .send()
        .await
        .map_err(|e| CommandError::other(format!("HTTP 请求失败: {}", e)))?;
    let status = resp.status().as_u16();
    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let headers: HashMap<String, String> = resp
        .headers()
        .iter()
        .filter_map(|(k, v)| v.to_str().ok().map(|s| (k.to_string(), s.to_string())))
        .collect();
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| CommandError::other(format!("读取响应体失败: {}", e)))?;
    Ok(HttpProxyResponse {
        status,
        headers,
        body: decode_body(&bytes, content_type.as_deref()),
    })
}
