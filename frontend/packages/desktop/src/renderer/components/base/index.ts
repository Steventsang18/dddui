/**
 * @license
 * Copyright 2025 DoDidDoneUi (dodiddoneui.com)
 * SPDX-License-Identifier: Apache-2.0
 */

/**
 * DoDidDoneUi 基础组件库统一导出 / DoDidDoneUi base components unified exports
 *
 * 提供所有基础组件和类型的统一导出入口
 * Provides unified export entry for all base components and types
 */

// ==================== 组件导出 / Component Exports ====================

export { default as RoseModal } from './RoseModal';
export { default as RoseCollapse } from './RoseCollapse';
export { default as RoseSelect } from './RoseSelect';
export { default as RoseScrollArea } from './RoseScrollArea';
export { default as RoseSteps } from './RoseSteps';
export { default as RoseSearchInput } from './RoseSearchInput';
export { default as RoseInlineSearchInput } from './RoseInlineSearchInput';

// ==================== 类型导出 / Type Exports ====================

// RoseModal 类型 / RoseModal types
export type {
  ModalSize,
  ModalHeaderConfig,
  ModalFooterConfig,
  ModalContentStyleConfig,
  RoseModalProps,
} from './RoseModal';
export { MODAL_SIZES } from './RoseModal';

// RoseCollapse 类型 / RoseCollapse types
export type { RoseCollapseProps, RoseCollapseItemProps } from './RoseCollapse';

// RoseSelect 类型 / RoseSelect types
export type { RoseSelectProps } from './RoseSelect';

// RoseSteps 类型 / RoseSteps types
export type { RoseStepsProps } from './RoseSteps';

// RoseSearchInput 类型 / RoseSearchInput types
export type { RoseSearchInputProps } from './RoseSearchInput';

// RoseInlineSearchInput 类型 / RoseInlineSearchInput types
export type { RoseInlineSearchInputProps } from './RoseInlineSearchInput';
