import{D as m,L as c}from"./applyTheme-DjbdwZGr.js";import"./vendor-markdown-Dl3FZDcy.js";import"./vendor-arco-C17IKtUQ.js";import"./vendor-react-BBeMfEpA.js";import"./vendor-katex-CrIz_Pv6.js";/**
 * @license
 * Copyright 2025 RoseUi (roseui.com)
 * SPDX-License-Identifier: Apache-2.0
 */const d="default-theme";function u(t){const r=t.theme==="dark"?"dark":"light";let a;const s=t["css.activeThemeId"]||"";s&&s!==d?a=s:a=r==="dark"?m:c;const i=(t["css.themes"]||[]).filter(e=>!e.is_preset).map(e=>({id:e.id,name:e.name,cover:e.cover,appearance:r,css:e.css,builtin:!1,created_at:e.created_at,updated_at:e.updated_at}));return{"theme.activeId":a,"theme.userThemes":i}}export{u as migrateThemeConfig};
