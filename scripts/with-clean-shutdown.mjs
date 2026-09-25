#!/usr/bin/env node
// 包装任意子命令：仅在「被信号终止」时把退出码归零，主动失败原样传透。
// 解决 pnpm dev:desktop / dev:web / dev:android 关闭 Tauri 窗口时 vite 收到
// SIGTERM → exit 143 → pnpm 打印 "ELIFECYCLE Command failed with exit code 143"
// 的体感问题；端口占用等真错误仍会以非零码传透。
//
// 用法: node scripts/with-clean-shutdown.mjs <cmd> [args...]

import { spawn } from "node:child_process";

const [, , cmd, ...args] = process.argv;
if (!cmd) {
  console.error(
    "[with-clean-shutdown] usage: node scripts/with-clean-shutdown.mjs <cmd> [args...]",
  );
  process.exit(2);
}

const child = spawn(cmd, args, {
  stdio: "inherit",
  env: process.env,
});

const forward = (sig) => {
  if (!child.killed) child.kill(sig);
};
process.on("SIGTERM", () => forward("SIGTERM"));
process.on("SIGINT", () => forward("SIGINT"));

child.on("exit", (code, signal) => {
  process.exit(signal ? 0 : (code ?? 0));
});