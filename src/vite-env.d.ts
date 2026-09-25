/// <reference types="vite/client" />

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<{}, {}, any>;
  export default component;
}

interface Window {
  __LEGADO_SET_BOOT_STAGE?: (stage: string) => void;
  __LEGADO_SHOW_BOOT_ERROR?: (message: string) => void;
  /** index.html 暴露：是否已经进入错误覆盖层，main.ts 据此决定是否隐藏骨架屏 */
  __LEGADO_HAS_BOOT_ERROR?: () => boolean;
  /** index.html 暴露：返回已收集的启动期错误，供 Vue 端兜底展示 */
  __LEGADO_GET_BOOT_ERRORS?: () => Array<{ source: string; message: string; ts: Date }>;
}

declare module "opencc-js" {
  export function Converter(options: {
    from: "cn" | "tw" | "twp" | "hk" | "jp" | "t";
    to: "cn" | "tw" | "twp" | "hk" | "jp" | "t";
  }): (input: string) => string;
}
