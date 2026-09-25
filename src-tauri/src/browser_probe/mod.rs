//! 内嵌浏览器探测：每个会话是一个独立的 `WebviewWindow`（label 即 session id）。
//!
//! - 结果回传用 `eval_with_callback`（wry 原生通道），不依赖页面里的
//!   `__TAURI_INTERNALS__`，因此外部站点也能跑。
//! - 用户代码包在 `(async function(){ ... })()` 里执行，支持 `return` 和 await；
//!   结果先写到页面的 `window.__probeResults[reqId]`，Rust 侧轮询取出。
//! - cookie 读写落在共享 WebContext 上（与主窗口同一份存储）。
//! - 移动端不支持多窗口，`build()` 会失败并返回明确错误。

use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::Deserialize;
use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use uuid::Uuid;

use crate::errors::{CommandError, CommandResult};

#[derive(Default)]
pub struct BrowserProbeState {
    /// 存活会话（window label）
    sessions: Mutex<Vec<String>>,
    /// `browser_probe_set_user_agent` 保存的 UA，作用于之后创建的会话
    user_agent: Mutex<Option<String>>,
}

// ── 入参 ───────────────────────────────────────────────────────────────────

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOptions {
    pub visible: Option<bool>,
    pub user_agent: Option<String>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub timeout_secs: Option<u64>,
    pub timeout: Option<u64>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigateOptions {
    pub wait_until: Option<String>,
    pub wait_for: Option<String>,
    pub timeout_secs: Option<u64>,
    pub timeout: Option<u64>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvalOptions {
    pub timeout_secs: Option<u64>,
    pub timeout: Option<u64>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunOptions {
    pub visible: Option<bool>,
    pub user_agent: Option<String>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub wait_until: Option<String>,
    pub wait_for: Option<String>,
    pub timeout_secs: Option<u64>,
    pub timeout: Option<u64>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeCookie {
    pub name: String,
    pub value: String,
    pub domain: Option<String>,
    pub path: Option<String>,
    pub expires: Option<i64>,
    pub http_only: Option<bool>,
    pub secure: Option<bool>,
    pub same_site: Option<String>,
}

/// 与前端 `browserTimeoutMs` 一致：timeoutSecs ?? timeout ?? timeoutMs ?? default
fn timeout_of(secs: Option<u64>, t: Option<u64>, ms: Option<u64>, default: Duration) -> Duration {
    if let Some(s) = secs {
        Duration::from_secs(s)
    } else if let Some(s) = t {
        Duration::from_secs(s)
    } else if let Some(m) = ms {
        Duration::from_millis(m)
    } else {
        default
    }
}

fn get_window(app: &AppHandle, session_id: &str) -> CommandResult<WebviewWindow> {
    app.get_webview_window(session_id).ok_or_else(|| {
        CommandError::not_found(format!("探测会话不存在或已关闭: {}", session_id))
    })
}

/// 在页面里执行 JS，拿回 JSON 序列化后的结果
pub(crate) async fn eval_value(window: &WebviewWindow, js: String) -> CommandResult<serde_json::Value> {
    let (tx, rx) = tokio::sync::oneshot::channel::<String>();
    let tx = std::sync::Mutex::new(Some(tx));
    window
        .eval_with_callback(js, move |raw| {
            if let Some(tx) = tx.lock().expect("eval tx mutex").take() {
                let _ = tx.send(raw);
            }
        })
        .map_err(|e| CommandError::other(format!("eval 注入失败: {}", e)))?;
    let raw = rx
        .await
        .map_err(|_| CommandError::other("eval 回调被丢弃（窗口可能已关闭）".to_string()))?;
    Ok(serde_json::from_str(&raw).unwrap_or(serde_json::Value::String(raw)))
}

/// 轮询页面里的 `window.__probeResults[req_id]`，直到有值或超时
pub(crate) async fn poll_result(
    window: &WebviewWindow,
    req_id: &str,
    timeout: Duration,
) -> CommandResult<serde_json::Value> {
    let deadline = Instant::now() + timeout;
    let poll_js = format!(
        "JSON.stringify((window.__probeResults || {{}})[{}] ?? null)",
        serde_json::to_string(req_id).unwrap_or_else(|_| "\"\"".to_string())
    );
    loop {
        match eval_value(window, poll_js.clone()).await {
            Ok(serde_json::Value::String(s)) if s != "null" => {
                // 清掉结果槽位（fire-and-forget）
                let cleanup = format!(
                    "delete (window.__probeResults || {{}})[{}];",
                    serde_json::to_string(req_id).unwrap_or_else(|_| "\"\"".to_string())
                );
                let _ = window.eval(cleanup);
                let parsed: serde_json::Value = serde_json::from_str(&s).map_err(|e| {
                    CommandError::other(format!("探测结果解析失败: {}", e))
                })?;
                return Ok(parsed);
            }
            Ok(_) => {}
            Err(e) => return Err(e),
        }
        if Instant::now() >= deadline {
            return Err(CommandError::other(format!(
                "浏览器探测超时（{}s）",
                timeout.as_secs()
            )));
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

// ── 命令 ───────────────────────────────────────────────────────────────────

/// 创建探测会话（独立窗口，默认隐藏）。返回 sessionId（即窗口 label）。
#[tauri::command(rename_all = "camelCase")]
pub async fn browser_probe_create(
    app: AppHandle,
    state: State<'_, BrowserProbeState>,
    options: Option<CreateOptions>,
) -> CommandResult<String> {
    let opts = options.unwrap_or_default();
    let label = format!("probe-{}", Uuid::new_v4().simple());

    let blank = tauri::Url::parse("about:blank")
        .map_err(|e| CommandError::other(format!("内部 URL 解析失败: {}", e)))?;
    let mut builder = WebviewWindowBuilder::new(&app, &label, WebviewUrl::External(blank))
        .title("浏览器探测")
        .visible(opts.visible.unwrap_or(false));
    if let (Some(w), Some(h)) = (opts.width, opts.height) {
        builder = builder.inner_size(w, h);
    }
    let ua = opts
        .user_agent
        .filter(|s| !s.trim().is_empty())
        .or_else(|| state.user_agent.lock().expect("ua mutex").clone());
    if let Some(ua) = ua {
        builder = builder.user_agent(&ua);
    }
    builder.build().map_err(|e| {
        CommandError::other(format!(
            "创建探测窗口失败（移动端不支持多窗口）: {}",
            e
        ))
    })?;

    state.sessions.lock().expect("sessions mutex").push(label.clone());
    Ok(label)
}

/// 导航到 URL 并按 waitUntil 等待页面就绪
#[tauri::command(rename_all = "camelCase")]
pub async fn browser_probe_navigate(
    app: AppHandle,
    session_id: String,
    url: String,
    options: Option<NavigateOptions>,
) -> CommandResult<()> {
    let opts = options.unwrap_or_default();
    let window = get_window(&app, &session_id)?;
    let parsed = tauri::Url::parse(&url)
        .map_err(|e| CommandError::invalid(format!("非法 URL {}: {}", url, e)))?;
    window
        .navigate(parsed)
        .map_err(|e| CommandError::other(format!("导航失败: {}", e)))?;

    let wait_until = opts
        .wait_until
        .or(opts.wait_for)
        .unwrap_or_else(|| "load".to_string());
    let timeout = timeout_of(
        opts.timeout_secs,
        opts.timeout,
        opts.timeout_ms,
        Duration::from_secs(60),
    );
    let deadline = Instant::now() + timeout;
    loop {
        let ready = eval_value(&window, "document.readyState".to_string()).await;
        match ready {
            Ok(serde_json::Value::String(state)) => {
                let done = match wait_until.as_str() {
                    "domcontentloaded" => state == "interactive" || state == "complete",
                    // load / networkidle 都等 complete；networkidle 额外再静候 500ms
                    _ => state == "complete",
                };
                if done {
                    if wait_until == "networkidle" {
                        tokio::time::sleep(Duration::from_millis(500)).await;
                    }
                    return Ok(());
                }
            }
            // 导航进行中 eval 可能短暂失败，重试即可
            Ok(_) => {}
            Err(_) => {}
        }
        if Instant::now() >= deadline {
            return Err(CommandError::other(format!(
                "页面加载超时（{}s，waitUntil={}）",
                timeout.as_secs(),
                wait_until
            )));
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// 在会话页面里执行用户代码（async 函数体，支持 return / await），返回结果
#[tauri::command(rename_all = "camelCase")]
pub async fn browser_probe_eval(
    app: AppHandle,
    session_id: String,
    code: String,
    options: Option<EvalOptions>,
) -> CommandResult<serde_json::Value> {
    let opts = options.unwrap_or_default();
    let window = get_window(&app, &session_id)?;
    let timeout = timeout_of(
        opts.timeout_secs,
        opts.timeout,
        opts.timeout_ms,
        Duration::from_secs(55),
    );
    let req_id = Uuid::new_v4().simple().to_string();
    let req_js = serde_json::to_string(&req_id).unwrap_or_else(|_| "\"\"".to_string());
    let bootstrap = format!(
        "window.__probeResults = window.__probeResults || {{}};\n\
         (async function() {{\n\
         \x20 try {{\n\
         \x20   const __v = await (async function() {{\n{code}\n\x20   }})();\n\
         \x20   window.__probeResults[{req}] = {{ ok: true, value: __v === undefined ? null : __v }};\n\
         \x20 }} catch (__e) {{\n\
         \x20   window.__probeResults[{req}] = {{ ok: false, error: String(__e && __e.stack || __e) }};\n\
         \x20 }}\n\
         }})();\n\
         \"started\"",
        code = code,
        req = req_js
    );
    // 注入后立即开始轮询；eval 本身不等用户代码跑完
    window
        .eval(bootstrap)
        .map_err(|e| CommandError::other(format!("注入探测代码失败: {}", e)))?;
    let result = poll_result(&window, &req_id, timeout).await?;
    match result.get("ok").and_then(|v| v.as_bool()) {
        Some(true) => Ok(result.get("value").cloned().unwrap_or(serde_json::Value::Null)),
        _ => Err(CommandError::other(
            result
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("页面内执行失败")
                .to_string(),
        )),
    }
}

/// 一次性探测：建临时会话 → 导航 → 执行 → 关闭，返回结果
#[tauri::command(rename_all = "camelCase")]
pub async fn browser_probe_run(
    app: AppHandle,
    state: State<'_, BrowserProbeState>,
    url: String,
    code: String,
    options: Option<RunOptions>,
) -> CommandResult<serde_json::Value> {
    let opts = options.unwrap_or_default();
    let session_id = browser_probe_create(
        app.clone(),
        state,
        Some(CreateOptions {
            visible: opts.visible,
            user_agent: opts.user_agent,
            width: opts.width,
            height: opts.height,
            ..Default::default()
        }),
    )
    .await?;

    let app2 = app.clone();
    let result = async {
        browser_probe_navigate(
            app2.clone(),
            session_id.clone(),
            url,
            Some(NavigateOptions {
                wait_until: opts.wait_until,
                wait_for: opts.wait_for,
                timeout_secs: opts.timeout_secs,
                timeout: opts.timeout,
                timeout_ms: opts.timeout_ms,
            }),
        )
        .await?;
        browser_probe_eval(
            app2,
            session_id.clone(),
            code,
            Some(EvalOptions {
                timeout_secs: opts.timeout_secs,
                timeout: opts.timeout,
                timeout_ms: opts.timeout_ms,
            }),
        )
        .await
    }
    .await;

    // 无论成败都关掉临时会话
    if let Ok(window) = get_window(&app, &session_id) {
        let _ = window.destroy();
    }
    result
}

/// 读取 cookie（url 为空时返回全部）。读的是共享 WebContext 的存储。
#[tauri::command(rename_all = "camelCase")]
pub async fn browser_probe_get_cookies(
    app: AppHandle,
    state: State<'_, BrowserProbeState>,
    url: Option<String>,
) -> CommandResult<serde_json::Value> {
    let window = pick_cookie_window(&app, &state)?;
    let cookies = if let Some(u) = url {
        let parsed = tauri::Url::parse(&u)
            .map_err(|e| CommandError::invalid(format!("非法 URL {}: {}", u, e)))?;
        window
            .cookies_for_url(parsed)
            .map_err(|e| CommandError::other(format!("读取 cookie 失败: {}", e)))?
    } else {
        window
            .cookies()
            .map_err(|e| CommandError::other(format!("读取 cookie 失败: {}", e)))?
    };
    let list: Vec<serde_json::Value> = cookies
        .iter()
        .map(|c| {
            serde_json::json!({
                "name": c.name(),
                "value": c.value(),
                "domain": c.domain(),
                "path": c.path(),
                "expires": c.expires().and_then(|e| match e {
                    tauri::webview::cookie::Expiration::DateTime(dt) => Some(dt.unix_timestamp()),
                    tauri::webview::cookie::Expiration::Session => None,
                }),
                "httpOnly": c.http_only(),
                "secure": c.secure(),
                "sameSite": c.same_site().map(|s| format!("{:?}", s)),
            })
        })
        .collect();
    Ok(serde_json::Value::Array(list))
}

/// 写入单个 cookie（共享存储）
#[tauri::command(rename_all = "camelCase")]
pub async fn browser_probe_set_cookie(
    app: AppHandle,
    state: State<'_, BrowserProbeState>,
    url: String,
    cookie: ProbeCookie,
) -> CommandResult<()> {
    let parsed = tauri::Url::parse(&url)
        .map_err(|e| CommandError::invalid(format!("非法 URL {}: {}", url, e)))?;
    let window = pick_cookie_window(&app, &state)?;

    let mut builder = tauri::webview::cookie::Cookie::build((cookie.name, cookie.value));
    if let Some(d) = cookie.domain.filter(|s| !s.is_empty()) {
        builder = builder.domain(d);
    } else if let Some(host) = parsed.host_str() {
        builder = builder.domain(host.to_string());
    }
    builder = builder.path(cookie.path.unwrap_or_else(|| "/".to_string()));
    if let Some(true) = cookie.secure {
        builder = builder.secure(true);
    }
    if let Some(true) = cookie.http_only {
        builder = builder.http_only(true);
    }
    if let Some(exp) = cookie.expires {
        if let Ok(dt) = tauri::webview::cookie::time::OffsetDateTime::from_unix_timestamp(exp) {
            builder = builder.expires(dt);
        }
    }
    if let Some(same_site) = cookie.same_site.as_deref() {
        use tauri::webview::cookie::SameSite;
        let parsed_ss = match same_site.to_ascii_lowercase().as_str() {
            "strict" => Some(SameSite::Strict),
            "lax" => Some(SameSite::Lax),
            "none" => Some(SameSite::None),
            _ => None,
        };
        if let Some(ss) = parsed_ss {
            builder = builder.same_site(ss);
        }
    }
    window
        .set_cookie(builder.build())
        .map_err(|e| CommandError::other(format!("写入 cookie 失败: {}", e)))
}

/// 设置之后创建的探测会话的 UA（已存在的会话不受影响）
#[tauri::command(rename_all = "camelCase")]
pub async fn browser_probe_set_user_agent(
    state: State<'_, BrowserProbeState>,
    user_agent: String,
) -> CommandResult<()> {
    let mut ua = state.user_agent.lock().expect("ua mutex");
    *ua = if user_agent.trim().is_empty() {
        None
    } else {
        Some(user_agent)
    };
    Ok(())
}

/// 清数据：关闭全部探测会话 + 清空共享浏览数据（cookie / 缓存 / localStorage）。
/// 注意：应用自身的持久化都在文件里（frontend_storage / app_config），
/// 主窗口 localStorage 被清不影响书架与设置。
#[tauri::command(rename_all = "camelCase")]
pub async fn browser_probe_clear_data(
    app: AppHandle,
    state: State<'_, BrowserProbeState>,
) -> CommandResult<()> {
    close_all_sessions(&app, &state);
    if let Some(main) = app.get_webview_window("main") {
        main.clear_all_browsing_data()
            .map_err(|e| CommandError::other(format!("清理浏览数据失败: {}", e)))?;
    }
    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn browser_probe_show(app: AppHandle, session_id: String) -> CommandResult<()> {
    let window = get_window(&app, &session_id)?;
    window
        .show()
        .map_err(|e| CommandError::other(format!("show 失败: {}", e)))?;
    window
        .set_focus()
        .map_err(|e| CommandError::other(format!("focus 失败: {}", e)))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn browser_probe_hide(app: AppHandle, session_id: String) -> CommandResult<()> {
    let window = get_window(&app, &session_id)?;
    window
        .hide()
        .map_err(|e| CommandError::other(format!("hide 失败: {}", e)))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn browser_probe_close(
    app: AppHandle,
    state: State<'_, BrowserProbeState>,
    session_id: String,
) -> CommandResult<()> {
    state
        .sessions
        .lock()
        .expect("sessions mutex")
        .retain(|l| l != &session_id);
    if let Ok(window) = get_window(&app, &session_id) {
        let _ = window.destroy();
    }
    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn browser_probe_close_all(
    app: AppHandle,
    state: State<'_, BrowserProbeState>,
) -> CommandResult<()> {
    close_all_sessions(&app, &state);
    Ok(())
}

// ── 内部工具 ───────────────────────────────────────────────────────────────

/// cookie 读写任选一个窗口的 webview：优先探测会话，退到主窗口（共享同一存储）
fn pick_cookie_window(
    app: &AppHandle,
    state: &State<'_, BrowserProbeState>,
) -> CommandResult<WebviewWindow> {
    let first = state.sessions.lock().expect("sessions mutex").first().cloned();
    if let Some(label) = first {
        if let Some(w) = app.get_webview_window(&label) {
            return Ok(w);
        }
    }
    app.get_webview_window("main")
        .ok_or_else(|| CommandError::other("主窗口不存在".to_string()))
}

fn close_all_sessions(app: &AppHandle, state: &State<'_, BrowserProbeState>) {
    let labels: Vec<String> = std::mem::take(&mut *state.sessions.lock().expect("sessions mutex"));
    for label in labels {
        if let Some(w) = app.get_webview_window(&label) {
            let _ = w.destroy();
        }
    }
}
