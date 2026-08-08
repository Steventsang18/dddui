/**
 * @license
 * Copyright 2025 RoseUi (roseui.com)
 * SPDX-License-Identifier: Apache-2.0
 */

import { getPlatformServices } from '@/common/platform';

/**
 * Returns baseName unchanged in release builds, or baseName + '-dev' in dev builds.
 * When ROSEUI_MULTI_INSTANCE=1, appends '-2' to isolate the second dev instance.
 * Used to isolate symlink and directory names between environments.
 *
 * @example
 * getEnvAwareName('.roseui')        // release → '.roseui',        dev → '.roseui-dev'
 * getEnvAwareName('.roseui-config') // release → '.roseui-config', dev → '.roseui-config-dev'
 * // with ROSEUI_MULTI_INSTANCE=1:  dev → '.roseui-dev-2'
 */
export function getEnvAwareName(baseName: string): string {
  if (getPlatformServices().paths.isPackaged() === true) return baseName;
  const suffix = process.env.ROSEUI_MULTI_INSTANCE === '1' ? '-dev-2' : '-dev';
  return `${baseName}${suffix}`;
}
