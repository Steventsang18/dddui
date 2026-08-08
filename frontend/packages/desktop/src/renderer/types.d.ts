declare module '*.svg' {
  const content: string;
  export default content;
}

declare module '*.module.css' {
  const classes: { [key: string]: string };
  export default classes;
}

declare module '*.png' {
  const content: string;
  export default content;
}

declare module '*?raw' {
  const content: string;
  export default content;
}

declare module 'unocss';

// ---------------------------------------------------------------------------
// Electron 类型桩（编译期占位）
// ---------------------------------------------------------------------------
// 本项目已改造为纯 Web 形态（Rust 后端 + 浏览器前端，无 Electron 运行时）。
// 以下声明仅为让仍残留 `import 'electron'` / `Electron.*` 引用的少量历史文件
// 通过 `tsc --noEmit` 类型检查；这些分支在 Web 运行时不会被触发（Web 下
// `process.versions.electron` 为假，且 <webview> 标签不在纯 Web 渲染路径）。
// 切勿在 Web 形态代码中依赖这些类型实现运行时行为。
declare module 'electron' {
  export interface WebviewTag extends HTMLElement {
    src: string;
    reload(): void;
    executeJavaScript(code: string): Promise<unknown>;
    setZoomFactor(factor: number): void;
    isDestroyed(): boolean;
    webContents: { isDestroyed(): boolean; send(channel: string, ...args: unknown[]): void };
    readonly style: CSSStyleDeclaration;
    addEventListener(type: string, listener: EventListenerOrEventListenerObject): void;
    removeEventListener(type: string, listener: EventListenerOrEventListenerObject): void;
  }
  export interface ConsoleMessageEvent {
    message: string;
  }
  export interface BrowserWindow {
    isDestroyed(): boolean;
    webContents: { isDestroyed(): boolean; send(channel: string, ...args: unknown[]): void };
    on(event: string, listener: (...args: unknown[]) => void): void;
  }
  export const ipcMain: {
    handle(channel: string, handler: (event: unknown, info: string) => Promise<unknown>): void;
  };
  export interface App {
    isPackaged: boolean;
    getName(): string;
    getVersion(): string;
    getAppPath(): string;
    getPath(name: string): string;
    setName(name: string): void;
    setPath(name: string, path: string): void;
  }
  export const app: App;
  export const net: { fetch: typeof fetch };
}

declare namespace Electron {
  type WebviewTag = import('electron').WebviewTag;
  type ConsoleMessageEvent = import('electron').ConsoleMessageEvent;
}
