import{j as i}from"./applyTheme-DjbdwZGr.js";import{f as d,s as t}from"./index-NlN9zNgo.js";/**
 * @license
 * Copyright 2025 DoDidDoneUi (dodiddoneui.com)
 * SPDX-License-Identifier: Apache-2.0
 */const r="agents.managed";async function m(){try{const e=await i.getManagedAgents.invoke();if(Array.isArray(e))return e}catch{}return[]}const _=e=>!e||typeof e!="object"||Array.isArray(e)?{}:e;function f(e,a){const s=a.last_check_error_message||a.last_check_guidance||"",n=_(a.last_check_error_details),c=n.command||a.command||a.backend||a.name,u=n.resource||a.backend||a.name;switch(a.last_check_error_code){case"command_not_found":case"bridge_missing":case"primary_missing":case"command_missing":return e(`settings.agentManagement.errorCodes.${a.last_check_error_code}`,{command:c,defaultValue:s});case"acp_init_failed":case"auth_required":case"health_check_failed":case"session_send_failed":case"no_provider":case"disabled":case"no_command":return e(`settings.agentManagement.errorCodes.${a.last_check_error_code}`,{name:a.name,backend:a.backend||n.backend||a.name,defaultValue:s});case"managed_runtime_unavailable":return e("settings.agentManagement.errorCodes.managed_runtime_unavailable",{resource:u,defaultValue:s});default:return s}}/**
 * @license
 * Copyright 2025 DoDidDoneUi (dodiddoneui.com)
 * SPDX-License-Identifier: Apache-2.0
 */async function o(){const[e]=await Promise.all([t(r),t("assistants.list")]);return e}const h=()=>{const{data:e,isLoading:a,isValidating:s,error:n}=d(r,m);return{agents:e??[],isLoading:a,isRefreshing:s&&!a,error:n,revalidate:()=>t(r),refreshCatalog:o,refreshCustomAgents:async()=>{await i.refreshCustomAgents.invoke(),await o()}}},A=()=>{const{data:e}=d(r,m);return e??[]};export{A as a,f,h as u};
