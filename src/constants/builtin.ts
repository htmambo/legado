/**
 * 跨前端 / 后端的内置常量（避免循环依赖：useAppConfig ↔ stores/appConfig）。
 */

/** 内置默认 User-Agent（与 Rust BUILTIN_USER_AGENT 保持一致） */
export const BUILTIN_USER_AGENT =
  "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 " +
  "(KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";