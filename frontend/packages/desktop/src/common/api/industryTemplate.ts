/**
 * @license
 * Copyright 2025 DoDidDoneUi (dodiddoneui.com)
 * SPDX-License-Identifier: Apache-2.0
 *
 * Industry solution template API client.
 * Mirrors the backend `/api/industry-template` + `/api/industry-templates`
 * endpoints defined in roseui-system/src/routes.rs.
 */

import { getBaseUrl } from '@/common/adapter/httpBridge';

export interface IndustryTemplateMeta {
  id: string;
  label: string;
  description: string;
  allowed_tools: string[];
  excluded_tools: string[];
  approval_policy: string;
}

export interface IndustryTemplateSelection {
  template_id: string;
  override_json: string;
}

interface ApiOk<T> {
  success: true;
  data: T;
}

interface ApiErr {
  success: false;
  error?: { message?: string };
  message?: string;
}

type ApiResult<T> = ApiOk<T> | ApiErr;

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${getBaseUrl()}${path}`, {
    headers: { 'Content-Type': 'application/json' },
    ...init,
  });
  const body = (await res.json()) as ApiResult<T>;
  if (!body.success) {
    const msg = (body as ApiErr).message || (body as ApiErr).error?.message || 'request failed';
    throw new Error(msg);
  }
  return (body as ApiOk<T>).data;
}

/** List the built-in industry templates available for selection. */
export async function listIndustryTemplates(): Promise<IndustryTemplateMeta[]> {
  return request<IndustryTemplateMeta[]>('/api/industry-templates');
}

/** Get the current user's selected template + company override. */
export async function getIndustryTemplate(): Promise<IndustryTemplateSelection | null> {
  return request<IndustryTemplateSelection | null>('/api/industry-template');
}

/** Upsert the current user's template selection + company override. */
export async function updateIndustryTemplate(
  templateId: string,
  overrideJson: string
): Promise<IndustryTemplateSelection> {
  return request<IndustryTemplateSelection>('/api/industry-template', {
    method: 'PUT',
    body: JSON.stringify({ template_id: templateId, override_json: overrideJson }),
  });
}

/** Clear the current user's template selection. */
export async function deleteIndustryTemplate(): Promise<void> {
  await request<null>('/api/industry-template', { method: 'DELETE' });
}
