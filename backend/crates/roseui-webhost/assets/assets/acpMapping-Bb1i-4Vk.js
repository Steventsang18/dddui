/**
 * @license
 * Copyright 2025 RoseUi (roseui.com)
 * SPDX-License-Identifier: Apache-2.0
 */function h(t,e,n){return!t||t.length===0?[]:t.map(r=>({name:r,description:e.get(r)??n,kind:"template",source:"skill",selectionBehavior:"insert"}))}function f(t,e,n){const r=new Map;for(const i of[t,e,n])for(const p of i)r.has(p.name)||r.set(p.name,p);return Array.from(r.values())}/**
 * @license
 * Copyright 2025 RoseUi (roseui.com)
 * SPDX-License-Identifier: Apache-2.0
 */const s=t=>typeof t=="object"&&t!==null&&!Array.isArray(t),a=t=>{if(t==="normal"||t==="neutral_tip_on_empty")return t},o=t=>"command"in t,u=t=>o(t)?typeof t.hint=="string"?t.hint:void 0:typeof t.input?.hint=="string"?t.input.hint:void 0,_=t=>o(t)?a(t.completion_behavior??t.completionBehavior):a(t._meta?.completion_behavior),m=t=>{if(o(t)){const e=t.empty_turn_tip_code??t.emptyTurnTipCode;return typeof e=="string"?e:void 0}return typeof t._meta?.empty_turn_tip_code=="string"?t._meta.empty_turn_tip_code:void 0},c=t=>{if(o(t)){const e=t.empty_turn_tip_params??t.emptyTurnTipParams;return s(e)?e:void 0}return s(t._meta?.empty_turn_tip_params)?t._meta.empty_turn_tip_params:void 0},y=t=>{const e=u(t),n=_(t),r=m(t),i=c(t);return{name:"command"in t?t.command:t.name,description:t.description,kind:"template",source:"acp",selectionBehavior:"insert",...e?{hint:e}:{},...n?{completionBehavior:n}:{},...r?{emptyTurnTipCode:r}:{},...i?{emptyTurnTipParams:i}:{}}},l=t=>t.map(y);export{l as a,h as b,f as m};
