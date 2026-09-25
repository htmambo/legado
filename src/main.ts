// [BOOT] 前端入口打点：记录 JS 开始执行的时间，用于切割 Android 52s 启动空档
const _bootT0 = Date.now();
console.log(`[BOOT][Frontend] main.ts 开始执行 t=${_bootT0}`);
window.__LEGADO_SET_BOOT_STAGE?.("main-ts-started");

import naive from "naive-ui";
import { createPinia } from "pinia";
import { createApp } from "vue";
import App from "./App.vue";
import "./style.css";
import "./styles/tokens.css";
import "./styles/theme.css";
import "./styles/base.css";
import "./styles/responsive.css";
import "./styles/reader.css";
import "./styles/focus.css";
import "./styles/remote.css";
import "./styles/components.css";
import { initFrontendStorage } from "./composables/useFrontendStorage";
import { warmupWebView } from "./composables/useWebViewWarmup";
import { precacheUrls } from "./composables/useCacheStrategy";
import { analyticsPlugin } from "./plugins/analytics";

(function initFontScale() {
  const scale = localStorage.getItem("legado-ui-font-scale") || "medium";
  const root = document.documentElement;
  root.setAttribute("data-font-scale", scale);
  const scaleMap: Record<string, string> = { small: "0.875", medium: "1", large: "1.125" };
  root.style.setProperty("--ui-font-scale", scaleMap[scale] || "1");
})();

// Naive UI 已通过 unplugin-vue-components 按需自动导入，无需全量 app.use(naive)
const app = createApp(App);
app.use(createPinia());
app.use(analyticsPlugin);
app.config.errorHandler = (err, instance, info) => {
  const details = err instanceof Error ? (err.stack ?? err.message) : String(err);
  console.error("[BOOT][Frontend] Vue error", { err, info, instance });
  window.__LEGADO_SHOW_BOOT_ERROR?.(`Vue 渲染异常 (${info}):\n${details}`);
};
// 挂载后记录首屏到达时间，并移除骨架屏
app.use(naive);

// 在挂载前从后端预取所有持久化数据，确保 useDynamicConfig 的 ready 可立即 resolve，
// 不再依赖 localStorage 作为同步备份，彻底消除脏数据干扰。
await initFrontendStorage();
console.log(`[BOOT][Frontend] 前端存储预取完成 cost=${Date.now() - _bootT0}ms`);

try {
  app.mount("#app");
  window.__LEGADO_SET_BOOT_STAGE?.("app-mounted");
  console.log(`[BOOT][Frontend] App 挂载完成 cost=${Date.now() - _bootT0}ms`);
} catch (err) {
  const details = err instanceof Error ? (err.stack ?? err.message) : String(err);
  window.__LEGADO_SHOW_BOOT_ERROR?.(`App 挂载失败:\n${details}`);
  throw err;
}

// 隐藏首屏骨架屏（过渡动画后移除）；但如果 boot 阶段已经累积了错误，
// 就保留覆盖层，让用户能看到诊断信息。
const skeleton = document.getElementById("app-skeleton");
if (skeleton) {
  const hasBootError = window.__LEGADO_HAS_BOOT_ERROR?.() === true;
  if (hasBootError) {
    console.warn(
      "[BOOT][Frontend] 启动期已捕获异常，骨架屏/错误覆盖层保持显示，由用户手动关闭",
    );
  } else {
    skeleton.classList.add("hidden");
    const removeSkeleton = () => {
      if (skeleton.parentNode) {
        skeleton.remove();
      }
    };
    skeleton.addEventListener("transitionend", removeSkeleton, { once: true });
    // Android WebView 有时不触发 transitionend，500ms 后强制移除
    setTimeout(removeSkeleton, 500);
  }
}

warmupWebView();

const CRITICAL_PRECACHE_URLS = ["/assets/booksource-default.svg"];

if (typeof requestIdleCallback !== "undefined") {
  requestIdleCallback(() => precacheUrls(CRITICAL_PRECACHE_URLS), { timeout: 5000 });
} else {
  setTimeout(() => precacheUrls(CRITICAL_PRECACHE_URLS), 3000);
}
