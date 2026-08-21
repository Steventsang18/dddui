/**
 * @license
 * Copyright 2025 DoDidDoneUi (dodiddoneui.com)
 * SPDX-License-Identifier: Apache-2.0
 */

/**
 * Platform detection utilities
 * 平台检测工具函数
 */

import { getBaseUrl } from '@/common/adapter/httpBridge';

/**
 * Check if running in Electron desktop environment
 * 检测是否运行在 Electron 桌面环境
 */
export const isElectronDesktop = (): boolean => {
  return typeof window !== 'undefined' && Boolean(window.electronAPI);
};

/**
 * Check if running inside the Tauri desktop shell.
 * 检测是否运行在 Tauri 桌面壳中
 */
export const isTauriDesktop = (): boolean => {
  if (typeof window === 'undefined') return false;
  return Boolean((window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__);
};

/**
 * Any desktop shell: legacy Electron or Tauri.
 * 任意桌面壳环境（Electron 或 Tauri）
 */
export const isDesktopShell = (): boolean => isElectronDesktop() || isTauriDesktop();

/**
 * Check if running on macOS
 * 检测是否运行在 macOS
 */
export const isMacOS = (): boolean => {
  return typeof navigator !== 'undefined' && /mac/i.test(navigator.userAgent);
};

/**
 * Check if running on Windows
 * 检测是否运行在 Windows
 */
export const isWindows = (): boolean => {
  return typeof navigator !== 'undefined' && /win/i.test(navigator.userAgent);
};

/**
 * Check if running on Linux
 * 检测是否运行在 Linux
 */
export const isLinux = (): boolean => {
  return typeof navigator !== 'undefined' && /linux/i.test(navigator.userAgent);
};

function isAbsoluteAssetUrl(url: string): boolean {
  return /^[a-z][a-z\d+.-]*:/i.test(url) || url.startsWith('//');
}

/**
 * Resolve a backend-served asset URL for the current environment.
 * In desktop shells (Electron / Tauri), renderer pages are not same-origin
 * with the backend, so backend-relative paths must be expanded against the
 * backend HTTP origin.
 */
export const resolveBackendAssetUrl = (url: string | undefined): string | undefined => {
  if (!url) return url;
  if (isAbsoluteAssetUrl(url) || /^data:/i.test(url)) return url;
  if (url.startsWith('/')) {
    return isDesktopShell() ? `${getBaseUrl()}${url}` : url;
  }
  return url;
};

/**
 * Resolve an extension asset URL for the current environment.
 * Backend-managed extension assets are already emitted as HTTP URLs, so this
 * helper resolves app-relative backend paths into absolute backend URLs when
 * the desktop renderer is not same-origin with the backend process.
 *
 * 将扩展资源 URL 转换为当前环境可用的地址
 */
export const resolveExtensionAssetUrl = (url: string | undefined): string | undefined => {
  return resolveBackendAssetUrl(url);
};

/**
 * Quit the Tauri desktop app. The shell's RunEvent::Exit handler shuts the
 * sidecar backend down cleanly, so we exit the whole app instead of POSTing
 * /api/system/shutdown — the sidecar watchdog would treat that as a crash
 * and restart the backend.
 *
 * 退出 Tauri 桌面应用（壳退出时会干净地停掉后端 sidecar）
 */
export const exitTauriApp = async (): Promise<void> => {
  const internals = (window as unknown as {
    __TAURI_INTERNALS__?: { invoke: (cmd: string, args?: Record<string, unknown>) => Promise<unknown> };
  }).__TAURI_INTERNALS__;
  await internals?.invoke('quit_app');
};

/**
 * Open external URL in the appropriate context
 * - Electron: uses shell.openExternal via IPC (opens on local machine)
 * - Tauri: uses the opener plugin via IPC internals (opens on local machine)
 * - WebUI: uses window.open in client browser (opens on remote client)
 *
 * 在适当的环境中打开外部链接
 */
export const openExternalUrl = async (url: string): Promise<void> => {
  if (!url) return;

  if (isElectronDesktop()) {
    const { ipcBridge } = await import('@/common');
    await ipcBridge.shell.openExternal.invoke(url);
    return;
  }

  if (isTauriDesktop()) {
    const internals = (window as unknown as {
      __TAURI_INTERNALS__?: { invoke: (cmd: string, args?: Record<string, unknown>) => Promise<unknown> };
    }).__TAURI_INTERNALS__;
    try {
      await internals?.invoke('plugin:opener|open_url', { path: url });
    } catch (error) {
      console.error('Failed to open external URL via Tauri:', error);
      window.open(url, '_blank', 'noopener,noreferrer');
    }
    return;
  }

  window.open(url, '_blank', 'noopener,noreferrer');
};
