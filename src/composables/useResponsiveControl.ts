import { computed, onMounted, onUnmounted, ref } from "vue";
import { isTauri } from "./useEnv";

export type ResponsiveBreakpoint = "compact" | "medium" | "expanded" | "wide";
export type DensityScale = 0.875 | 1 | 1.125;
export type DensityMode = "compact" | "standard" | "comfortable";

const DENSITY_SCALE_MAP: Record<DensityMode, DensityScale> = {
  compact: 0.875,
  standard: 1,
  comfortable: 1.125,
};

export interface DialogSizeConfig {
  width: string;
  height: string;
  maxWidth: string;
}

// 共享 width ref：多个组件实例读同一来源，避免重复监听；监听器本身仍按实例生命周期管理。
const width = ref(typeof window === "undefined" ? 0 : window.innerWidth);

function getBreakpoint(value: number): ResponsiveBreakpoint {
  if (value > 1200) return "wide";
  if (value > 840) return "expanded";
  if (value > 600) return "medium";
  return "compact";
}

const DIALOG_SIZE_MAP: Record<ResponsiveBreakpoint, DialogSizeConfig> = {
  compact: { width: "100vw", height: "92vh", maxWidth: "none" },
  medium: { width: "480px", height: "85vh", maxWidth: "90vw" },
  expanded: { width: "600px", height: "80vh", maxWidth: "640px" },
  wide: { width: "720px", height: "75vh", maxWidth: "760px" },
};

export function useResponsiveControl() {
  // Tauri 模式：直接订阅 OS 窗口事件，绕开 webview layout viewport 与 OS 窗口
  // 在 macOS unmaximize/restore 时不同步的已知问题；web 模式 fallback 到 ResizeObserver。
  let unlistenResize: (() => void) | null = null;
  let resizeObserver: ResizeObserver | null = null;
  let landscapeMql: MediaQueryList | null = null;

  const isLandscape = ref(
    typeof window !== "undefined" && window.matchMedia
      ? window.matchMedia("(orientation: landscape)").matches
      : false,
  );

  function onLandscapeChange(e: MediaQueryListEvent) {
    isLandscape.value = e.matches;
  }

  function applyWidth(w: number) {
    if (w > 0) width.value = w;
  }

  function setupResizeObserver() {
    if (typeof document === "undefined" || typeof ResizeObserver === "undefined") return;
    resizeObserver = new ResizeObserver((entries) => {
      const entry = entries[entries.length - 1];
      const w = entry ? entry.contentRect.width : window.innerWidth;
      applyWidth(w);
    });
    resizeObserver.observe(document.documentElement);
  }

  onMounted(async () => {
    applyWidth(window.innerWidth);

    if (isTauri) {
      try {
        const { getCurrentWindow } = await import("@tauri-apps/api/window");
        const appWindow = getCurrentWindow();
        // Tauri 2 的 innerSize() 返回物理像素，要除以 scaleFactor 得到 CSS 像素，
        // 否则 bp/columns 会按物理像素算出来和 layout viewport 严重不匹配。
        const scale = await appWindow.scaleFactor();
        const cssFromPhysical = (px: number) => Math.round(px / scale);
        const size = await appWindow.innerSize();
        applyWidth(cssFromPhysical(size.width));
        unlistenResize = await appWindow.onResized((event) => {
          const w = event.payload.width;
          applyWidth(cssFromPhysical(w));
        });
      } catch (err) {
        // Tauri API 不可用（非 macOS 桌面 / Harmony / 调用失败）→ 回退到 ResizeObserver
        console.warn("[RC] tauri innerSize failed, fallback to ResizeObserver", err);
        setupResizeObserver();
      }
    } else {
      setupResizeObserver();
    }

    if (typeof window !== "undefined" && window.matchMedia) {
      landscapeMql = window.matchMedia("(orientation: landscape)");
      landscapeMql.addEventListener("change", onLandscapeChange);
    }
  });

  onUnmounted(() => {
    unlistenResize?.();
    resizeObserver?.disconnect();
    landscapeMql?.removeEventListener("change", onLandscapeChange);
  });

  const breakpoint = computed<ResponsiveBreakpoint>(() => getBreakpoint(width.value));

  const densityMode = computed<DensityMode>(() => {
    if (typeof window === "undefined") return "standard";
    const stored = localStorage.getItem("legado-ui-density");
    if (stored === "compact" || stored === "standard" || stored === "comfortable") {
      return stored;
    }
    return "standard";
  });

  const densityScale = computed<DensityScale>(() => DENSITY_SCALE_MAP[densityMode.value]);

  const dialogSize = computed<DialogSizeConfig>(() => DIALOG_SIZE_MAP[breakpoint.value]);

  const fontScale = computed<number>(() => {
    if (typeof document === "undefined") return 1;
    const value = getComputedStyle(document.documentElement)
      .getPropertyValue("--ui-font-scale")
      .trim();
    const num = parseFloat(value);
    return Number.isFinite(num) && num > 0 ? num : 1;
  });

  const columns = computed<number>(() => {
    switch (breakpoint.value) {
      case "wide":
        return 6;
      case "expanded":
        return 5;
      case "medium":
        return 4;
      case "compact":
      default:
        return 3;
    }
  });

  return {
    width,
    breakpoint,
    densityMode,
    densityScale,
    dialogSize,
    fontScale,
    isLandscape,
    columns,
  };
}