import{l as d}from"./applyTheme-DjbdwZGr.js";/**
 * @license
 * Copyright 2025 DoDidDoneUi (dodiddoneui.com)
 * SPDX-License-Identifier: Apache-2.0
 */const _=50,f=200;async function o(e,s={}){return d.getConversationMessages.invoke({conversation_id:e,limit:s.limit??_,...s.before?{before:s.before}:{},...s.after?{after:s.after}:{},...s.anchorMessageId?{anchor_message_id:s.anchorMessageId}:{},content_mode:s.contentMode??"compact"})}function u(e,s={}){return o(e,s)}function M(e,s,a={}){return o(e,{...a,anchorMessageId:s})}async function A(e,s={}){const a=s.limit??f,c=s.contentMode??"full",t=await o(e,{limit:a,contentMode:c}),i=[t.items];let n=t.oldest_cursor??void 0,l=t.has_more_before;for(;l&&n;){const r=await o(e,{limit:a,before:n,contentMode:c});i.unshift(r.items),n=r.oldest_cursor??void 0,l=r.has_more_before}return i.flat()}/**
 * @license
 * Copyright 2025 DoDidDoneUi (dodiddoneui.com)
 * SPDX-License-Identifier: Apache-2.0
 */const h=({transform:e})=>({...e,x:0}),E=({transform:e})=>({...e,y:0}),g=new Set(["openclaw-gateway","nanobot","remote","gemini","codex"]),v=e=>e?.runtime?.is_processing===!0,b=e=>!!(e&&g.has(e));export{_ as D,v as a,E as b,u as c,o as d,M as e,b as i,A as l,h as r};
