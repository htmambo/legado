<script setup lang="ts">
import {
  useMessage,
  type MessageApi,
  type MessageOptions,
  type MessageReactive,
} from "naive-ui";
import { isHarmonyNative } from "@/composables/useEnv";
import { invokeWithTimeout } from "@/composables/useInvoke";
import { useScriptBridgeStore } from "@/stores";

type MessageLevel = "success" | "error" | "warning" | "info";

type MirrorPatchedApi = MessageApi & {
  __legadoMirrorInstalled__?: boolean;
};

/** 各 level 的默认持续时间。错误停留更久，便于用户看清。 */
const DURATION_BY_LEVEL: Record<MessageLevel, number> = {
  success: 3000,
  info: 4000,
  warning: 10000,
  error: 30000,
};

/** 错误/警告需要可关闭，普通提示不强求 */
const CLOSABLE_BY_LEVEL: Record<MessageLevel, boolean> = {
  success: false,
  info: false,
  warning: true,
  error: true,
};

function normalizeContent(content: unknown): string {
  if (typeof content === "string") {
    return content;
  }
  if (content instanceof Error) {
    return content.stack || content.message;
  }
  if (typeof content === "number" || typeof content === "boolean") {
    return String(content);
  }
  if (content && typeof content === "object") {
    const maybeContent = (content as Record<string, unknown>).content;
    if (typeof maybeContent === "string") {
      return maybeContent;
    }
    try {
      return JSON.stringify(content);
    } catch {
      return String(content);
    }
  }
  return String(content ?? "");
}

/**
 * 把每条 UI 提示同步输出到：
 *  1. console（devtools 直接看，按 level 染色）
 *  2. `<appDataDir>/frontend.log`（Rust 端 frontend_log 命令）
 *  3. 内置 log window（已有的 appendDebugLog 通道，命令未注册时降级到这）
 *
 * 注意：仅在用户调用 message.error/warning/info/success 时触发，setup 期间不会跑。
 */
function mirrorPrompt(level: MessageLevel, content: unknown): void {
  const text = normalizeContent(content).trim();
  if (!text) {
    return;
  }

  // 1. console 染色
  const consoleMethod =
    level === "error"
      ? console.error
      : level === "warning"
        ? console.warn
        : console.info;
  consoleMethod(`[UI][${level}]`, text);

  // 2. 转发到 Rust 端（失败时降级）
  invokeWithTimeout("frontend_log", { level, message: text }, 3000).catch((error) => {
    const fallbackLevel = level === "error" ? "ERROR" : level === "warning" ? "WARN" : "INFO";
    useScriptBridgeStore().appendDebugLog(
      `[UI][${fallbackLevel}][mirror-fallback][${level}] ${text}`,
      "app",
    );
    console.warn("[GlobalFeedbackMirror] 后端日志转发失败，已回退本地日志:", error);
  });

  // 3. Harmony 原生壳额外走一份 console
  if (isHarmonyNative) {
    const line = `[PromptMirror][${level}] ${text}`;
    switch (level) {
      case "error":
        console.error(line);
        break;
      case "warning":
        console.warn(line);
        break;
      default:
        console.info(line);
        break;
    }
  }
}

function patchMethod(message: MirrorPatchedApi, method: MessageLevel): void {
  const original = message[method];
  if (typeof original !== "function") {
    return;
  }

  const wrapped = ((content: unknown, options?: MessageOptions): MessageReactive => {
    mirrorPrompt(method, content);

    const opts: MessageOptions = {
      duration: DURATION_BY_LEVEL[method],
      closable: CLOSABLE_BY_LEVEL[method],
      ...options,
    };

    return original.call(message, content as never, opts);
  }) as typeof original;

  Object.assign(wrapped, original);
  message[method] = wrapped;
}

const message = useMessage() as MirrorPatchedApi;
if (!message.__legadoMirrorInstalled__) {
  message.__legadoMirrorInstalled__ = true;

  patchMethod(message, "success");
  patchMethod(message, "error");
  patchMethod(message, "warning");
  patchMethod(message, "info");
}
</script>

<template></template>