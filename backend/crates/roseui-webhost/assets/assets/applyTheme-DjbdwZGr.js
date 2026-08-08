const __vite__mapDeps=(i,m=__vite__mapDeps,d=(m.f||(m.f=["./migrateThemeConfig-CFQzXfr7.js","./vendor-markdown-Dl3FZDcy.js","./vendor-arco-C17IKtUQ.js","./vendor-react-BBeMfEpA.js","./vendor-arco-DMdW430E.css","./vendor-katex-CrIz_Pv6.js","./vendor-katex-DLVrwanO.css"])))=>i.map(i=>d[i]);
import{_ as An}from"./vendor-markdown-Dl3FZDcy.js";import{g as Pn}from"./vendor-react-BBeMfEpA.js";var H={exports:{}},en;function Tn(){return en||(en=1,(function(n){var a=Object.prototype.hasOwnProperty,e="~";function r(){}Object.create&&(r.prototype=Object.create(null),new r().__proto__||(e=!1));function t(m,l,p){this.fn=m,this.context=l,this.once=p||!1}function c(m,l,p,h,_){if(typeof p!="function")throw new TypeError("The listener must be a function");var w=new t(p,h||m,_),f=e?e+l:l;return m._events[f]?m._events[f].fn?m._events[f]=[m._events[f],w]:m._events[f].push(w):(m._events[f]=w,m._eventsCount++),m}function u(m,l){--m._eventsCount===0?m._events=new r:delete m._events[l]}function g(){this._events=new r,this._eventsCount=0}g.prototype.eventNames=function(){var l=[],p,h;if(this._eventsCount===0)return l;for(h in p=this._events)a.call(p,h)&&l.push(e?h.slice(1):h);return Object.getOwnPropertySymbols?l.concat(Object.getOwnPropertySymbols(p)):l},g.prototype.listeners=function(l){var p=e?e+l:l,h=this._events[p];if(!h)return[];if(h.fn)return[h.fn];for(var _=0,w=h.length,f=new Array(w);_<w;_++)f[_]=h[_].fn;return f},g.prototype.listenerCount=function(l){var p=e?e+l:l,h=this._events[p];return h?h.fn?1:h.length:0},g.prototype.emit=function(l,p,h,_,w,f){var S=e?e+l:l;if(!this._events[S])return!1;var b=this._events[S],I=arguments.length,z,x;if(b.fn){switch(b.once&&this.removeListener(l,b.fn,void 0,!0),I){case 1:return b.fn.call(b.context),!0;case 2:return b.fn.call(b.context,p),!0;case 3:return b.fn.call(b.context,p,h),!0;case 4:return b.fn.call(b.context,p,h,_),!0;case 5:return b.fn.call(b.context,p,h,_,w),!0;case 6:return b.fn.call(b.context,p,h,_,w,f),!0}for(x=1,z=new Array(I-1);x<I;x++)z[x-1]=arguments[x];b.fn.apply(b.context,z)}else{var zn=b.length,$;for(x=0;x<zn;x++)switch(b[x].once&&this.removeListener(l,b[x].fn,void 0,!0),I){case 1:b[x].fn.call(b[x].context);break;case 2:b[x].fn.call(b[x].context,p);break;case 3:b[x].fn.call(b[x].context,p,h);break;case 4:b[x].fn.call(b[x].context,p,h,_);break;default:if(!z)for($=1,z=new Array(I-1);$<I;$++)z[$-1]=arguments[$];b[x].fn.apply(b[x].context,z)}}return!0},g.prototype.on=function(l,p,h){return c(this,l,p,h,!1)},g.prototype.once=function(l,p,h){return c(this,l,p,h,!0)},g.prototype.removeListener=function(l,p,h,_){var w=e?e+l:l;if(!this._events[w])return this;if(!p)return u(this,w),this;var f=this._events[w];if(f.fn)f.fn===p&&(!_||f.once)&&(!h||f.context===h)&&u(this,w);else{for(var S=0,b=[],I=f.length;S<I;S++)(f[S].fn!==p||_&&!f[S].once||h&&f[S].context!==h)&&b.push(f[S]);b.length?this._events[w]=b.length===1?b[0]:b:u(this,w)}return this},g.prototype.removeAllListeners=function(l){var p;return l?(p=e?e+l:l,this._events[p]&&u(this,p)):(this._events=new r,this._eventsCount=0),this},g.prototype.off=g.prototype.removeListener,g.prototype.addListener=g.prototype.on,g.prefixed=e,g.EventEmitter=g,n.exports=g})(H)),H.exports}var En=Tn();const Mn=Pn(En);/**
 * @license
 * Copyright 2026 RoseUi (roseui.com)
 * SPDX-License-Identifier: Apache-2.0
 */const O=new Mn,j=[],T=new Map,Rn=()=>{};let un=()=>{},rn;const $n=n=>`${n}${Math.random().toString(16).slice(2,10)}`,Un=n=>{rn?.(),un=n.emit;const a=n.on({emit(e,r,...t){return O.emit(e,r,...t)}});rn=typeof a=="function"?a:void 0},G=(n,a,...e)=>{un(n,a,...e)},Bn=(n,a)=>{const e=T.get(n)?.get(a);if(!e){O.off(n,a);return}for(const r of e)O.off(n,r);T.get(n)?.delete(a),T.get(n)?.size===0&&T.delete(n)},N=(n,a)=>{const e=(...c)=>{if(/^subscribe(\.callback)?-/.test(n)||j.length===0)return a(...c);Promise.all(j.map(u=>u({name:n,data:c[0]}))).then(()=>a(...c))};let r=T.get(n);r||(r=new Map,T.set(n,r));let t=r.get(a);return t||(t=new Set,r.set(a,t)),t.add(e),O.on(n,e),()=>{O.off(n,e),t.delete(e),t.size===0&&r.delete(a),r.size===0&&T.delete(n)}},Wn=n=>(j.push(n),()=>{const a=j.indexOf(n);a>=0&&j.splice(a,1)}),fn=(n,a)=>N(`subscribe-${n}`,e=>{typeof e!="object"||e===null||!("id"in e)||typeof e.id!="string"||Promise.resolve(a(e.data)).then(r=>G(`subscribe.callback-${n}${e.id}`,r)).catch(r=>{console.error(`[bridge] Provider "${n}" failed:`,r)})}),xn=(n,a)=>{const e=$n(n),r=`subscribe.callback-${n}${e}`;return new Promise(t=>{const c=N(r,u=>{c(),t(u)});G(`subscribe-${n}`,{id:e,data:a})})},jn=n=>{let a=Rn;return{provider(e){return a(),a=fn(n,e),a},invoke:(e=>xn(n,e))}},On=n=>({on:(a=>N(n,a)),emit:(a=>G(n,a))}),i={adapter:Un,buildEmitter:On,buildProvider:jn,emit:G,intercept:Wn,invoke:xn,off:Bn,on:N,subscribe:fn};/**
 * @license
 * Copyright 2025 RoseUi (roseui.com)
 * SPDX-License-Identifier: Apache-2.0
 */function Dn(n){return!!(n&&typeof n.id=="string"&&n.id.trim().length>0&&typeof n.use_model=="string"&&n.use_model.trim().length>0)}function Fn(n){return{provider_id:n.id,model:n.use_model}}function J(n){return Dn(n)?Fn(n):void 0}function Ln(n){const e={type:n.assistant!==void 0&&n.assistant!==null?void 0:n.type,id:n.id,name:n.name,assistant:n.assistant,extra:n.extra},r=n.type==="acp"?void 0:J(n.model);return r&&(e.model=r),e}function Gn(n){return{id:n.provider_id,platform:"",name:"",base_url:"",api_key:"",use_model:n.use_model??n.model}}function Nn(n){return n?Gn(n):void 0}function P(n){if(!n||typeof n!="object")return n;const a=n,e={...a};"model"in a&&(e.model=Nn(a.model));const r=a.extra;if(r&&typeof r=="object"&&!("custom_workspace"in r)){const t=typeof r.workspace=="string"?r.workspace:"",c=r.is_temporary_workspace===!0;e.extra={...r,custom_workspace:t.length>0&&!c}}return e}function Hn(n){return{...n,items:n.items.map(P)}}function kn(){return typeof window<"u"&&window.__backendPort?window.__backendPort:globalThis.__backendPort??13400}function vn(){return typeof window<"u"&&typeof document<"u"&&!window.__backendPort}function Yn(){return vn()?"":`http://127.0.0.1:${kn()}`}function qn(){return vn()?`${window.location.protocol==="https:"?"wss:":"ws:"}//${window.location.host}/ws`:`ws://127.0.0.1:${kn()}/ws`}class yn extends Error{constructor(a){const{method:e,path:r,status:t,body:c}=a;let u="",g="",m;if(c&&typeof c=="object"){const l=c;typeof l.code=="string"&&(u=l.code),typeof l.error=="string"&&(g=l.error),m=l.details}else typeof c=="string"&&(g=c);super(`Backend ${e} ${r} failed (${t}): ${JSON.stringify(c)}`),this.name="BackendHttpError",this.status=t,this.code=u,this.backendMessage=g,this.details=m,this.body=c}}function Ce(n){return!!(n instanceof yn||n&&typeof n=="object"&&"name"in n&&n.name==="BackendHttpError"&&"status"in n&&typeof n.status=="number"&&"code"in n&&typeof n.code=="string")}const Jn=/api[_-]?key|authorization|auth[_-]?token|access[_-]?token|refresh[_-]?token|secret/i;function K(n,a=0){return a>8||n===null||typeof n!="object"?n:Array.isArray(n)?n.map(e=>K(e,a+1)):Object.fromEntries(Object.entries(n).map(([e,r])=>[e,Jn.test(e)?"[REDACTED]":K(r,a+1)]))}async function M(n,a,e,r){const t=`${Yn()}${a}`,c={};e!==void 0&&(c["Content-Type"]="application/json"),console.debug(`[httpBridge] ${n} ${a}`,e!==void 0?JSON.stringify(K(e)).slice(0,500):"(no body)");const u=await fetch(t,{method:n,headers:c,body:e!==void 0?JSON.stringify(e):void 0});if(!u.ok){const l=await u.text().catch(()=>"");let p;try{p=JSON.parse(l)}catch{p=l}throw r?.silentStatuses?.includes(u.status)?console.debug(`[httpBridge] ${n} ${a} → ${u.status} (silenced)`,p):console.error(`[httpBridge] ${n} ${a} → ${u.status}`,p),new yn({method:n,path:a,status:u.status,body:p})}if(console.debug(`[httpBridge] ${n} ${a} → ${u.status} OK`),!u.headers.get("Content-Type")?.includes("application/json"))return;const m=await u.json();return m&&typeof m=="object"&&"data"in m?m.data:m}function k(n,a){return{provider:()=>{},invoke:(async e=>{const r=await n.invoke(e);return a(r)})}}function s(n,a){return{provider:()=>{},invoke:(async e=>{const r=typeof n=="function"?n(e):n;return M("GET",r,void 0,a)})}}function o(n,a){return{provider:()=>{},invoke:(async e=>{const r=typeof n=="function"?n(e):n,t=a?a(e):e;return M("POST",r,t)})}}function v(n,a){return{provider:()=>{},invoke:(async e=>{const r=typeof n=="function"?n(e):n,t=a?a(e):e;return M("PUT",r,t)})}}function E(n,a){return{provider:()=>{},invoke:(async e=>{const r=typeof n=="function"?n(e):n,t=a?a(e):e;return M("PATCH",r,t)})}}function y(n){return{provider:()=>{},invoke:(async a=>{const e=typeof n=="function"?n(a):n;return M("DELETE",e)})}}function Q(n,a){return{provider:()=>{},invoke:(async e=>(console.warn(`[httpBridge] stub: ${n} not yet implemented in backend`),a))}}const Kn="realtime.reconnected",W=new Map;let C=null,Y=null,Z=0,on=!1;function tn(n,a){const e=W.get(n);if(e)for(const r of e)try{r(a)}catch{}}function nn(){if(typeof window>"u"){console.debug("[ensureWs] skipped: no window");return}if(C&&(C.readyState===WebSocket.OPEN||C.readyState===WebSocket.CONNECTING)){console.debug("[ensureWs] skipped: already open/connecting, readyState=",C.readyState);return}const n=qn();console.debug("[ensureWs] connecting to",n);try{C=new WebSocket(n)}catch(e){console.error("[ensureWs] WebSocket constructor threw:",e),sn();return}const a=C;a.addEventListener("open",()=>{console.debug("[ensureWs] CONNECTED");const e=on;on=!0,Z=0,e&&tn(Kn,{timestamp:Date.now()})}),a.addEventListener("close",e=>{console.debug("[ensureWs] CLOSED code="+e.code+" reason="+e.reason),C===a&&(C=null),sn()}),a.addEventListener("error",e=>{console.error("[ensureWs] ERROR",e),a.close()}),a.addEventListener("message",e=>{try{const r=JSON.parse(e.data),t=r.name??r.event,c=r.data??r.payload;console.debug("[WS:msg]",t,JSON.stringify(c).slice(0,200)),t&&tn(t,c)}catch{}})}function sn(){if(Y)return;const n=Math.min(1e3*Math.pow(2,Z),3e4);Z++,Y=setTimeout(()=>{Y=null,nn()},n)}function Se(n,a){if(nn(),!C||C.readyState!==WebSocket.OPEN)return!1;try{return C.send(JSON.stringify({name:n,data:a})),!0}catch(e){return console.error("[wsSend] send failed:",e),!1}}function d(n){return{on:a=>{nn(),W.has(n)||W.set(n,new Set);const e=a;return W.get(n).add(e),()=>{W.get(n)?.delete(e)}},emit:(()=>{})}}function F(n,a){const e=d(n);return{on:r=>e.on(t=>{r(a(t))}),emit:(()=>{})}}/**
 * @license
 * Copyright 2025 RoseUi (roseui.com)
 * SPDX-License-Identifier: Apache-2.0
 */function Qn(n){return{...n,items:n.items.map(Zn)}}function Zn(n){return{conversation:P({...n.conversation,model:n.conversation.model??void 0}),message_id:n.message_id,message_type:n.message_type,message_created_at:n.message_created_at,preview_text:n.preview_text}}/**
 * @license
 * Copyright 2025 RoseUi (roseui.com)
 * SPDX-License-Identifier: Apache-2.0
 */const Xn=new Set(["leader","teammate"]),Vn=new Set(["shared","isolated"]);function na(n){return n==="lead"?"leader":Xn.has(n)?n:"teammate"}function aa(n){return{pending:"pending",idle:"idle",working:"active",thinking:"active",tool_use:"active",completed:"completed",error:"failed",dormant:"dormant"}[n??""]??"idle"}function ea(n){return Vn.has(n)?n:"shared"}function wn(n){const a=n??{},e=a.agent_type??a.backend??"",r=a.assistant_backend??a.backend??e;return{slot_id:a.slot_id??"",conversation_id:a.conversation_id??"",role:na(a.role),assistant_backend:r,icon:a.icon,assistant_name:a.assistant_name??a.agent_name??a.name??"",status:aa(a.status),cli_path:a.cli_path,assistant_id:a.assistant_id,model:a.model,pending_confirmations:a.pending_confirmations??a.pendingConfirmations??0}}function an(n){const a=n??{},r=(Array.isArray(a.assistants)?a.assistants:Array.isArray(a.agents)?a.agents:[]).map(wn),t=a.leader_assistant_id??a.leader_agent_id??"";return{id:a.id??"",user_id:a.user_id??"",name:a.name??"",workspace:a.workspace??"",workspace_mode:ea(a.workspace_mode),leader_assistant_id:t,assistants:r,leader_agent_id:t,agents:r,session_mode:a.session_mode,created_at:a.created_at??0,updated_at:a.updated_at??0}}function ra(n){return Array.isArray(n)?n.map(an):[]}function oa(n){return n==null?null:an(n)}function dn(n){if(!n.assistant_id)throw new Error("assistant_id is required");return{name:n.assistant_name,role:n.role==="leader"?"lead":n.role,model:n.model||"default",assistant_id:n.assistant_id}}/**
 * @license
 * Copyright 2025 RoseUi (roseui.com)
 * SPDX-License-Identifier: Apache-2.0
 */function cn(n){return n.replace(/\\/g,"/")}function L(n){return n.replace(/\/+$/,"")}function ta(n,a){if(!n||!a)return n||".";const e=L(cn(n)),r=L(cn(a));return e===r?".":e.startsWith(r+"/")?e.slice(r.length+1)||".":n}function sa(n,a,e){const r=L(a),t=n.name||"",c=n.type==="directory",u=e?`${e}/${t}`:t;return{name:t,fullPath:`${r}/${u}`,relativePath:u,isDir:c,isFile:!c}}function ia(n,a,e){const r=L(a),t=e==="."?"":e,c=n.map(g=>sa(g,r,t));return e==="."||!e?[{name:r.split("/").pop()||"",fullPath:r,relativePath:"",isDir:!0,isFile:!1,children:c}]:[{name:e.split("/").pop()||"",fullPath:`${r}/${e}`,relativePath:e,isDir:!0,isFile:!1,children:c}]}function da(n){return n.map(a=>({name:a.name,fullPath:a.full_path,relativePath:a.relative_path}))}/**
 * @license
 * Copyright 2025 RoseUi (roseui.com)
 * SPDX-License-Identifier: Apache-2.0
 */const U=n=>({provider:()=>{},invoke:(async()=>(await M("GET",`/api/settings/client?keys=${encodeURIComponent(n)}`))?.[n])}),ca={openFile:o("/api/shell/open-file",n=>({file_path:n})),showItemInFolder:o("/api/shell/show-item-in-folder",n=>({file_path:n})),openExternal:o("/api/shell/open-external",n=>({url:n})),checkToolInstalled:o("/api/shell/check-tool-installed"),openFolderWith:o("/api/shell/open-folder-with")},la={list:s("/api/assistants"),get:s(({id:n,locale:a})=>`/api/assistants/${encodeURIComponent(n)}${a?`?locale=${encodeURIComponent(a)}`:""}`),create:o("/api/assistants"),update:v(n=>`/api/assistants/${n.id}`),delete:y(n=>`/api/assistants/${n.id}`),setState:E(n=>`/api/assistants/${n.id}/state`,n=>{const{id:a,...e}=n;return e}),import:o("/api/assistants/import")},D={create:k(o("/api/conversations",n=>Ln(n)),P),createWithConversation:k(o("/api/conversations/clone",n=>{const a=n.conversation.type==="aionrs",{model:e,...r}=n.conversation,t={...r};if(a){const c=J(e);c&&(t.model=c)}return{conversation:t}}),P),get:k(s(n=>`/api/conversations/${n.id}`,{silentStatuses:[404]}),P),getAssociateConversation:k(s(n=>`/api/conversations/${n.conversation_id}/associated`),n=>n.map(P)),listByCronJob:k(s(n=>`/api/cron/jobs/${n.cron_job_id}/conversations`),n=>n.map(P)),remove:y(n=>`/api/conversations/${n.id}`),update:E(n=>`/api/conversations/${n.id}`,n=>{const a=n.updates,{model:e,...r}=a,t=J(e);return{...r,...t?{model:t}:{},merge_extra:n.merge_extra}}),reset:o(n=>`/api/conversations/${n.id}/reset`),ensureRuntime:o(n=>`/api/conversations/${n.conversation_id}/runtime/ensure`,()=>{}),activeLease:o(n=>`/api/conversations/${n.conversation_id}/active-lease`,()=>{}),stop:o(n=>`/api/conversations/${n.conversation_id}/cancel`,n=>({turn_id:n.turn_id})),activeCount:s("/api/conversations/active-count"),sendMessage:o(n=>`/api/conversations/${n.conversation_id}/messages`,n=>({content:n.input,files:n.files,loading_id:n.loading_id,inject_skills:n.inject_skills})),getSlashCommands:s(n=>`/api/conversations/${n.conversation_id}/slash-commands`),getUsage:s(n=>`/api/conversations/${n.conversation_id}/usage`),askSideQuestion:o(n=>`/api/conversations/${n.conversation_id}/side-question`,n=>({question:n.question})),confirmMessage:o(n=>`/api/conversations/${n.conversation_id}/confirmations/${encodeURIComponent(n.call_id)}/confirm`,n=>({msg_id:n.msg_id,data:n.confirm_key})),listArtifacts:s(n=>`/api/conversations/${n.conversation_id}/artifacts`),updateArtifact:E(n=>`/api/conversations/${n.conversation_id}/artifacts/${n.artifact_id}`,n=>({status:n.status})),responseStream:d("message.stream"),userCreated:d("message.userCreated"),artifactStream:d("conversation.artifact"),turnCompleted:F("turn.completed",n=>{const a=n,e=a.last_message??a.lastMessage,r=e?{id:e.id,type:e.type,content:e.content??null,status:e.status,created_at:e.created_at??e.createdAt??Date.now()}:{content:null,created_at:Date.now()},t=a.runtime??{},c={state:t.state??"idle",can_send_message:t.can_send_message??t.canSendMessage??!0,has_task:t.has_task??t.hasTask??!1,task_status:t.task_status??t.taskStatus,is_processing:t.is_processing??t.isProcessing??!1,pending_confirmations:t.pending_confirmations??t.pendingConfirmations??0,turn_id:t.turn_id??t.turnId??null},u=a.model??{},g={platform:u.platform??"",name:u.name??"",use_model:u.use_model??u.useModel??""};return{session_id:a.session_id??a.sessionId??a.conversation_id??"",turn_id:a.turn_id??a.turnId??c.turn_id??"",status:a.status??"finished",state:a.state??(a.status==="finished"?"ai_waiting_input":"unknown"),detail:a.detail??"",can_send_message:a.can_send_message??a.canSendMessage??a.status==="finished",runtime:c,workspace:a.workspace??"",model:g,last_message:r}}),listChanged:d("conversation.listChanged"),getWorkspace:{provider:()=>{},invoke:(async n=>{const a=ta(n.path,n.workspace),e=`/api/conversations/${n.conversation_id}/workspace?path=${encodeURIComponent(a)}${n.search?`&search=${encodeURIComponent(n.search)}`:""}`,r=await M("GET",e);return ia(r,n.workspace,a)})},confirmation:{add:d("confirmation.add"),update:d("confirmation.update"),confirm:o(n=>`/api/conversations/${n.conversation_id}/confirmations/${encodeURIComponent(n.call_id)}/confirm`,n=>({msg_id:n.msg_id,data:n.data,always_allow:n.always_allow??!1})),list:s(n=>`/api/conversations/${n.conversation_id}/confirmations`),remove:d("confirmation.remove")},approval:{check:s(n=>`/api/conversations/${n.conversation_id}/approvals/check?action=${encodeURIComponent(n.action)}${n.command_type?`&command_type=${encodeURIComponent(n.command_type)}`:""}`)}},pa={statusChanged:d("runtime.statusChanged")},ba={get:s(n=>`/api/projects/${encodeURIComponent(n.project_id)}`),attachFolder:o(n=>`/api/projects/${encodeURIComponent(n.project_id)}/folders`,n=>n.display_name?{uri:n.uri,display_name:n.display_name}:{uri:n.uri}),removeFolder:y(n=>`/api/projects/${encodeURIComponent(n.project_id)}/folders/${encodeURIComponent(n.pe_id)}`)},ma={restart:i.buildProvider("restart-app"),openDevTools:i.buildProvider("open-dev-tools"),isDevToolsOpened:i.buildProvider("is-dev-tools-opened"),systemInfo:k(s("/api/system/info"),n=>({cacheDir:n.cache_dir,workDir:n.work_dir,logDir:n.log_dir,platform:n.platform,arch:n.arch})),getPath:i.buildProvider("app.get-path"),updateSystemInfo:i.buildProvider("update-system-info"),getZoomFactor:i.buildProvider("app.get-zoom-factor"),setZoomFactor:i.buildProvider("app.set-zoom-factor"),getCdpStatus:i.buildProvider("app.get-cdp-status"),updateCdpConfig:i.buildProvider("app.update-cdp-config"),getStartOnBootStatus:i.buildProvider("app.get-start-on-boot-status"),setStartOnBoot:i.buildProvider("app.set-start-on-boot"),getGpuStatus:i.buildProvider("app.get-gpu-status"),setGpuOverride:i.buildProvider("app.set-gpu-override"),writeRendererLog:i.buildProvider("app.write-renderer-log"),logStream:i.buildEmitter("app.log-stream"),devToolsStateChanged:i.buildEmitter("app.devtools-state-changed")},ga={open:i.buildEmitter("update.open"),check:i.buildProvider("update.check"),consumeInstallerLastFailure:i.buildProvider("update.installer-last-failure.consume"),download:i.buildProvider("update.download"),cancelDownload:i.buildProvider("update.download.cancel"),downloadProgress:i.buildEmitter("update.download.progress")},ha={check:i.buildProvider("auto-update.check"),restoreDownloaded:i.buildProvider("auto-update.restore-downloaded"),download:i.buildProvider("auto-update.download"),cancelDownload:i.buildProvider("auto-update.download.cancel"),quitAndInstall:i.buildProvider("auto-update.quit-and-install"),status:i.buildEmitter("auto-update.status")};let X=null;const ua=n=>{X=n},ln=i.buildProvider("show-open"),fa={showOpen:{provider:ln.provider,invoke:(n=>!(typeof window<"u"&&!!window.electronAPI)&&X?X(n):ln.invoke(n))}},xa={getFilesByDir:o("/api/fs/dir"),listWorkspaceFiles:k(o("/api/fs/list"),da),getImageBase64:o("/api/fs/image-base64"),fetchRemoteImage:o("/api/fs/fetch-remote-image"),readFile:o("/api/fs/read"),writeFile:o("/api/fs/write"),createZip:o("/api/fs/zip"),cancelZip:o("/api/fs/zip/cancel"),getFileMetadata:o("/api/fs/metadata"),copyFilesToProject:o("/api/fs/copy"),readBuiltinRule:o("/api/skills/builtin-rule"),readBuiltinSkill:o("/api/skills/builtin-skill"),readAssistantRule:o("/api/skills/assistant-rule/read"),writeAssistantRule:o("/api/skills/assistant-rule/write"),deleteAssistantRule:y(n=>`/api/skills/assistant-rule/${n.assistant_id}`),listAvailableSkills:s("/api/skills"),materializeSkillsForAgent:o("/api/skills/materialize-for-agent"),readSkillInfo:o("/api/skills/info"),importSkill:o("/api/skills/import"),scanForSkills:o("/api/skills/scan"),detectCommonSkillPaths:s("/api/skills/detect-paths"),detectAndCountExternalSkills:s("/api/skills/detect-external"),importSkills:o("/api/skills/import"),listSkillImportHistory:s("/api/skills/import-history"),getSkillImportLimits:s("/api/skills/import-limits"),deleteSkill:y(n=>`/api/skills/${n.skill_name}`),getSkillPaths:s("/api/skills/paths"),getCustomExternalPaths:s("/api/skills/external-paths"),addCustomExternalPath:o("/api/skills/external-paths"),removeCustomExternalPath:y(n=>`/api/skills/external-paths?path=${encodeURIComponent(n.path)}`),enableSkillsMarket:o("/api/skills/market/enable"),disableSkillsMarket:o("/api/skills/market/disable"),listSkillFiles:i.buildProvider("skills.files.list"),readSkillFile:i.buildProvider("skills.files.read")},ka={start:o("/api/fs/office-watch/start"),stop:o("/api/fs/office-watch/stop"),fileAdded:d("workspaceOfficeWatch.fileAdded")},va={contentUpdate:d("fileStream.contentUpdate")},ya={status:Q("googleAuth.status",{success:!1,msg:"Google Auth not available in backend mode"})},wa={subscriptionStatus:s("/api/google/subscription-status")},_a={testConnection:o("/api/bedrock/test-connection")},Ca={listProviders:s("/api/providers"),createProvider:o("/api/providers"),updateProvider:v(n=>`/api/providers/${n.id}`,n=>{const{id:a,...e}=n;return e}),deleteProvider:y(n=>`/api/providers/${n.id}`),fetchProviderModels:o(n=>`/api/providers/${n.id}/models`,n=>({try_fix:n.try_fix})),fetchModelList:o("/api/providers/fetch-models"),detectProtocol:o("/api/providers/detect-protocol")},Sa={sendMessage:D.sendMessage,responseStream:D.responseStream,getManagedAgents:s("/api/agents/management"),getAgentOverrides:s(n=>`/api/agents/${encodeURIComponent(n.id)}/overrides`),setAgentOverrides:v(n=>`/api/agents/${encodeURIComponent(n.id)}/overrides`,n=>({command_override:n.command_override,env_override:n.env_override})),refreshCustomAgents:o("/api/agents/refresh"),testCustomAgent:o("/api/agents/custom/try-connect"),createCustomAgent:o("/api/agents/custom"),updateCustomAgent:v(n=>`/api/agents/custom/${n.id}`,n=>{const{id:a,...e}=n;return e}),deleteCustomAgent:y(n=>`/api/agents/custom/${n.id}`),setAgentEnabled:E(n=>`/api/agents/${n.id}/enabled`,n=>({enabled:n.enabled})),checkManagedAgentHealthById:o(n=>`/api/agents/${n.id}/health-check`,()=>{}),checkProviderHealth:o("/api/agents/provider-health-check"),setConfigOption:v(n=>`/api/conversations/${n.conversation_id}/config-options/${encodeURIComponent(n.option_id)}`,n=>({value:n.value}))},Ia={listServers:s("/api/mcp/servers"),createServer:o("/api/mcp/servers"),importServers:o("/api/mcp/servers/import"),updateServer:v(n=>`/api/mcp/servers/${n.id}`,n=>n.data),deleteServer:y(n=>`/api/mcp/servers/${n.id}`),toggleServer:o(n=>`/api/mcp/servers/${n.id}/toggle`,()=>{}),batchImportServers:o("/api/mcp/servers/import"),getAgentMcpConfigs:s("/api/mcp/agent-configs"),testMcpConnection:o("/api/mcp/test-connection"),checkOAuthStatus:o("/api/mcp/oauth/check-status"),loginMcpOAuth:o("/api/mcp/oauth/login"),logoutMcpOAuth:o("/api/mcp/oauth/logout"),getAuthenticatedServers:s("/api/mcp/oauth/authenticated")},za={sendMessage:D.sendMessage,responseStream:D.responseStream,getRuntime:s(n=>`/api/conversations/${n.conversation_id}/openclaw/runtime`)},Aa={list:s("/api/remote-agents"),get:s(n=>`/api/remote-agents/${n.id}`),create:o("/api/remote-agents"),update:v(n=>`/api/remote-agents/${n.id}`,n=>n.updates),delete:y(n=>`/api/remote-agents/${n.id}`),testConnection:o("/api/remote-agents/test-connection"),handshake:o(n=>`/api/remote-agents/${n.id}/handshake`)},Pa={getConversationMessages:s(n=>{const a=new URLSearchParams;n.limit!==void 0&&a.set("limit",String(n.limit)),n.before&&a.set("before",n.before),n.after&&a.set("after",n.after),n.anchor_message_id&&a.set("anchor_message_id",n.anchor_message_id),n.content_mode&&a.set("content_mode",n.content_mode);const e=a.toString();return`/api/conversations/${n.conversation_id}/messages${e?`?${e}`:""}`}),getConversationMessage:s(n=>`/api/conversations/${n.conversation_id}/messages/${encodeURIComponent(n.message_id)}`),getUserConversations:k(s(n=>{const a=new URLSearchParams;n.cursor&&a.set("cursor",n.cursor),n.limit&&a.set("limit",String(n.limit));const e=a.toString();return`/api/conversations${e?`?${e}`:""}`}),Hn),searchConversationMessages:k(s(n=>`/api/messages/search?keyword=${encodeURIComponent(n.keyword)}&page=${n.page??1}&page_size=${n.page_size??50}`),Qn)};function q(n){return{...n,content_type:n.contentType,contentType:void 0}}const Ta={list:o("/api/preview-history/list",n=>({target:q(n.target)})),save:o("/api/preview-history/save",n=>({target:q(n.target),content:n.content})),getContent:o("/api/preview-history/get-content",n=>({target:q(n.target),snapshot_id:n.snapshot_id}))},Ea={open:d("preview.open")},Ma={convert:o("/api/document/convert")},Ra={start:o("/api/ppt-preview/start"),stop:o("/api/ppt-preview/stop"),status:d("ppt-preview.status")},$a={start:o("/api/word-preview/start"),stop:o("/api/word-preview/stop"),status:d("word-preview.status")},Ua={start:o("/api/excel-preview/start"),stop:o("/api/excel-preview/stop"),status:d("excel-preview.status")},Ba={received:i.buildEmitter("deep-link.received")},Wa={minimize:i.buildProvider("window-controls:minimize"),maximize:i.buildProvider("window-controls:maximize"),unmaximize:i.buildProvider("window-controls:unmaximize"),close:i.buildProvider("window-controls:close"),isMaximized:i.buildProvider("window-controls:is-maximized"),maximizedChanged:i.buildEmitter("window-controls:maximized-changed")},_n={changed:i.buildEmitter("theme:changed"),setActive:i.buildProvider("theme:set-active"),requestCurrent:i.buildProvider("theme:request-current")},ja={getCloseToTray:i.buildProvider("system-settings:get-close-to-tray"),setCloseToTray:i.buildProvider("system-settings:set-close-to-tray"),getNotificationEnabled:U("notificationEnabled"),setNotificationEnabled:v("/api/settings/client",n=>({notificationEnabled:n.enabled})),getCronNotificationEnabled:U("cronNotificationEnabled"),setCronNotificationEnabled:v("/api/settings/client",n=>({cronNotificationEnabled:n.enabled})),getKeepAwake:U("keepAwake"),setKeepAwake:v("/api/settings/client",n=>({keepAwake:n.enabled})),changeLanguage:E("/api/settings",n=>({language:n.language})),languageChanged:d("system-settings:language-changed"),getSaveUploadToWorkspace:U("saveUploadToWorkspace"),setSaveUploadToWorkspace:v("/api/settings/client",n=>({saveUploadToWorkspace:n.enabled})),getAutoPreviewOfficeFiles:U("autoPreviewOfficeFiles"),setAutoPreviewOfficeFiles:v("/api/settings/client",n=>({autoPreviewOfficeFiles:n.enabled})),getPetEnabled:i.buildProvider("system-settings:get-pet-enabled"),setPetEnabled:i.buildProvider("system-settings:set-pet-enabled"),getPetSize:i.buildProvider("system-settings:get-pet-size"),setPetSize:i.buildProvider("system-settings:set-pet-size"),getPetDnd:i.buildProvider("system-settings:get-pet-dnd"),setPetDnd:i.buildProvider("system-settings:set-pet-dnd"),getPetConfirmEnabled:i.buildProvider("system-settings:get-pet-confirm-enabled"),setPetConfirmEnabled:i.buildProvider("system-settings:set-pet-confirm-enabled"),ensureNodeRuntime:o("/api/system/ensure-node-runtime"),ensureManagedAcpTool:o("/api/system/ensure-managed-acp-tool")},Oa={show:i.buildProvider("notification.show"),clicked:i.buildEmitter("notification.clicked")},Da={stopAll:Q("task.stopAll",{success:!0,count:0}),getRunningCount:Q("task.getRunningCount",{success:!0,count:0})},Fa={getStatus:i.buildProvider("webui.get-status"),start:i.buildProvider("webui.start"),stop:i.buildProvider("webui.stop"),statusChanged:i.buildEmitter("webui.status-changed"),changePassword:o("/api/webui/change-password",n=>({new_password:n.newPassword})),changeUsername:o("/api/webui/change-username",n=>({new_username:n.newUsername})),resetPassword:o("/api/webui/reset-password"),generateQRToken:o("/api/webui/generate-qr-token")},La={listJobs:s("/api/cron/jobs"),listJobsByConversation:s(n=>`/api/cron/jobs?conversation_id=${encodeURIComponent(n.conversation_id)}`),getJob:s(n=>`/api/cron/jobs/${n.job_id}`),addJob:o("/api/cron/jobs"),updateJob:v(n=>`/api/cron/jobs/${n.job_id}`,n=>({name:n.updates.name,description:n.updates.description,enabled:n.updates.enabled,schedule:n.updates.schedule,message:n.updates.target?.payload.text,execution_mode:n.updates.target?.execution_mode,agent_config:n.updates.metadata?.agent_config,conversation_title:n.updates.metadata?.conversation_title,max_retries:n.updates.state?.max_retries,queue_enabled:n.updates.state?.queue_enabled})),removeJob:y(n=>`/api/cron/jobs/${n.job_id}`),runNow:o(n=>`/api/cron/jobs/${n.job_id}/run`),saveSkill:o(n=>`/api/cron/jobs/${n.job_id}/skill`,n=>({content:n.content})),hasSkill:k(s(n=>`/api/cron/jobs/${n.job_id}/skill`),n=>!!n?.has_skill),deleteSkill:y(n=>`/api/cron/jobs/${n.job_id}/skill`),onJobCreated:d("cron.job-created"),onJobUpdated:d("cron.job-updated"),onJobRemoved:d("cron.job-removed"),onJobExecuted:d("cron.job-executed")},Ga={health:s("/api/wiki/health"),listPages:s(n=>`/api/wiki/pages?limit=${n.limit??50}&offset=${n.offset??0}`),createPage:o("/api/wiki/pages"),getPage:s(n=>`/api/wiki/pages/${encodeURIComponent(n.page_id)}`),updatePage:v(n=>`/api/wiki/pages/${encodeURIComponent(n.page_id)}`,n=>n.updates),deletePage:y(n=>`/api/wiki/pages/${encodeURIComponent(n.page_id)}`),setTags:v(n=>`/api/wiki/pages/${encodeURIComponent(n.page_id)}/tags`,n=>n.tags),getBacklinks:s(n=>`/api/wiki/pages/${encodeURIComponent(n.page_id)}/backlinks`),search:s(n=>`/api/wiki/search?q=${encodeURIComponent(n.q)}${n.limit?`&limit=${n.limit}`:""}${n.category?`&category=${encodeURIComponent(n.category)}`:""}${n.doc_type?`&doc_type=${encodeURIComponent(n.doc_type)}`:""}`),listTags:s("/api/wiki/tags"),listEdges:s(n=>`/api/wiki/pages/${encodeURIComponent(n.page_id)}/edges`),putEdge:v(n=>`/api/wiki/pages/${encodeURIComponent(n.page_id)}/edges`,n=>({to_page_id:n.to_page_id,edge_type:n.edge_type})),deleteEdge:y(n=>`/api/wiki/pages/${encodeURIComponent(n.page_id)}/edges?to_page_id=${encodeURIComponent(n.to_page_id)}&edge_type=${encodeURIComponent(n.edge_type)}`),getLinkGraph:s(n=>`/api/wiki/pages/${encodeURIComponent(n.page_id)}/graph`),cite:s(n=>`/api/wiki/pages/${encodeURIComponent(n.page_id)}/cite${n.anchor?`?anchor=${encodeURIComponent(n.anchor)}`:""}${n.style?`${n.anchor?"&":"?"}style=${encodeURIComponent(n.style)}`:""}`),unlinked:s(n=>`/api/wiki/pages/${encodeURIComponent(n.page_id)}/unlinked`),titleSearch:s(n=>`/api/wiki/title-search?q=${encodeURIComponent(n.q)}${n.limit?`&limit=${n.limit}`:""}`),listTemplates:s("/api/wiki/templates"),initTemplate:o("/api/wiki/init-template"),listRaw:s("/api/wiki/raw"),ingestRaw:o("/api/wiki/ingest"),deleteRaw:y(n=>`/api/wiki/raw/${encodeURIComponent(n.raw_path)}`),uploadRawFile:{provider:()=>({}),invoke:async(n,a)=>{const e=new FormData;e.append("file",n,a??n.name),a&&e.append("path",a);const r=await fetch("/api/wiki/raw",{method:"POST",body:e});if(!r.ok){const t=await r.text().catch(()=>"");throw new Error(t||`upload failed: ${r.status}`)}return await r.json()}}},Na={getThemes:s("/api/extensions/themes"),getLoadedExtensions:s("/api/extensions"),getAssistants:s("/api/extensions/assistants"),getAgents:s("/api/extensions/agents"),getAcpAdapters:s("/api/extensions/acp-adapters"),getMcpServers:s("/api/extensions/mcp-servers"),getSkills:s("/api/extensions/skills"),getSettingsTabs:s("/api/extensions/settings-tabs"),getWebuiContributions:s("/api/extensions/webui"),getAgentActivitySnapshot:s("/api/extensions/agent-activity"),getExtI18nForLocale:o("/api/extensions/i18n"),enableExtension:o("/api/extensions/enable"),disableExtension:o("/api/extensions/disable"),getPermissions:o("/api/extensions/permissions"),getRiskLevel:o("/api/extensions/risk-level"),stateChanged:d("extensions.state-changed")};function pn(n){return{id:n.plugin_id??n.id,type:n.type??n.plugin_type,name:n.name,enabled:n.enabled,connected:n.connected??!1,status:n.status,last_connected:n.last_connected,activeUsers:n.active_users??0,botUsername:n.bot_username,hasToken:n.has_token??!1,isExtension:n.is_extension,extensionMeta:n.extension_meta}}function bn(n){return{code:n.code,platformUserId:n.platform_user_id,platformType:n.platform_type,display_name:n.display_name,requestedAt:n.requested_at,expiresAt:n.expires_at}}function mn(n){return{id:n.id,platformUserId:n.platform_user_id,platformType:n.platform_type,display_name:n.display_name,authorizedAt:n.authorized_at,lastActive:n.last_active,session_id:n.session_id}}function Ha(n){return{id:n.id,user_id:n.user_id,agent_type:n.agent_type,conversation_id:n.conversation_id,workspace:n.workspace,chatId:n.chat_id,created_at:n.created_at,lastActivity:n.last_activity}}const Ya={getPluginStatus:k(s("/api/channel/plugins"),n=>n.map(pn)),enablePlugin:o("/api/channel/plugins/enable"),disablePlugin:o("/api/channel/plugins/disable"),testPlugin:o("/api/channel/plugins/test"),getPendingPairings:k(s("/api/channel/pairings"),n=>n.map(bn)),approvePairing:o("/api/channel/pairings/approve"),rejectPairing:o("/api/channel/pairings/reject"),getAuthorizedUsers:k(s("/api/channel/users"),n=>n.map(mn)),revokeUser:o("/api/channel/users/revoke"),getActiveSessions:k(s("/api/channel/sessions"),n=>n.map(Ha)),getPlatformSettings:s(n=>`/api/channel/settings/${encodeURIComponent(n.platform)}`),setAssistantSetting:v(n=>`/api/channel/settings/${encodeURIComponent(n.platform)}/assistant`,n=>n.assistant),setDefaultModelSetting:v(n=>`/api/channel/settings/${encodeURIComponent(n.platform)}/default-model`,n=>n.default_model),syncChannelSettings:o("/api/channel/settings/sync"),pairingRequested:F("channel.pairing-requested",n=>bn(n)),pluginStatusChanged:F("channel.plugin-status-changed",n=>{const a=n;return{plugin_id:a.plugin_id,status:pn(a.status)}}),userAuthorized:F("channel.user-authorized",n=>mn(n))},qa={getExtensionList:s("/api/hub/extensions"),install:o("/api/hub/install"),uninstall:o("/api/hub/uninstall"),retryInstall:o("/api/hub/retry-install"),checkUpdates:o("/api/hub/check-updates"),update:o("/api/hub/update"),onStateChanged:d("hub.state-changed")},Ja={reconnected:d("realtime.reconnected")},Ka={create:k(o("/api/teams",n=>({name:n.name,agents:n.agents.map(dn),...n.workspace?{workspace:n.workspace}:{}})),an),list:k(s(n=>`/api/teams?user_id=${encodeURIComponent(n.user_id)}`),ra),get:k(s(n=>`/api/teams/${n.id}`),oa),remove:y(n=>`/api/teams/${n.id}`),addAgent:k(o(n=>`/api/teams/${n.team_id}/agents`,n=>({assistant:dn(n.assistant)})),wn),removeAgent:y(n=>`/api/teams/${n.team_id}/agents/${n.slot_id}`),stop:y(n=>`/api/teams/${n.team_id}/session`),ensureSession:o(n=>`/api/teams/${n.team_id}/session`),getConfigOptions:s(n=>`/api/teams/${n.team_id}/conversations/${encodeURIComponent(n.conversation_id)}/config-options`),activeLease:o(n=>`/api/teams/${n.team_id}/active-lease`,()=>{}),renameAgent:E(n=>`/api/teams/${n.team_id}/agents/${n.slot_id}/name`,n=>({name:n.new_name})),renameTeam:E(n=>`/api/teams/${n.id}/name`,n=>({name:n.name})),setSessionMode:o(n=>`/api/teams/${n.team_id}/session-mode`,n=>({mode:n.session_mode})),getRunState:s(n=>`/api/teams/${n.team_id}/run-state`),sendMessage:o(n=>`/api/teams/${n.team_id}/messages`,n=>({content:n.input,files:n.files})),sendMessageToAgent:o(n=>`/api/teams/${n.team_id}/agents/${n.slot_id}/messages`,n=>({content:n.input,files:n.files})),attachAgent:o(n=>`/api/teams/${n.team_id}/agents/${n.slot_id}/attach`),cancelRun:o(n=>`/api/teams/${n.team_id}/runs/${n.team_run_id}/cancel`,n=>({target_slot_id:n.target_slot_id,reason:n.reason})),cancelChildTurn:o(n=>`/api/teams/${n.team_id}/runs/${n.team_run_id}/agents/${n.slot_id}/cancel`,n=>({reason:n.reason})),pauseSlotWork:o(n=>`/api/teams/${n.team_id}/runs/${n.team_run_id}/agents/${n.slot_id}/pause`,n=>({reason:n.reason})),agentStatusChanged:d("team.agentStatusChanged"),agentSpawned:d("team.agentSpawned"),agentRemoved:d("team.agentRemoved"),agentRenamed:d("team.agentRenamed"),agentRuntimeStatusChanged:d("team.agentRuntimeStatusChanged"),listChanged:d("team.listChanged"),created:d("team.created"),removed:d("team.removed"),renamed:d("team.renamed"),teammateMessage:d("team.teammateMessage"),sessionStatusChanged:d("team.sessionStatusChanged"),taskChanged:d("team.taskChanged"),sessionChanged:d("team.sessionChanged"),runAccepted:d("team.runAccepted"),runStarted:d("team.runStarted"),runUpdated:d("team.runUpdated"),runCompleted:d("team.runCompleted"),runCancelled:d("team.runCancelled"),runFailed:d("team.runFailed"),childTurnStarted:d("team.childTurnStarted"),childTurnCompleted:d("team.childTurnCompleted"),childTurnCancelled:d("team.childTurnCancelled"),slotWorkChanged:d("team.slotWorkChanged")},Ie=Object.freeze(Object.defineProperty({__proto__:null,acpConversation:Sa,application:ma,assistants:la,autoUpdate:ha,bedrock:_a,channel:Ya,conversation:D,cron:La,database:Pa,deepLink:Ba,dialog:fa,document:Ma,excelPreview:Ua,extensions:Na,fileStream:va,fs:xa,google:wa,googleAuth:ya,hub:qa,mcpService:Ia,mode:Ca,notification:Oa,openclawConversation:za,pptPreview:Ra,preview:Ea,previewHistory:Ta,project:ba,realtime:Ja,registerWebShowOpenHandler:ua,remoteAgent:Aa,runtime:pa,shell:ca,systemSettings:ja,task:Da,team:Ka,theme:_n,update:ga,webui:Fa,wiki:Ga,windowControls:Wa,wordPreview:$a,workspaceOfficeWatch:ka},Symbol.toStringTag,{value:"Module"}));function Qa(){return typeof window<"u"&&typeof document<"u"&&!window.__backendPort?"":`http://127.0.0.1:${typeof window<"u"&&window.__backendPort||13400}`}async function B(n,a,e){const r=`${Qa()}${a}`,t={};e!==void 0&&(t["Content-Type"]="application/json");const c=await fetch(r,{method:n,headers:t,body:e!==void 0?JSON.stringify(e):void 0});if(!c.ok){const m=await c.text();throw new Error(`ConfigService ${n} ${a} failed (${c.status}): ${m}`)}if(!c.headers.get("Content-Type")?.includes("application/json"))return;const g=await c.json();return g&&typeof g=="object"&&"data"in g?g.data:g}class Za{constructor(){this.cache=new Map,this.subscribers=new Map,this.initialized=!1,this.initPromise=null}initialize(){return this.initPromise?this.initPromise:(this.initPromise=(async()=>{const a=await B("GET","/api/settings/client");if(this.cache.clear(),a)for(const[e,r]of Object.entries(a))this.cache.set(e,r);if(!this.cache.has("theme.activeId")){const{migrateThemeConfig:e}=await An(async()=>{const{migrateThemeConfig:t}=await import("./migrateThemeConfig-CFQzXfr7.js");return{migrateThemeConfig:t}},__vite__mapDeps([0,1,2,3,4,5,6]),import.meta.url),r=e({theme:this.cache.get("theme"),"css.activeThemeId":this.cache.get("css.activeThemeId"),"css.themes":this.cache.get("css.themes"),customCss:this.cache.get("customCss")});this.cache.set("theme.activeId",r["theme.activeId"]),this.cache.set("theme.userThemes",r["theme.userThemes"]),B("PUT","/api/settings/client",r).catch(()=>{})}this.initialized=!0})(),this.initPromise.catch(()=>{this.initPromise=null}),this.initPromise)}whenReady(){return this.initialize()}get(a){return this.cache.get(a)}async set(a,e){this.cache.set(a,e),this.notify(a,e),await B("PUT","/api/settings/client",{[a]:e})}setLocal(a,e){this.cache.set(a,e),this.notify(a,e)}async remove(a){this.cache.delete(a),this.notify(a,void 0),await B("PUT","/api/settings/client",{[a]:null})}async setBatch(a){for(const[e,r]of Object.entries(a))this.cache.set(e,r),this.notify(e,r);await B("PUT","/api/settings/client",a)}subscribe(a,e){return this.subscribers.has(a)||this.subscribers.set(a,new Set),this.subscribers.get(a).add(e),()=>{this.subscribers.get(a)?.delete(e)}}isInitialized(){return this.initialized}reset(){this.cache.clear(),this.subscribers.clear(),this.initialized=!1,this.initPromise=null}notify(a,e){const r=this.subscribers.get(a);if(r)for(const t of r)t(e)}}const gn=new Za;/**
 * @license
 * Copyright 2025 RoseUi (roseui.com)
 * SPDX-License-Identifier: Apache-2.0
 */const V="light",Cn="dark",Xa="system";/**
 * @license
 * Copyright 2025 RoseUi (roseui.com)
 * SPDX-License-Identifier: Apache-2.0
 */function Va(n,a,e){const r=n===Xa?e?Cn:V:n;return a.find(t=>t.id===r)??a.find(t=>t.id===V)??a[0]}const ne="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAALQAAABwCAYAAAC3tFqQAAAACXBIWXMAAAsTAAALEwEAmpwYAAAAAXNSR0IArs4c6QAAAARnQU1BAACxjwv8YQUAAA1YSURBVHgB7Z17TFRXHsd/MyBUBETxxcOIYnnERWDFtvgsPjY0tOpmxT50U9PYx6rpYk3VrK5F3ZpqsrYmmNXGNG3drruRpLpoa9ZH/cNKF0VYrWlXcaHaWuiCpTxFgdv7PcMZ7gwzPGZgmLn390lO7r1zzsy9A9/zu9/zO/feMZEdF0uuPk5kWtzeblpCpMQQw3gZJpOplBSltM3sty09NbHCpk6ulJSUhz2gxjfqfqrP+fvfDtGJ40fp+zt3iGG8jYfjEiguPoFefOl3FBkd9c4QGrYtNXVirbUBxFx0+cuSP+a+qYSEhCrqS1y4eH0JCQlR1q3fqBRdvlYCDZOkqOTLt9et3+RTX4YLF1kQiKFhdZ1MhSVfxVR9e6v8t89lU319HTGMr6FGavq44F80bFhwhtmvve2Ng+/+hcXM+Cz19fX0j8OHyM9sWmJWFEq5fv3rLo3i1PJZR3nJwYc8O1+tU4N8wU6ip2Z0rQ94Vu056puDC4j84ohhBpTjBcfgPxab/l38pfLotF90aaDqkCI028+p5XrHekiQRcyS+iaiRZstS2COVIX8z856pV6tyyCGGVBULZPZWWWw3XaIdn2oXV2Q7bbJ/s0M4yGcCvpdzToic7Fm+04N0eEzndtYl9EZtKlvaL/euX3/MDGMZ4DlICfpENU5KHHdpEumxan10c7r/aeRovrnQU3pcDFOgZb9qRt6micsvt59fWsxMYxHMRPD6AgWNKMrWNCMrmBBM7qCBc3oChY0oytY0IyuYEEzuoIFzegKFjSjK1jQjK5gQfswZrMZt/QT04k/MT5Le3s7MbZwhGZ0BQua0RUsaEZXsKA9AA/ePAcLeoCBkOfOnUuLFy+mIUOGEDOwcJZjgImIiKBdu3aJJbISJ06coLa2NmIGiO5ukuXiXhkxYoTy3nvvKS0tLYoqZqW4uFhJT0/3qe/gSwVaFpYjPimD/IcEEtN/wGosX76cnn76aQoICBDbSUlJtG7dOgoPDyem/4B2oWEgBD0uKp6mzVhKDw0NIcZ9MAhcsGABrV69moKCgqiiooLKysrE61lZWfTCCy+Qvz+7vf4Amk1+ZJHQMDDbVwSHjiLGPeLj42nbtm2UkJBAdXV1tHXrVsrJyaHy8nIh8Ndff51mz54tBM64TnBouEWzIZ1nPJu/KESNSB0dM5UY1wgLC6MNGzZQWloaKYpC+/fvpyNHjtCnn35KGzdupFu3btHo0aNpz549NGHCBGJcIyomSdVqdhdX4TBExCbMoNjEGeyrXWDFihW0bNkyYSmOHTtG+/bto3v37okMR0FBAeXm5lJTUxNNmTKFtmzZwn66j0CTsYkzaXLCTIf1Ts950ROmsq/uI8nJybR+/XoKDAyk48ePi/Xbt29b6x88eED5+fn0wQcfiG0MGFetWiUGjUzPWB3EhCSnbfx7+oBH5y6nm19foG8rrhDjHD8/P+GZT58+TXfv3qVDhw6JwSBsh/TKWMfDuRGZa2tracyYMUL8iOb3798nxjmwGDGT09S/VfeuQTwfesPmPOqJyu/+S9+UXaJ7zfXEOAaiRoFwEY3ByJEjaffu3cIvb968mYqKisTrmDWU0+Foi/cwXUFQRUoubGRkj213v7m29zOFSIvgQyFqiJvpCmYA7WcBo6OjKSMjgyIjIykxMZEuXrxoI3jGOaPGTlTF/HiPUVlLn5KhsrcM7xC20aM1oiwmStQZQPr888/F4M8e2I4DBw5QaGgonTt3zmkkxqQLctSffPIJXblibHvXl6hsj0vZfRmtv/vmqqG9NQS9aNEiMYGSl5dHBw8eFN5YC/LQe/fuFT5anQJ3+Bm4eGn79u1C9JcuXSIj01uv7AyXM/voRUjvYdBo1EwI0m8rV64UNmLTpk0izwxbYX9VHYTc3Nxsc8sUBA5/vWbNGjGAxMBwx44dItIbEQRITJIgHeeqmIHfiy+vzj11tohcBXlBTMRA1I31NdTaaqzR+o8//ihEiNTbM888Q/PmzaPx48dTY2MjVVZWdrEYGAhOmjRJXE66c+dOys7OpjNnztCrr75KhYWFhst2QD8T4x+juClz3A6MC+c/0vssR2+Ap5bZEKOBKe0nnniCMjMzac6cOSJKw1tfu3ZN2I7q6mqRpsO0+PTp08WEyo0bN0R0xixiVVWVoTIdEHKUmk+OVi2GOxFZS5+yHL0BPQz+Bx7baNkQ2A/MDGJQl5KSQkuWLBHCxoxgcHCwsBuIvshRX716lQ4fPixSePDcra2tZCSgjwmqTgbCqg7IJV9ylIqDNpKwIUwUWIfLly+LqD18+HCKjY0V0RdRGDYE2RB0AKPlnuGToQlXshe9ZUCvYTSqsAEGgijw2EjdGRlPCFnikYtyjSxsIwNrMbYjxespPHqVub2wq3+ooNYHLcToh4EY7PVp/zQIWIWtZkVq797xqllHpNX4uoq+g/8povFgCVkyqPcB4Y+A0xIKhI2Zx+qqchpMfEnMmJzB8Q7mMXvSH/cGr7mxDX8QlHteGLW9lcF6WKO3RGNHeN2dmvZRu0odQGLJ4h5c4I3HRcVR+JiJXhONHeHVtx7LqA1Y3J7HV0SsxWfupWdxdzKQA1ecIUeNjfEpEWvxyYdDaMXdUF9DP6nCxmASAmf6BqIwHgOAi+nDx8T4/JWTPv+0E/wzUKI6bpyEqGt+KKeGuhrdCtyd6CwFjGdaIApj6W0DO3fQ3eN7tNEbQNS4rBXLhrpqtyyKvAfQk2kyd+0FIi4eHoS/ybAOIetJwPbo/nlUUuAygre2tojoDZsCuwKBQ+i9YTDyvX3ZJ8SL7woBB4r1CF2L1xGGe8Aa/sH2URxIcTd2RHFt8SZkxPX3DxBLP3WJ78LPT7HATwzsQHrxUerAyB4pbNyNg2tPWsR6i7p+3yp4WWe/7gx4WYhSu965DBAdD1EWr0GseJ1F2zMs6F4AIbGYfAN+/CWjK1jQjK5gQTO6ggXN6AoWNKMrWNCMrmBBM7qCBc3oChY0oysMP1M4OTaagocNFeulV244bbd0SYbaLojyj35GDY1N5AlSpj4slmU3v3O6z3Fjw9UyUq1vVtt9K17DcWYufJQaGprp5OkvyEgYWtAQw8G8zdbtnI17HIo6c+FjtPblZWIdYsl79wh54tje2fVat8cFVi7PUo8vXa2/rrZ72/Laiiy1A84T6+gI5wuN8wxvQ1sOGQGt20lxDtsh0lnXm5rJ29EeL5GJjIShI/TK5U+K5clThSLKLf31fHr/oxNd2p0v/A+9tedDYU3yj54lb+f9j45T5Q81qrCbxLEbCcMKOmVqnDitg/xjZ2nWjGRhJxC17U/vFo+tiNM32tj7WXzOrPSpNHnSeBEdy/53m0qv3qDKqhqbdrAu4PyFKxQcPJSWLp4nlmgHr2vf3lUs30sRn200DCvozAUd4lIjGAZTJ099IXwnonbOlbdt2gYHB9Gm154X6zlVtn4W7eFZ7YE4t+zYbx2oAfkZJ5MsZwQtK1c8SW/9+UNV2IXkLhC0dV+njDUoNKSHRsRFRAbnL1hOyfLUjMiNKNwbEHGlmGFFtmzfLwrELAd18ixg+750YXPQFlZGRua1r2QT4x6GFLS0F0h1yYiILIG0EkjR9QatB887cER0CpRVa94UIkXHga2wB+0hZLSV6wDtHXUApvcYUtCZCyyn+/MXSm1ez//4M7GU0bs77D24FnQUOXjM/FV6l/faD9TKbnb+HjhyyozrGM5DQ4QQowR5XMnk2CjLUh3cORoc2n5Op/C0PlkibYSctNFiP6hEB2D6B8MJWitg+4GZllmPpXQraE/SnaeXddwpLBhO0DI6ny8sdZijxeQKhA6rgJy0sylnrdgdRfNZ6RbbghSeKyDCyzTh5EnRTvPJmLoX+7np2n70hqE8NEQmfW/egXyR0rIvGNwBWAWZN3YEBIeBJFj70jKbKIp9yOgvfbkryPdiwkemGSXY39qXs63fx2jXbDjDUBFaGzWdTWLg1A2hIpKjfXczg7im451d60SUPLjvD1aBy0GlyG+7kVfOP3pGdCqRV17/vMhVW7InQTRu3EhrJxIzg/00KePrGEbQfYmaECEErc1kOAKCXbVmJ/1p6yvCFmg9OTrC+389Qe6AzoULjuD70UksV9aF2+wf++mPyRi90K8/jWxkpNjgeysr7w7IJaba7AwiMkdlW/r9p5GNjCcEJi0N4xy+Y4XRFSxoRlewoBldwYJmdAULmtEVLGhGV7CgGV3BgmZ0BQua0RUsaEZXsKAZXcGCZnQFC5rRFSxoRlewoBldYVbIVDFiRCgxjC8TGTEKD1otVSN0+7G01ARiGF8mMmI0mRRV0Eo7HZ01M5mGPhRIDOOLQLsL5z9CbWa/beb06UnnHgoM2PtU1ixiGF8EYg4bHrI3PTWxQgwKA/1ac9OmJZYuyprNkZrxGaDVZb+ZT7NnJpcG+g3LxWvW3ysoKSkJa2nzz62tbfj9qbNFdOf7/6ulmhjG20ASI+2XCTR7RjIFBgbuhZhTUyfWoq7LD3AUlnwVY257kEtmczIplEIM43WYKkxqMqNNHf/BMmtrfgYpZBlCEo9IfAAAAABJRU5ErkJggg==",ae=""+new URL("misaka-mikoto-theme-Dpit7WIR.png",import.meta.url).href,ee=""+new URL("hello-kitty-D-kfmBoX.png",import.meta.url).href,re=""+new URL("retro-windows-DXRiLN5o.png",import.meta.url).href,oe=""+new URL("y2k-ledger-cover-DFTue_rd.png",import.meta.url).href,te=""+new URL("obsidian-book-cover-CUYTvHZx.png",import.meta.url).href,se=`/* Misaka Mikoto Theme - 御坂美琴主题 (优化版) */
/* 参考《科学超电磁炮》配色风格 */

:root {
  /* ========== 核心颜色变量 ========== */
  /* 主色调 - Tokiwadai Blue & Electric Blue */
  --color-primary-base: #1e3a8a;
  --color-primary: var(--color-primary-base);
  --primary: var(--color-primary-base);
  --color-primary-light-1: #3b82f6;
  --color-primary-light-2: #60a5fa;
  --color-primary-light-3: #93c5fd;
  --color-primary-dark-1: #1e40af;
  --primary-rgb: 30, 58, 138;

  /* 品牌色 - 使用变量引用减少重复 */
  --brand: var(--color-primary-base);
  --brand-light: #dbeafe;
  --brand-hover: var(--color-primary-light-1);
  --color-brand-fill: var(--color-primary-base);
  --color-brand-bg: #dbeafe;

  /* AOU 品牌色板 - 蓝色系渐变（常盘台校服色） */
  --aou-1: #eff6ff;
  --aou-2: #dbeafe;
  --aou-3: #bfdbfe;
  --aou-4: #93c5fd;
  --aou-5: #60a5fa;
  --aou-6: #3b82f6;
  --aou-7: #2563eb;
  --aou-8: #1e40af;
  --aou-9: #1e3a8a;
  --aou-10: #172554;

  /* 背景色 - 完整定义以兼容所有组件 */
  --bg-base-color: #f0f9ff;
  --bg-base: #ffffff;
  --bg-1: var(--bg-base-color);
  --bg-2: #ffffff;
  --bg-3: #e0f2fe;
  --bg-4: #bae6fd;
  --bg-5: #93c5fd;
  --bg-6: #60a5fa;
  --bg-8: #3b82f6;
  --bg-9: #1e3a8a;
  --bg-10: #172554;
  --color-bg-1: var(--bg-base-color);
  --color-bg-2: #ffffff;
  --color-bg-3: #e0f2fe;
  --color-bg-4: #bae6fd;
  --bg-hover: #e0f2fe;
  --bg-active: #bae6fd;
  --fill: var(--bg-base-color);
  --color-fill: var(--bg-base-color);
  --fill-0: #ffffff;
  --fill-white-to-black: #ffffff;
  --color-fill-2: #e0f2fe;
  --color-fill-3: #bae6fd;

  /* 文字色 - 完整定义 */
  --text-base-color: #1e293b;
  --text-0: #000000;
  --text-primary: var(--text-base-color);
  --text-secondary: #475569;
  --text-disabled: #94a3b8;
  --text-white: #ffffff;
  --color-text-1: var(--text-base-color);
  --color-text-2: #475569;
  --color-text-3: #94a3b8;
  --color-text-4: #cbd5e1;

  /* 边框色 - 完整定义 */
  --border-base-color: #93c5fd;
  --border-base: var(--border-base-color);
  --border-light: #bfdbfe;
  --border-special: #93c5fd;
  --color-border: var(--border-base-color);
  --color-border-1: var(--border-base-color);
  --color-border-2: #bfdbfe;
  --color-border-3: #dbeafe;
  --color-border-4: #eff6ff;

  /* 语义色 */
  --success: #10b981;
  --warning: #f59e0b;
  --danger: #ef4444;
  --info: var(--color-primary-light-1);

  /* 消息背景色 */
  --message-user-bg: #dbeafe;
  --message-tips-bg: var(--bg-base-color);
  --workspace-btn-bg: #e0f2fe;

  /* 对话框颜色 */
  --dialog-fill-0: rgba(255, 255, 255, 0.9);

  /* ========== 动画变量 ========== */
  --transition-duration: 0.3s;
  --transition-timing: ease;

  /* ========== 渐变背景 ========== */
  --gradient-primary: linear-gradient(135deg, #1e3a8a 0%, #3b82f6 100%);
  --gradient-primary-hover: linear-gradient(135deg, #3b82f6 0%, #60a5fa 100%);
}

/* ========== 全局样式 ========== */
body {
  font-family: 'Inter', 'SF Pro Display', 'Segoe UI', 'Microsoft YaHei', sans-serif;
  background-color: var(--bg-1);
}

html {
  background-color: var(--bg-1);
}

/* ========== 布局样式 ========== */
.arco-layout,
[class*='layout'] {
  background-color: var(--bg-1);
}

.arco-layout-content {
  background-color: var(--bg-1);
}

/* ========== 侧边栏 ========== */
.layout-sider {
  background-color: #e0f2fe;
  border-right: 2px solid var(--border-base-color);
  position: relative;
  z-index: 100;
}

.layout-sider-header {
  background: var(--gradient-primary);
  color: white;
  box-shadow: 0 2px 8px rgba(30, 58, 138, 0.3);
}

.layout-sider svg,
.layout-sider-header svg {
  fill: none;
  stroke: rgba(255, 255, 255, 0.9);
  color: rgba(255, 255, 255, 0.9);
  transition: stroke var(--transition-duration) var(--transition-timing);
}

.layout-sider-header svg:hover {
  fill: none;
  stroke: white;
  color: white;
}

/* ========== 图标样式 - 简化选择器 ========== */
/* 全局图标默认颜色 */
.theme-icon svg,
svg:not([class*='model'] svg):not([class*='Model'] svg) {
  fill: none;
  stroke: var(--color-primary-base);
  color: var(--color-primary-base);
  transition:
    stroke var(--transition-duration) var(--transition-timing),
    color var(--transition-duration) var(--transition-timing);
}

.theme-icon svg:hover,
svg:not([class*='model'] svg):not([class*='Model'] svg):hover {
  fill: none;
  stroke: var(--color-primary-light-1);
  color: var(--color-primary-light-1);
}

/* 按钮内图标 */
button:not([class*='model']) svg,
.arco-btn:not([class*='model']) svg {
  fill: none;
  stroke: var(--color-primary-base);
  color: var(--color-primary-base);
  transition: stroke var(--transition-duration) var(--transition-timing);
}

button:not([class*='model']) svg:hover,
.arco-btn:not([class*='model']) svg:hover {
  fill: none;
  stroke: var(--color-primary-light-1);
  color: var(--color-primary-light-1);
}

/* 主要按钮内的图标为白色 */
.arco-btn-primary svg {
  stroke: white;
  color: white;
}

/* ========== 背景图片设置 ========== */
.layout-content.bg-1 {
  background-color: var(--bg-1);
  position: relative;
}

/* 半透明遮罩层 */
.layout-content.bg-1::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(
    135deg,
    rgba(240, 249, 255, 0.75) 0%,
    rgba(224, 242, 254, 0.8) 50%,
    rgba(240, 249, 255, 0.75) 100%
  );
  z-index: 0;
  pointer-events: none;
}

/* 聊天页面背景图 */
.chat-layout-header,
[class*='chat-layout'] .arco-layout-content,
[class*='conversation'] .arco-layout-content {
  position: relative;
}

[class*='chat-layout'] .arco-layout-content::before,
[class*='conversation'] .arco-layout-content::before {
  content: '';
  position: absolute;
  inset: 0;
  background: transparent;
  opacity: 0;
  z-index: 0;
  pointer-events: none;
}

/* 确保内容在背景之上 */
.layout-content.bg-1 > *,
[class*='chat-layout'] .arco-layout-content > *,
[class*='conversation'] .arco-layout-content > * {
  position: relative;
  z-index: 1;
}

/* ========== 输入框和发送框 ========== */
.guidLayout,
[class*='guid'] {
  position: relative;
  z-index: 10;
}

.guidInputCard {
  background-color: rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(8px);
  border: 2px solid var(--border-base-color);
  border-radius: 16px;
  box-shadow: 0 2px 20px rgba(30, 58, 138, 0.1);
}

.guidInputCard textarea,
[class*='guidInputCard'] textarea {
  background-color: rgba(255, 255, 255, 0.98);
  color: var(--color-text-1);
}

/* 发送框样式 */
.sendbox-container:not([class*='model']):not([class*='Model']),
[class*='sendbox']:not([class*='input']):not([class*='textarea']):not([class*='model']):not([class*='Model']):not(
    [class*='tools']
  ) {
  border-radius: 16px;
  border: 2px solid var(--border-base-color);
  background-color: rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(8px);
  box-shadow: 0 4px 20px rgba(30, 58, 138, 0.15);
  transition: all var(--transition-duration) var(--transition-timing);
}

.sendbox-container textarea,
[class*='sendbox'] textarea {
  border: none;
  background: transparent;
}

.sendbox-container:focus-within,
[class*='sendbox']:focus-within {
  border-color: var(--color-primary-light-1);
  box-shadow: 0 6px 24px rgba(59, 130, 246, 0.3);
}

.sendbox-container svg:not([class*='model'] svg),
[class*='sendbox']:not([class*='model']) svg {
  fill: none;
  stroke: var(--color-primary-base);
  color: var(--color-primary-base);
  transition: stroke var(--transition-duration) var(--transition-timing);
}

.sendbox-container svg:not([class*='model'] svg):hover,
[class*='sendbox']:not([class*='model']) svg:hover {
  fill: none;
  stroke: var(--color-primary-light-1);
  color: var(--color-primary-light-1);
  transform: scale(1.1);
}

/* ========== 消息气泡 ========== */
.message-item.user .message-bubble,
[class*='message'][class*='user'] .message-content {
  background: var(--gradient-primary);
  color: white;
  border-radius: 16px 16px 4px 16px;
  border: none;
  box-shadow: 0 4px 12px rgba(30, 58, 138, 0.3);
  padding: 12px 16px;
}

.message-item.ai .message-bubble,
[class*='message'][class*='ai'] .message-content,
[class*='message'][class*='assistant'] .message-content {
  background-color: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(10px);
  border: 2px solid #bfdbfe;
  border-radius: 16px 16px 16px 4px;
  box-shadow: 0 4px 16px rgba(30, 58, 138, 0.15);
  padding: 12px 16px;
}

/* 工具调用相关样式 */
.message-item.ai .arco-alert,
[class*='message'][class*='ai'] .arco-alert,
[class*='message'][class*='assistant'] .arco-alert,
.message-item.ai [class*='alert'],
[class*='message'][class*='ai'] [class*='alert'],
[class*='message'][class*='assistant'] [class*='alert'] {
  background-color: rgba(255, 255, 255, 0.6);
  border: 1px solid #bfdbfe;
  border-radius: 8px;
  margin: 4px 0;
}

.message-item.ai .arco-card,
[class*='message'][class*='ai'] .arco-card,
[class*='message'][class*='assistant'] .arco-card,
.message-item.ai [class*='card'],
[class*='message'][class*='ai'] [class*='card'],
[class*='message'][class*='assistant'] [class*='card'] {
  background-color: rgba(255, 255, 255, 0.6);
  border: 1px solid #bfdbfe;
  border-radius: 8px;
  margin: 4px 0;
}

.message-item.ai [class*='status']:not([class*='message']):not([class*='bubble']),
[class*='message'][class*='ai'] [class*='status']:not([class*='message']):not([class*='bubble']) {
  background-color: rgba(255, 255, 255, 0.8);
  border: 1px solid #bfdbfe;
  border-radius: 6px;
  padding: 2px 6px;
}

/* ========== 按钮样式 ========== */
.arco-btn-primary:not([class*='icon']):not([class*='circle']):not([class*='model']),
button[type='primary']:not([class*='icon']):not([class*='circle']):not([class*='model']) {
  background: var(--gradient-primary);
  border-color: var(--color-primary-base);
  border-radius: 12px;
  font-weight: 600;
  color: white;
  transition: all var(--transition-duration) var(--transition-timing);
}

.arco-btn-primary:hover:not([class*='icon']):not([class*='circle']):not([class*='model']),
button[type='primary']:hover:not([class*='icon']):not([class*='circle']):not([class*='model']) {
  background: var(--gradient-primary-hover);
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(59, 130, 246, 0.4);
}

.arco-btn-secondary:not([class*='model']) svg,
button[type='secondary']:not([class*='model']) svg {
  fill: none;
  stroke: var(--color-primary-base);
  color: var(--color-primary-base);
}

.arco-btn-secondary:not([class*='model']) svg:hover,
button[type='secondary']:not([class*='model']) svg:hover {
  fill: none;
  stroke: var(--color-primary-light-1);
  color: var(--color-primary-light-1);
}

/* ========== 滚动条 ========== */
::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-thumb {
  background: transparent;
  border-radius: 4px;
  transition: background var(--transition-duration) var(--transition-timing);
}

::-webkit-scrollbar-thumb:hover {
  background: var(--gradient-primary-hover);
}

*:hover::-webkit-scrollbar-thumb {
  background: rgba(59, 130, 246, 0.3);
}

*:hover::-webkit-scrollbar-thumb:hover {
  background: var(--gradient-primary-hover);
}

::-webkit-scrollbar-track {
  background: transparent;
  border-radius: 4px;
}

/* ========== 选中和链接 ========== */
::selection {
  background-color: var(--color-primary-light-1);
  color: white;
}

a:not([class*='button']):not([class*='btn']) {
  color: var(--color-primary-base);
  transition: color var(--transition-duration) var(--transition-timing);
}

a:hover:not([class*='button']):not([class*='btn']) {
  color: var(--color-primary-light-1);
  text-decoration: underline;
}

/* ========== Tooltip 和 Popover ========== */
.arco-tooltip-popup,
.arco-popover-popup {
  pointer-events: none;
}

/* ========== 对话框 ========== */
.arco-modal-body {
  background-color: rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(10px);
}

.arco-modal-header {
  background: var(--gradient-primary);
  color: white;
  border-bottom: 1px solid var(--color-primary-dark-1);
}

.arco-modal-footer {
  background-color: rgba(255, 255, 255, 0.8);
  border-top: 1px solid var(--border-base-color);
}

/* ========================================================= */
/* ==================== 深色模式 Dark Mode ================= */
/* ========================================================= */

[data-theme='dark'] {
  /* 主色调 */
  --color-primary-base: #60a5fa;
  --color-primary: var(--color-primary-base);
  --primary: var(--color-primary-base);
  --color-primary-light-1: #93c5fd;
  --color-primary-light-2: #bfdbfe;
  --color-primary-light-3: #dbeafe;
  --color-primary-dark-1: #3b82f6;
  --primary-rgb: 96, 165, 250;

  /* 品牌色 */
  --brand: var(--color-primary-base);
  --brand-light: #1e3a5a;
  --brand-hover: var(--color-primary-light-1);
  --color-brand-fill: var(--color-primary-base);
  --color-brand-bg: #1e3a5a;

  /* AOU 品牌色板 */
  --aou-1: #0f1729;
  --aou-2: #1e2a47;
  --aou-3: #1e3a5a;
  --aou-4: #2d4a6f;
  --aou-5: #3d5a8f;
  --aou-6: #60a5fa;
  --aou-7: #93c5fd;
  --aou-8: #bfdbfe;
  --aou-9: #dbeafe;
  --aou-10: #eff6ff;

  /* 背景色 */
  --bg-base-color: #0f1729;
  --color-bg-1: var(--bg-base-color);
  --bg-1: var(--bg-base-color);
  --color-bg-2: #1a2332;
  --bg-2: #1a2332;
  --color-bg-3: #1e3a5a;
  --bg-3: #1e3a5a;
  --color-bg-4: #2d4a6f;
  --bg-4: #2d4a6f;
  --bg-base: #0a0f1a;
  --bg-hover: #1a2332;
  --bg-active: #1e3a5a;
  --fill: var(--bg-base-color);
  --color-fill: var(--bg-base-color);

  /* 文字色 */
  --text-base-color: #e0f2fe;
  --color-text-1: var(--text-base-color);
  --text-primary: var(--text-base-color);
  --color-text-2: #bfdbfe;
  --text-secondary: #bfdbfe;
  --color-text-3: #93c5fd;
  --text-disabled: #93c5fd;
  --text-0: #ffffff;

  /* 边框色 */
  --border-base-color: #3d5a8f;
  --color-border: var(--border-base-color);
  --color-border-1: var(--border-base-color);
  --color-border-2: #2d4a6f;
  --border-base: var(--border-base-color);
  --border-light: #2d4a6f;

  /* 语义色 */
  --success: #34d399;
  --warning: #fbbf24;
  --danger: #f87171;
  --info: var(--color-primary-base);

  /* 消息背景色 */
  --message-user-bg: #2d4a6f;
  --message-tips-bg: #1e3a5a;
  --workspace-btn-bg: #1a2332;

  /* 对话框颜色 */
  --dialog-fill-0: rgba(15, 23, 41, 0.95);

  /* 渐变 */
  --gradient-primary: linear-gradient(135deg, #3b82f6 0%, #60a5fa 100%);
  --gradient-primary-hover: linear-gradient(135deg, #60a5fa 0%, #93c5fd 100%);
}

/* 深色模式侧边栏 */
[data-theme='dark'] .layout-sider {
  background: linear-gradient(180deg, #1e3a5a 0%, #1a2332 100%);
  border-right: 3px solid var(--color-primary-base);
  box-shadow: 4px 0 20px rgba(96, 165, 250, 0.2);
}

[data-theme='dark'] .layout-sider-header {
  background: var(--gradient-primary);
  box-shadow: 0 4px 12px rgba(96, 165, 250, 0.4);
  border-bottom: 2px solid rgba(255, 255, 255, 0.2);
}

/* 深色模式图标 */
[data-theme='dark'] svg:not([class*='model'] svg),
[data-theme='dark'] .theme-icon svg {
  stroke: var(--color-primary-base);
  color: var(--color-primary-base);
}

[data-theme='dark'] svg:not([class*='model'] svg):hover,
[data-theme='dark'] .theme-icon svg:hover {
  stroke: var(--color-primary-light-1);
  color: var(--color-primary-light-1);
  filter: drop-shadow(0 0 8px rgba(147, 197, 253, 0.6));
}

[data-theme='dark'] button:not([class*='model']) svg,
[data-theme='dark'] .arco-btn:not([class*='model']) svg {
  stroke: var(--color-primary-base);
  color: var(--color-primary-base);
}

[data-theme='dark'] button:not([class*='model']) svg:hover,
[data-theme='dark'] .arco-btn:not([class*='model']) svg:hover {
  stroke: var(--color-primary-light-1);
  color: var(--color-primary-light-1);
  filter: drop-shadow(0 0 8px rgba(147, 197, 253, 0.6));
}

/* 深色模式背景图 */
[data-theme='dark'] .layout-content.bg-1::before {
  background: linear-gradient(135deg, rgba(15, 23, 41, 0.8) 0%, rgba(30, 58, 90, 0.85) 50%, rgba(15, 23, 41, 0.8) 100%);
}

[data-theme='dark'] [class*='chat-layout'] .arco-layout-content::before,
[data-theme='dark'] [class*='conversation'] .arco-layout-content::before {
  opacity: 0.2;
  filter: brightness(1.1) saturate(1.3) hue-rotate(-10deg);
}

/* 深色模式输入框 */
[data-theme='dark'] .guidInputCard {
  background: linear-gradient(135deg, rgba(30, 58, 90, 0.9) 0%, rgba(45, 74, 111, 0.9) 100%);
  border: 3px solid var(--color-primary-base);
  box-shadow:
    0 8px 32px rgba(96, 165, 250, 0.25),
    inset 0 1px 0 rgba(255, 255, 255, 0.1);
}

[data-theme='dark'] .guidInputCard textarea,
[data-theme='dark'] [class*='guidInputCard'] textarea {
  background-color: rgba(30, 58, 90, 0.8);
  color: var(--color-text-1);
}

[data-theme='dark'] .sendbox-container:not([class*='model']),
[data-theme='dark']
  [class*='sendbox']:not([class*='input']):not([class*='textarea']):not([class*='model']):not([class*='tools']) {
  border: 3px solid var(--color-primary-base);
  background: linear-gradient(135deg, rgba(30, 58, 90, 0.85) 0%, rgba(45, 74, 111, 0.85) 100%);
  box-shadow:
    0 8px 24px rgba(96, 165, 250, 0.3),
    inset 0 1px 0 rgba(255, 255, 255, 0.1);
}

[data-theme='dark'] .sendbox-container:focus-within,
[data-theme='dark'] [class*='sendbox']:focus-within {
  border-color: var(--color-primary-light-1);
  box-shadow:
    0 8px 32px rgba(147, 197, 253, 0.5),
    0 0 20px rgba(96, 165, 250, 0.4);
  transform: translateY(-2px);
}

[data-theme='dark'] .sendbox-container svg:not([class*='model'] svg),
[data-theme='dark'] [class*='sendbox']:not([class*='model']) svg {
  stroke: var(--color-primary-base);
  color: var(--color-primary-base);
}

[data-theme='dark'] .sendbox-container svg:not([class*='model'] svg):hover,
[data-theme='dark'] [class*='sendbox']:not([class*='model']) svg:hover {
  stroke: var(--color-primary-light-1);
  color: var(--color-primary-light-1);
  filter: drop-shadow(0 0 8px rgba(147, 197, 253, 0.8));
}

/* 深色模式消息气泡 */
[data-theme='dark'] .message-item.user .message-bubble,
[data-theme='dark'] [class*='message'][class*='user'] .message-content {
  background: linear-gradient(135deg, #3b82f6 0%, #60a5fa 50%, #93c5fd 100%);
  box-shadow:
    0 6px 20px rgba(96, 165, 250, 0.5),
    0 0 0 2px rgba(147, 197, 253, 0.3);
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
}

[data-theme='dark'] .message-item.ai .message-bubble,
[data-theme='dark'] [class*='message'][class*='ai'] .message-content,
[data-theme='dark'] [class*='message'][class*='assistant'] .message-content {
  background: linear-gradient(135deg, rgba(30, 58, 90, 0.9) 0%, rgba(45, 74, 111, 0.9) 100%);
  border: 2px solid var(--border-base-color);
  box-shadow:
    0 6px 20px rgba(96, 165, 250, 0.2),
    inset 0 1px 0 rgba(255, 255, 255, 0.1);
}

[data-theme='dark'] .message-item.ai .arco-alert,
[data-theme='dark'] [class*='message'][class*='ai'] .arco-alert,
[data-theme='dark'] [class*='message'][class*='assistant'] .arco-alert,
[data-theme='dark'] .message-item.ai [class*='alert'],
[data-theme='dark'] [class*='message'][class*='ai'] [class*='alert'],
[data-theme='dark'] [class*='message'][class*='assistant'] [class*='alert'] {
  background-color: rgba(30, 42, 71, 0.7);
  border: 1px solid var(--border-base-color);
}

[data-theme='dark'] .message-item.ai .arco-card,
[data-theme='dark'] [class*='message'][class*='ai'] .arco-card,
[data-theme='dark'] [class*='message'][class*='assistant'] .arco-card,
[data-theme='dark'] .message-item.ai [class*='card'],
[data-theme='dark'] [class*='message'][class*='ai'] [class*='card'],
[data-theme='dark'] [class*='message'][class*='assistant'] [class*='card'] {
  background-color: rgba(30, 42, 71, 0.7);
  border: 1px solid var(--border-base-color);
}

[data-theme='dark'] .message-item.ai [class*='status']:not([class*='message']):not([class*='bubble']),
[data-theme='dark'] [class*='message'][class*='ai'] [class*='status']:not([class*='message']):not([class*='bubble']) {
  background-color: rgba(30, 58, 90, 0.9);
  border: 1px solid var(--border-base-color);
}

/* 深色模式按钮 */
[data-theme='dark'] .arco-btn-primary:not([class*='icon']):not([class*='circle']):not([class*='model']),
[data-theme='dark'] button[type='primary']:not([class*='icon']):not([class*='circle']):not([class*='model']) {
  background: var(--gradient-primary);
  border-color: var(--color-primary-dark-1);
  box-shadow: 0 4px 12px rgba(96, 165, 250, 0.4);
}

[data-theme='dark'] .arco-btn-primary:hover:not([class*='icon']):not([class*='circle']):not([class*='model']),
[data-theme='dark'] button[type='primary']:hover:not([class*='icon']):not([class*='circle']):not([class*='model']) {
  background: var(--gradient-primary-hover);
  box-shadow:
    0 8px 24px rgba(147, 197, 253, 0.6),
    0 0 20px rgba(96, 165, 250, 0.5);
  transform: translateY(-2px);
}

/* 深色模式滚动条 */
[data-theme='dark'] *:hover::-webkit-scrollbar-thumb {
  background: rgba(96, 165, 250, 0.5);
}

[data-theme='dark'] *:hover::-webkit-scrollbar-thumb:hover {
  background: var(--gradient-primary-hover);
  box-shadow: 0 0 8px rgba(96, 165, 250, 0.6);
}

/* 深色模式选中文字 */
[data-theme='dark'] ::selection {
  background-color: var(--color-primary-dark-1);
  text-shadow: 0 0 4px rgba(59, 130, 246, 0.5);
}

/* 深色模式链接 */
[data-theme='dark'] a:not([class*='button']):not([class*='btn']) {
  color: var(--color-primary-base);
  text-decoration-color: rgba(96, 165, 250, 0.4);
}

[data-theme='dark'] a:hover:not([class*='button']):not([class*='btn']) {
  color: var(--color-primary-light-1);
  text-shadow: 0 0 8px rgba(147, 197, 253, 0.5);
}

/* 深色模式对话框 */
[data-theme='dark'] .arco-modal,
[data-theme='dark'] .arco-modal-wrapper {
  color: var(--text-base-color);
}

[data-theme='dark'] .arco-modal-body {
  background: linear-gradient(135deg, rgba(30, 58, 90, 0.98) 0%, rgba(45, 74, 111, 0.98) 100%);
  backdrop-filter: blur(20px);
  color: var(--text-base-color);
}

[data-theme='dark'] .arco-modal-header {
  background: var(--gradient-primary);
  border-bottom: 2px solid rgba(255, 255, 255, 0.2);
  box-shadow: 0 2px 8px rgba(96, 165, 250, 0.3);
}

[data-theme='dark'] .arco-modal-footer {
  background: linear-gradient(135deg, rgba(30, 58, 90, 0.98) 0%, rgba(45, 74, 111, 0.98) 100%);
  border-top: 2px solid var(--color-primary-base);
}

/* 深色模式表单 */
[data-theme='dark'] .arco-form-label,
[data-theme='dark'] .arco-form-label-item,
[data-theme='dark'] label {
  color: var(--color-text-2);
}

[data-theme='dark'] .arco-input,
[data-theme='dark'] .arco-textarea,
[data-theme='dark'] .arco-select-view,
[data-theme='dark'] input:not([type='checkbox']):not([type='radio']):not([type='button']),
[data-theme='dark'] textarea {
  background-color: rgba(30, 58, 90, 0.6);
  border: 2px solid var(--color-primary-base);
  color: var(--text-base-color);
}

[data-theme='dark'] .arco-input:hover,
[data-theme='dark'] .arco-textarea:hover,
[data-theme='dark'] input:not([type='checkbox']):not([type='radio']):hover,
[data-theme='dark'] textarea:hover {
  background-color: rgba(45, 74, 111, 0.7);
  border-color: var(--color-primary-light-1);
}

[data-theme='dark'] .arco-input:focus,
[data-theme='dark'] .arco-textarea:focus,
[data-theme='dark'] input:not([type='checkbox']):not([type='radio']):focus,
[data-theme='dark'] textarea:focus {
  background-color: rgba(45, 74, 111, 0.8);
  border-color: var(--color-primary-light-1);
  box-shadow: 0 0 0 3px rgba(147, 197, 253, 0.3);
}

[data-theme='dark'] .arco-input::placeholder,
[data-theme='dark'] .arco-textarea::placeholder,
[data-theme='dark'] input::placeholder,
[data-theme='dark'] textarea::placeholder {
  color: var(--color-text-3);
  opacity: 0.5;
}

/* 深色模式开关 */
[data-theme='dark'] .arco-switch {
  background-color: var(--border-base-color);
}

[data-theme='dark'] .arco-switch-checked {
  background-color: var(--color-primary-base);
}

/* 深色模式文字 */
[data-theme='dark'] .arco-typography,
[data-theme='dark'] p,
[data-theme='dark'] span:not([class*='icon']) {
  color: var(--color-text-2);
}

/* 深色模式分割线 */
[data-theme='dark'] .arco-divider {
  border-color: var(--color-primary-base);
  opacity: 0.3;
}
`,ie=`/* ========================================
   Hello Kitty 主题 - 优化版
   粉色系可爱风格，支持明暗双模式
   ======================================== */

/* ==================== 明亮模式 (Light Mode) ==================== */
:root {
  /* ===== 主色调 - Primary ===== */
  --hk-primary: #ff85a2;
  --hk-primary-light: #ffb7c5;
  --hk-primary-lighter: #ffe4e8;
  --hk-primary-lightest: #fff0f3;
  --hk-primary-dark: #e06b88;
  --hk-primary-darker: #c95a75;
  --hk-primary-rgb: 255, 133, 162;

  /* ===== 品牌色板渐变 ===== */
  --hk-shade-1: #fff0f3;
  --hk-shade-2: #ffe4e8;
  --hk-shade-3: #ffcad4;
  --hk-shade-4: #ffb7c5;
  --hk-shade-5: #ff9db6;
  --hk-shade-6: #ff85a2;
  --hk-shade-7: #e06b88;
  --hk-shade-8: #c95a75;
  --hk-shade-9: #a84a62;
  --hk-shade-10: #8c3d4f;

  /* ===== 背景色 ===== */
  --hk-bg-base: #ffffff;
  --hk-bg-1: #fff0f3;
  --hk-bg-2: #ffffff;
  --hk-bg-3: #ffe4e8;
  --hk-bg-4: #ffb7c5;
  --hk-bg-hover: #ffe4e8;
  --hk-bg-active: #ffcad4;

  /* ===== 文字色 ===== */
  --hk-text-primary: #5a3e45;
  --hk-text-secondary: #8c6b74;
  --hk-text-tertiary: #bfa5ac;
  --hk-text-disabled: #d4c0c6;
  --hk-text-inverse: #ffffff;

  /* ===== 边框色 ===== */
  --hk-border-base: #ffcad4;
  --hk-border-light: #ffe4e8;
  --hk-border-strong: #ffb7c5;

  /* ===== 语义色 ===== */
  --hk-success: #52c41a;
  --hk-warning: #faad14;
  --hk-error: #f5222d;
  --hk-info: #ff85a2;

  /* ===== 阴影 ===== */
  --hk-shadow-sm: 0 2px 8px rgba(255, 133, 162, 0.15);
  --hk-shadow-md: 0 4px 16px rgba(255, 133, 162, 0.2);
  --hk-shadow-lg: 0 8px 24px rgba(255, 133, 162, 0.25);
  --hk-shadow-glow: 0 0 20px rgba(255, 133, 162, 0.3);

  /* ===== 渐变 ===== */
  --hk-gradient-primary: linear-gradient(135deg, #ff85a2 0%, #ff9db6 100%);
  --hk-gradient-light: linear-gradient(135deg, #fff0f3 0%, #ffe4e8 100%);
  --hk-gradient-button: linear-gradient(135deg, #ff85a2 0%, #ffb7c5 100%);

  /* ===== 映射到系统变量 ===== */
  --color-primary: var(--hk-primary);
  --primary: var(--hk-primary);
  --brand: var(--hk-primary);
  --color-bg-1: var(--hk-bg-1);
  --bg-1: var(--hk-bg-1);
  --color-bg-2: var(--hk-bg-2);
  --bg-2: var(--hk-bg-2);
  --color-text-1: var(--hk-text-primary);
  --text-primary: var(--hk-text-primary);
  --color-text-2: var(--hk-text-secondary);
  --text-secondary: var(--hk-text-secondary);
  --color-border: var(--hk-border-base);
  --border-base: var(--hk-border-base);
  --success: var(--hk-success);
  --warning: var(--hk-warning);
  --danger: var(--hk-error);
  --info: var(--hk-info);
}

/* ===== 字体设置 ===== */
body {
  font-family: 'Varela Round', 'Nunito', 'PingFang SC', 'Microsoft YaHei', sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

/* ===== 全局背景 ===== */
body,
html {
  background-color: var(--hk-bg-1);
  color: var(--hk-text-primary);
}

.arco-layout,
[class*='layout'] {
  background-color: var(--hk-bg-1);
}

.arco-layout-content {
  background-color: var(--hk-bg-1);
}

/* ===== 背景图设置 ===== */
.layout-content.bg-1 {
  position: relative;
  background-color: var(--hk-bg-1);
}

/* 半透明遮罩层 */
.layout-content.bg-1::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(
    135deg,
    rgba(240, 249, 255, 0.75) 0%,
    rgba(224, 242, 254, 0.8) 50%,
    rgba(240, 249, 255, 0.75) 100%
  );
  z-index: 0;
  pointer-events: none;
}

.layout-content.bg-1 > * {
  position: relative;
  z-index: 1;
}

/* 聊天页面背景 */
[class*='chat-layout'] .arco-layout-content,
[class*='conversation'] .arco-layout-content {
  position: relative;
}

[class*='chat-layout'] .arco-layout-content::before,
[class*='conversation'] .arco-layout-content::before {
  content: '';
  position: absolute;
  inset: 0;
  background: transparent;
  opacity: 0;
  z-index: 0;
  pointer-events: none;
}

[class*='chat-layout'] .arco-layout-content > *,
[class*='conversation'] .arco-layout-content > * {
  position: relative;
  z-index: 1;
}

/* ==================== 侧边栏 Sidebar ==================== */
.layout-sider {
  background-color: var(--hk-bg-1);
  border-right: 2px solid var(--hk-border-strong);
}

.layout-sider-header {
  background: var(--hk-gradient-primary);
  color: var(--hk-text-inverse);
  box-shadow: var(--hk-shadow-sm);
  border-bottom: 1px solid rgba(255, 255, 255, 0.2);
}

/* 侧边栏图标 */
.layout-sider-header svg {
  color: rgba(255, 255, 255, 0.9);
  transition:
    color 0.3s ease,
    transform 0.2s ease;
}

.layout-sider-header svg:hover {
  color: var(--hk-text-inverse);
  transform: scale(1.1);
}

/* ==================== 输入框 Input ==================== */
/* 首页输入框 */
.guidInputCard {
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(12px);
  border: 2px solid var(--hk-border-strong);
  border-radius: 20px;
  box-shadow: var(--hk-shadow-md);
  transition: all 0.3s ease;
}

.guidInputCard:hover {
  border-color: var(--hk-primary);
  box-shadow: var(--hk-shadow-lg);
}

.guidInputCard textarea {
  background-color: transparent;
  color: var(--hk-text-primary);
  border: none;
}

/* 发送框 */
.sendbox-container {
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(12px);
  border: 2px solid var(--hk-border-strong);
  border-radius: 24px;
  box-shadow: var(--hk-shadow-md);
  transition: all 0.3s ease;
}

.sendbox-container:focus-within {
  border-color: var(--hk-primary);
  box-shadow: var(--hk-shadow-lg), var(--hk-shadow-glow);
  transform: translateY(-1px);
}

.sendbox-container textarea {
  background: transparent;
  border: none;
  color: var(--hk-text-primary);
}

/* ==================== 消息气泡 Message ==================== */
/* 用户消息 */
.message-item.user .message-bubble,
[class*='message-user'] .message-content {
  background: var(--hk-gradient-primary);
  color: var(--hk-text-inverse);
  border-radius: 20px 20px 4px 20px;
  box-shadow: var(--hk-shadow-md);
  padding: 12px 18px;
  border: none;
}

/* AI 消息 */
.message-item.ai .message-bubble,
.message-item.assistant .message-bubble,
[class*='message-ai'] .message-content,
[class*='message-assistant'] .message-content {
  background: rgba(255, 255, 255, 0.98);
  backdrop-filter: blur(10px);
  border: 2px solid var(--hk-border-light);
  border-radius: 20px 20px 20px 4px;
  box-shadow: var(--hk-shadow-sm);
  padding: 12px 18px;
  color: var(--hk-text-primary);
}

/* 工具调用提示 - 保持简洁 */
.message-item.ai .arco-alert,
.message-item.assistant .arco-alert {
  background-color: rgba(255, 255, 255, 0.7);
  border: 1px solid var(--hk-border-light);
  border-radius: 8px;
  margin: 8px 0;
}

.message-item.ai .arco-card,
.message-item.assistant .arco-card {
  background-color: rgba(255, 255, 255, 0.7);
  border: 1px solid var(--hk-border-light);
  border-radius: 8px;
  margin: 8px 0;
}

/* ==================== 按钮 Button ==================== */
.arco-btn-primary,
button[type='primary'] {
  background: var(--hk-gradient-button);
  border: none;
  border-radius: 20px;
  color: var(--hk-text-inverse);
  font-weight: 600;
  box-shadow: var(--hk-shadow-sm);
  transition: all 0.3s ease;
}

.arco-btn-primary:hover,
button[type='primary']:hover {
  background: linear-gradient(135deg, #ff9db6 0%, #ffcad4 100%);
  box-shadow: var(--hk-shadow-md), var(--hk-shadow-glow);
  transform: translateY(-2px);
}

.arco-btn-primary:active,
button[type='primary']:active {
  transform: translateY(0);
  box-shadow: var(--hk-shadow-sm);
}

.arco-btn-secondary,
button[type='secondary'] {
  background: transparent;
  border: 2px solid var(--hk-primary);
  border-radius: 20px;
  color: var(--hk-primary);
  font-weight: 600;
  transition: all 0.3s ease;
}

.arco-btn-secondary:hover,
button[type='secondary']:hover {
  background: var(--hk-bg-hover);
  border-color: var(--hk-primary-light);
  color: var(--hk-primary-light);
  transform: translateY(-1px);
}

/* 按钮禁用状态 */
.arco-btn:disabled,
button:disabled {
  background: var(--hk-bg-3);
  color: var(--hk-text-disabled);
  border-color: var(--hk-border-light);
  cursor: not-allowed;
  opacity: 0.6;
}

/* ==================== 图标 Icon ==================== */
/* 基础图标颜色 - 仅针对需要的图标 */
.arco-icon {
  color: var(--hk-primary);
  transition:
    color 0.3s ease,
    transform 0.2s ease;
}

.arco-icon:hover {
  color: var(--hk-primary-light);
  transform: scale(1.1);
}

/* 按钮内图标 */
.arco-btn-primary .arco-icon,
button[type='primary'] .arco-icon {
  color: var(--hk-text-inverse);
}

/* ==================== 滚动条 Scrollbar ==================== */
::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  background: transparent;
  border-radius: 4px;
}

::-webkit-scrollbar-thumb {
  background: transparent;
  border-radius: 4px;
  transition: background 0.3s ease;
}

*:hover::-webkit-scrollbar-thumb {
  background: rgba(255, 133, 162, 0.3);
}

*:hover::-webkit-scrollbar-thumb:hover {
  background: var(--hk-gradient-primary);
}

/* ==================== 其他元素 ==================== */
/* 链接 */
a {
  color: var(--hk-primary);
  text-decoration: none;
  transition: color 0.3s ease;
}

a:hover {
  color: var(--hk-primary-light);
  text-decoration: underline;
}

/* 选中文本 */
::selection {
  background-color: var(--hk-primary);
  color: var(--hk-text-inverse);
}

/* Tooltip */
/* Modal 对话框 */
.arco-modal-header {
  background: var(--hk-gradient-light);
  border-bottom: 1px solid var(--hk-border-light);
  color: var(--hk-text-primary);
}

.arco-modal-body {
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(10px);
  color: var(--hk-text-primary);
}

.arco-modal-footer {
  background: rgba(255, 255, 255, 0.95);
  border-top: 1px solid var(--hk-border-light);
}

/* ==================== 深色模式 (Dark Mode) ==================== */
[data-theme='dark'] {
  /* ===== 主色调 - 调亮以提高可见度 ===== */
  --hk-primary: #ffb7c5;
  --hk-primary-light: #ffcad4;
  --hk-primary-lighter: #ffe4e8;
  --hk-primary-lightest: #fff0f3;
  --hk-primary-dark: #ff9db6;
  --hk-primary-darker: #ff85a2;
  --hk-primary-rgb: 255, 183, 197;

  /* ===== 品牌色板 - 深色模式反转 ===== */
  --hk-shade-1: #2d1a24;
  --hk-shade-2: #3d2431;
  --hk-shade-3: #4a2f3a;
  --hk-shade-4: #5d3b4a;
  --hk-shade-5: #7a4d5f;
  --hk-shade-6: #ffb7c5;
  --hk-shade-7: #ffcad4;
  --hk-shade-8: #ffe4e8;
  --hk-shade-9: #fff0f3;
  --hk-shade-10: #fff5f7;

  /* ===== 背景色 - 温暖的深粉紫色 ===== */
  --hk-bg-base: #1f1119;
  --hk-bg-1: #2d1a24;
  --hk-bg-2: #3d2431;
  --hk-bg-3: #4a2f3a;
  --hk-bg-4: #5d3b4a;
  --hk-bg-hover: #3d2431;
  --hk-bg-active: #4a2f3a;

  /* ===== 文字色 - 高对比度粉白色 ===== */
  --hk-text-primary: #fff0f3;
  --hk-text-secondary: #ffcad4;
  --hk-text-tertiary: #ff9db6;
  --hk-text-disabled: #8c6b74;
  --hk-text-inverse: #ffffff;

  /* ===== 边框色 ===== */
  --hk-border-base: #7a4d5f;
  --hk-border-light: #5d3b4a;
  --hk-border-strong: #ff9db6;

  /* ===== 语义色 - 深色模式调整 ===== */
  --hk-success: #95de64;
  --hk-warning: #ffc53d;
  --hk-error: #ff7875;
  --hk-info: #ffb7c5;

  /* ===== 阴影 - 增强发光效果 ===== */
  --hk-shadow-sm: 0 2px 12px rgba(255, 183, 197, 0.2);
  --hk-shadow-md: 0 4px 20px rgba(255, 183, 197, 0.3);
  --hk-shadow-lg: 0 8px 32px rgba(255, 183, 197, 0.4);
  --hk-shadow-glow: 0 0 24px rgba(255, 183, 197, 0.4);

  /* ===== 渐变 - 深色模式保持鲜艳 ===== */
  --hk-gradient-primary: linear-gradient(135deg, #ff85a2 0%, #ffb7c5 100%);
  --hk-gradient-light: linear-gradient(135deg, #4a2f3a 0%, #5d3b4a 100%);
  --hk-gradient-button: linear-gradient(135deg, #ff85a2 0%, #ffb7c5 100%);

  /* ===== 重新映射系统变量 ===== */
  --color-primary: var(--hk-primary);
  --primary: var(--hk-primary);
  --brand: var(--hk-primary);
  --color-bg-1: var(--hk-bg-1);
  --bg-1: var(--hk-bg-1);
  --color-bg-2: var(--hk-bg-2);
  --bg-2: var(--hk-bg-2);
  --bg-hover: var(--hk-bg-hover);
  --bg-active: var(--hk-bg-active);
  --color-text-1: var(--hk-text-primary);
  --text-primary: var(--hk-text-primary);
  --color-text-2: var(--hk-text-secondary);
  --text-secondary: var(--hk-text-secondary);
  --color-border: var(--hk-border-base);
  --border-base: var(--hk-border-base);
}

/* ===== 深色模式全局样式 ===== */
[data-theme='dark'] body,
[data-theme='dark'] html {
  background-color: var(--hk-bg-1);
  color: var(--hk-text-primary);
}

/* ===== 深色模式标题栏 ===== */
[data-theme='dark'] .app-titlebar {
  background-color: var(--hk-bg-2);
  border-color: var(--hk-border-base);
}

[data-theme='dark'] .app-titlebar__button {
  color: var(--hk-text-primary);
}

[data-theme='dark'] .app-titlebar__button:hover {
  background-color: var(--hk-bg-hover);
  color: var(--hk-primary);
}

[data-theme='dark'] .app-titlebar__brand {
  color: var(--hk-text-primary);
}

[data-theme='dark'] .arco-layout,
[data-theme='dark'] [class*='layout'] {
  background-color: var(--hk-bg-1);
}

[data-theme='dark'] .arco-layout-content {
  background-color: var(--hk-bg-1);
}

/* ===== 深色模式背景图 ===== */
[data-theme='dark'] .layout-content.bg-1::before {
  background: linear-gradient(
    135deg,
    rgba(45, 26, 36, 0.85) 0%,
    rgba(61, 36, 49, 0.9) 50%,
    rgba(45, 26, 36, 0.85) 100%
  );
}

[data-theme='dark'] [class*='chat-layout'] .arco-layout-content::before,
[data-theme='dark'] [class*='conversation'] .arco-layout-content::before {
  opacity: 0.2;
  filter: brightness(0.9) saturate(1.2);
}

/* ===== 深色模式侧边栏 ===== */
[data-theme='dark'] .layout-sider {
  background: linear-gradient(180deg, #4a2f3a 0%, #3d2431 100%);
  border-right: 2px solid var(--hk-border-strong);
  box-shadow: 4px 0 20px rgba(255, 183, 197, 0.15);
}

[data-theme='dark'] .layout-sider-header {
  background: var(--hk-gradient-primary);
  box-shadow: 0 4px 16px rgba(255, 183, 197, 0.4);
  border-bottom: 1px solid rgba(255, 255, 255, 0.3);
}

/* ===== 深色模式输入框 ===== */
[data-theme='dark'] .guidInputCard {
  background: linear-gradient(135deg, rgba(74, 47, 58, 0.95) 0%, rgba(93, 59, 74, 0.95) 100%);
  backdrop-filter: blur(16px);
  border: 2px solid var(--hk-border-strong);
  box-shadow:
    var(--hk-shadow-md),
    inset 0 1px 0 rgba(255, 255, 255, 0.05);
}

[data-theme='dark'] .guidInputCard:hover {
  border-color: var(--hk-primary);
  box-shadow: var(--hk-shadow-lg), var(--hk-shadow-glow);
}

[data-theme='dark'] .guidInputCard textarea {
  color: var(--hk-text-primary);
}

[data-theme='dark'] .sendbox-container {
  background: linear-gradient(135deg, rgba(74, 47, 58, 0.95) 0%, rgba(93, 59, 74, 0.95) 100%);
  backdrop-filter: blur(16px);
  border: 2px solid var(--hk-border-strong);
  box-shadow:
    var(--hk-shadow-md),
    inset 0 1px 0 rgba(255, 255, 255, 0.05);
}

[data-theme='dark'] .sendbox-container:focus-within {
  border-color: var(--hk-primary);
  box-shadow: var(--hk-shadow-lg), var(--hk-shadow-glow);
}

[data-theme='dark'] .sendbox-container textarea {
  color: var(--hk-text-primary);
}

/* ===== 深色模式消息气泡 ===== */
[data-theme='dark'] .message-item.user .message-bubble,
[data-theme='dark'] [class*='message-user'] .message-content {
  background: var(--hk-gradient-primary);
  color: var(--hk-text-inverse);
  box-shadow:
    var(--hk-shadow-md),
    0 0 0 1px rgba(255, 255, 255, 0.1);
}

[data-theme='dark'] .message-item.ai .message-bubble,
[data-theme='dark'] .message-item.assistant .message-bubble,
[data-theme='dark'] [class*='message-ai'] .message-content,
[data-theme='dark'] [class*='message-assistant'] .message-content {
  background: linear-gradient(135deg, rgba(74, 47, 58, 0.95) 0%, rgba(93, 59, 74, 0.95) 100%);
  backdrop-filter: blur(10px);
  border: 2px solid var(--hk-border-base);
  box-shadow:
    var(--hk-shadow-sm),
    inset 0 1px 0 rgba(255, 255, 255, 0.05);
  color: var(--hk-text-primary);
}

[data-theme='dark'] .message-item.ai .arco-alert,
[data-theme='dark'] .message-item.assistant .arco-alert {
  background-color: rgba(61, 36, 49, 0.8);
  border-color: var(--hk-border-light);
}

[data-theme='dark'] .message-item.ai .arco-card,
[data-theme='dark'] .message-item.assistant .arco-card {
  background-color: rgba(61, 36, 49, 0.8);
  border-color: var(--hk-border-light);
}

/* ===== 深色模式按钮 ===== */
[data-theme='dark'] .arco-btn-primary,
[data-theme='dark'] button[type='primary'] {
  background: var(--hk-gradient-primary);
  box-shadow: var(--hk-shadow-md);
}

[data-theme='dark'] .arco-btn-primary:hover,
[data-theme='dark'] button[type='primary']:hover {
  background: linear-gradient(135deg, #ffb7c5 0%, #ffcad4 100%);
  box-shadow: var(--hk-shadow-lg), var(--hk-shadow-glow);
}

[data-theme='dark'] .arco-btn-secondary,
[data-theme='dark'] button[type='secondary'] {
  border-color: var(--hk-primary);
  color: var(--hk-primary);
}

[data-theme='dark'] .arco-btn-secondary:hover,
[data-theme='dark'] button[type='secondary']:hover {
  background: var(--hk-bg-hover);
  border-color: var(--hk-primary-light);
  color: var(--hk-primary-light);
  box-shadow: 0 0 12px rgba(255, 183, 197, 0.3);
}

/* ===== 深色模式图标 ===== */
[data-theme='dark'] .arco-icon {
  color: var(--hk-primary);
}

[data-theme='dark'] .arco-icon:hover {
  color: var(--hk-primary-light);
  filter: drop-shadow(0 0 8px rgba(255, 183, 197, 0.5));
}

/* ===== 深色模式滚动条 ===== */
[data-theme='dark'] *:hover::-webkit-scrollbar-thumb {
  background: rgba(255, 183, 197, 0.4);
}

[data-theme='dark'] *:hover::-webkit-scrollbar-thumb:hover {
  background: var(--hk-gradient-primary);
  box-shadow: 0 0 8px rgba(255, 183, 197, 0.5);
}

/* ===== 深色模式其他元素 ===== */
[data-theme='dark'] a {
  color: var(--hk-primary);
}

[data-theme='dark'] a:hover {
  color: var(--hk-primary-light);
  text-shadow: 0 0 8px rgba(255, 183, 197, 0.4);
}

[data-theme='dark'] ::selection {
  background-color: var(--hk-primary);
  color: var(--hk-text-inverse);
}

[data-theme='dark'] .arco-modal-header {
  background: var(--hk-gradient-primary);
  color: var(--hk-text-inverse);
  border-bottom: 1px solid rgba(255, 255, 255, 0.2);
}

[data-theme='dark'] .arco-modal-body {
  background: linear-gradient(135deg, rgba(61, 36, 49, 0.98) 0%, rgba(74, 47, 58, 0.98) 100%);
  backdrop-filter: blur(20px);
  color: var(--hk-text-primary);
}

[data-theme='dark'] .arco-modal-footer {
  background: linear-gradient(135deg, rgba(61, 36, 49, 0.98) 0%, rgba(74, 47, 58, 0.98) 100%);
  border-top: 1px solid var(--hk-border-base);
}

/* ==================== 动画效果 ==================== */
@keyframes hk-float {
  0%,
  100% {
    transform: translateY(0);
  }
  50% {
    transform: translateY(-4px);
  }
}

@keyframes hk-pulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.8;
  }
}

/* 可选：给某些元素添加悬浮动画 */
.arco-btn-primary:hover {
  animation: hk-float 2s ease-in-out infinite;
}

/* ==================== 响应式调整 ==================== */
@media (max-width: 768px) {
  /* 移动端优化 */
  .guidInputCard,
  .sendbox-container {
    border-radius: 16px;
  }

  .message-item.user .message-bubble,
  .message-item.ai .message-bubble,
  .message-item.assistant .message-bubble {
    border-radius: 16px;
    padding: 10px 14px;
  }

  .arco-btn-primary,
  .arco-btn-secondary {
    border-radius: 16px;
    padding: 8px 16px;
  }
}

/* ==================== 打印样式 ==================== */
@media print {
  /* 打印时移除背景图和阴影 */
  .layout-content.bg-1::before,
  [class*='chat-layout'] .arco-layout-content::before {
    display: none;
  }

  * {
    box-shadow: none !important;
    text-shadow: none !important;
  }
}
`,de=`/* ========================================
   Windows Classic Theme - 优化版
   复古 Windows 配色，支持明暗双模式
   确保可读性，适度添加 Windows 经典元素
   ======================================== */

/* ==================== 明色模式 (Light Mode) ==================== */
/* 核心颜色变量 - 复古 Windows 配色 */
:root {
  /* 主色调 - Classic Windows Blue */
  --color-primary: #0078d4;
  --primary: #0078d4;
  --color-primary-light-1: #1a86d9;
  --color-primary-light-2: #3399e6;
  --color-primary-light-3: #4da6f0;
  --color-primary-dark-1: #005a9e;
  --primary-rgb: 0, 120, 212;

  /* 品牌色 - Windows Classic */
  --brand: #0078d4;
  --brand-light: #e6f2fa;
  --brand-hover: #1a86d9;
  --color-brand-fill: #0078d4;
  --color-brand-bg: #e6f2fa;

  /* AOU 品牌色板 - 蓝色系渐变 */
  --aou-1: #e6f2fa;
  --aou-2: #cce5f5;
  --aou-3: #b3d8f0;
  --aou-4: #99cbeb;
  --aou-5: #66b1e1;
  --aou-6: #0078d4;
  --aou-7: #005a9e;
  --aou-8: #004578;
  --aou-9: #003052;
  --aou-10: #001b2c;

  /* 背景色 - Classic Windows Gray/Beige */
  --color-bg-1: #f0f0f0;
  --bg-1: #f0f0f0;
  --color-bg-2: #ffffff;
  --bg-2: #ffffff;
  --color-bg-3: #e0e0e0;
  --bg-3: #e0e0e0;
  --color-bg-4: #c0c0c0;
  --bg-4: #c0c0c0;
  --bg-base: #ffffff;
  --bg-hover: #e0e0e0;
  --bg-active: #c0c0c0;
  --fill: #f0f0f0;
  --color-fill: #f0f0f0;

  /* 文字色 - Classic Windows Text */
  --color-text-1: #000000;
  --text-primary: #000000;
  --color-text-2: #404040;
  --text-secondary: #404040;
  --color-text-3: #808080;
  --text-disabled: #808080;
  --text-0: #000000;

  /* 边框色 - Classic Windows Border */
  --color-border: #808080;
  --color-border-1: #808080;
  --color-border-2: #c0c0c0;
  --border-base: #808080;
  --border-light: #c0c0c0;

  /* 语义色 - Classic Windows Colors */
  --success: #00a300; /* Windows 绿 */
  --warning: #ff8c00;
  --danger: #d13438;
  --info: #0078d4; /* Windows 蓝 */

  /* Windows 经典绿色 - 适度使用 */
  --windows-green: #00a300;
  --windows-green-light: #00c300;
  --windows-green-dark: #008000;

  /* 消息背景色 - Message Backgrounds */
  --message-user-bg: #d0e8f5;
  --message-tips-bg: #f0f0f0;
  --workspace-btn-bg: #e0e0e0;

  /* 对话框颜色 - Dialog Colors */
  --dialog-fill-0: rgba(255, 255, 255, 0.95);
}

/* 全局字体 - 经典 Windows 字体 */
body {
  font-family: 'MS Sans Serif', 'Tahoma', 'Arial', 'Microsoft YaHei', sans-serif;
}

/* 全局背景色 - 经典 Windows 米色 */
body,
html {
  background-color: var(--bg-1, #f0f0f0);
}

/* 全局主要背景区域 */
.arco-layout,
[class*='layout'] {
  background-color: var(--bg-1, #f0f0f0);
}

/* 全局内容区域背景 */
.arco-layout-content {
  background-color: var(--bg-1, #f0f0f0);
}

/* ==================== 侧边栏 Sidebar ==================== */
/* 侧边栏样式 - 只保留基础样式，其他使用系统默认 */
.layout-sider {
  background-color: #e0e0e0;
  border-right: 2px solid #808080;
  position: relative;
  z-index: 100;
}

.layout-sider.collapsed {
  overflow: hidden;
}

.layout-sider.collapsed * {
  overflow: hidden;
}

.layout-sider.collapsed::-webkit-scrollbar {
  display: none !important;
  width: 0 !important;
  height: 0 !important;
}

.layout-sider-header {
  background: linear-gradient(180deg, #0078d4 0%, #005a9e 100%);
  color: white;
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.3),
    0 1px 2px rgba(0, 0, 0, 0.2);
}

/* 按钮内的图标 - 保持原有样式，不强制设置 */
button:not(.sendbox-model-btn):not([class*='model']):not([class*='Model']) svg,
.arco-btn:not(.sendbox-model-btn):not([class*='model']):not([class*='Model']) svg {
  /* 保持图标原有样式 */
}

/* 主要按钮内的图标 - 保持原有样式 */
.arco-btn-primary:not(.sendbox-model-btn):not([class*='model']):not([class*='Model']) svg {
  /* 保持图标原有样式 */
}

/* ==================== 背景图设置 ==================== */
/* 背景图片设置 - 让背景图穿透显示 */
.layout-content.bg-1 {
  background-color: var(--bg-1, #f0f0f0);
  position: relative;
}

/* 半透明遮罩层 */
.layout-content.bg-1::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(
    135deg,
    rgba(240, 249, 255, 0.75) 0%,
    rgba(224, 242, 254, 0.8) 50%,
    rgba(240, 249, 255, 0.75) 100%
  );
  z-index: 0;
  pointer-events: none;
}

/* 聊天页面背景图 - Windows 经典配色 */
.chat-layout-header,
[class*='chat-layout'] .arco-layout-content,
[class*='conversation'] .arco-layout-content {
  position: relative;
}

[class*='chat-layout'] .arco-layout-content::before,
[class*='conversation'] .arco-layout-content::before {
  content: '';
  position: absolute;
  inset: 0;
  background: transparent;
  opacity: 0;
  z-index: 0;
  pointer-events: none;
}

/* 确保聊天内容在背景图之上 */
[class*='chat-layout'] .arco-layout-content > *,
[class*='conversation'] .arco-layout-content > * {
  position: relative;
  z-index: 1;
}

/* 确保内容在遮罩之上 */
.layout-content.bg-1 > * {
  position: relative;
  z-index: 1;
}

/* 首页对话框和输入区域 - 确保完全可见 */
.guidLayout,
[class*='guid'] {
  position: relative;
  z-index: 10;
}

/* 输入框文本域 - 确保文字清晰可见 */
.guidInputCard textarea,
[class*='guidInputCard'] textarea {
  background-color: rgba(255, 255, 255, 0.98);
  color: var(--color-text-1);
}

/* ==================== 输入框 Input ==================== */
/* 发送框样式 - 只针对可见的发送框容器，排除模型选择器等系统组件 */
.sendbox-container:not([class*='model']):not([class*='Model']),
[class*='sendbox']:not([class*='input']):not([class*='textarea']):not([class*='model']):not([class*='Model']):not(
    [class*='tools']
  ) {
  border-radius: 4px; /* 经典 Windows 方角 */
  border: 2px outset #c0c0c0; /* 经典 3D 边框效果 */
  background-color: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(4px);
  box-shadow:
    inset 1px 1px 0 rgba(255, 255, 255, 0.8),
    inset -1px -1px 0 rgba(0, 0, 0, 0.1),
    0 2px 4px rgba(0, 0, 0, 0.1);
  transition: all 0.2s ease;
}

/* 首页输入框对话框 - 白色90%不透明度，确保用户看得清 */
.guidInputCard {
  background-color: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(4px);
  border: 2px outset #c0c0c0; /* 经典 3D 边框 */
  border-radius: 4px;
  box-shadow:
    inset 1px 1px 0 rgba(255, 255, 255, 0.8),
    inset -1px -1px 0 rgba(0, 0, 0, 0.1),
    0 2px 4px rgba(0, 0, 0, 0.1);
}

/* 发送框内的文本域 - 保持原有样式，只调整边框 */
.sendbox-container textarea,
[class*='sendbox'] textarea {
  border: none;
  background: transparent;
  color: var(--color-text-1); /* 确保文字清晰 */
}

.sendbox-container:focus-within,
[class*='sendbox']:focus-within {
  border: 2px inset #808080; /* 聚焦时变为内陷效果 */
  box-shadow: inset 2px 2px 4px rgba(0, 0, 0, 0.2);
}

/* 发送框内图标颜色调整 - 排除模型选择按钮和系统组件 */
.sendbox-container svg:not(.sendbox-model-btn svg):not([class*='model'] svg),
[class*='sendbox']:not([class*='model']):not([class*='Model']) svg:not(.sendbox-model-btn svg) {
  color: #0078d4;
  transition: color 0.3s ease;
}

.sendbox-container svg:not(.sendbox-model-btn svg):not([class*='model'] svg):hover,
[class*='sendbox']:not([class*='model']):not([class*='Model']) svg:not(.sendbox-model-btn svg):hover {
  color: #1a86d9;
  transform: scale(1.1);
}

/* ==================== 消息气泡 Message ==================== */
/* 用户消息气泡 - 经典 Windows 蓝色 */
.message-item.user .message-bubble,
[class*='message'][class*='user'] .message-content {
  background: linear-gradient(180deg, #0078d4 0%, #005a9e 100%);
  color: white;
  border-radius: 4px; /* 方角 */
  border: 1px solid #005a9e;
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.3),
    0 2px 4px rgba(0, 0, 0, 0.2);
  padding: 12px 16px;
}

/* AI 消息气泡 - 经典 Windows 白色 */
.message-item.ai .message-bubble,
[class*='message'][class*='ai'] .message-content,
[class*='message'][class*='assistant'] .message-content {
  background-color: rgba(255, 255, 255, 0.98);
  backdrop-filter: blur(4px);
  border: 1px solid #c0c0c0;
  border-radius: 4px; /* 方角 */
  box-shadow:
    inset 1px 1px 0 rgba(255, 255, 255, 0.8),
    inset -1px -1px 0 rgba(0, 0, 0, 0.1),
    0 2px 4px rgba(0, 0, 0, 0.1);
  padding: 12px 16px;
  color: var(--color-text-1); /* 确保文字清晰 */
}

/* 工具调用消息 - 保持原有样式，只微调背景色以融入主题 */
.message-item.ai .arco-alert,
[class*='message'][class*='ai'] .arco-alert,
[class*='message'][class*='assistant'] .arco-alert,
.message-item.ai [class*='alert'],
[class*='message'][class*='ai'] [class*='alert'],
[class*='message'][class*='assistant'] [class*='alert'] {
  background-color: rgba(255, 255, 255, 0.9);
  border: 1px solid #c0c0c0;
  border-radius: 4px;
  box-shadow:
    inset 1px 1px 0 rgba(255, 255, 255, 0.8),
    inset -1px -1px 0 rgba(0, 0, 0, 0.1);
  backdrop-filter: none;
  margin: 4px 0;
  color: var(--color-text-1); /* 确保文字清晰 */
}

/* 工具调用卡片 - 恢复原有样式，微调 */
.message-item.ai .arco-card,
[class*='message'][class*='ai'] .arco-card,
[class*='message'][class*='assistant'] .arco-card,
.message-item.ai [class*='card'],
[class*='message'][class*='ai'] [class*='card'],
[class*='message'][class*='assistant'] [class*='card'] {
  background-color: rgba(255, 255, 255, 0.9);
  border: 1px solid #c0c0c0;
  border-radius: 4px;
  box-shadow:
    inset 1px 1px 0 rgba(255, 255, 255, 0.8),
    inset -1px -1px 0 rgba(0, 0, 0, 0.1);
  backdrop-filter: none;
  margin: 4px 0;
  color: var(--color-text-1); /* 确保文字清晰 */
}

/* 工具调用相关的内容区域 - 恢复简洁样式 */
.message-item.ai [class*='tool']:not([class*='message']):not([class*='bubble']),
[class*='message'][class*='ai'] [class*='tool']:not([class*='message']):not([class*='bubble']),
.message-item.ai [class*='Tool']:not([class*='message']):not([class*='bubble']),
[class*='message'][class*='ai'] [class*='Tool']:not([class*='message']):not([class*='bubble']),
.message-item.ai [class*='WebFetch'],
[class*='message'][class*='ai'] [class*='WebFetch'],
.message-item.ai [class*='web_search'],
[class*='message'][class*='ai'] [class*='web_search'],
.message-item.ai [class*='exec_command'],
[class*='message'][class*='ai'] [class*='exec_command'],
.message-item.ai [class*='mcp_tool'],
[class*='message'][class*='ai'] [class*='mcp_tool'] {
  background-color: transparent;
  border: none;
  border-radius: 0;
  padding: 0;
  margin: 0;
}

/* 工具调用状态标签 - 恢复简洁样式 */
.message-item.ai [class*='status']:not([class*='message']):not([class*='bubble']),
[class*='message'][class*='ai'] [class*='status']:not([class*='message']):not([class*='bubble']),
.message-item.ai [class*='Status']:not([class*='message']):not([class*='bubble']),
[class*='message'][class*='ai'] [class*='Status']:not([class*='message']):not([class*='bubble']) {
  background-color: rgba(255, 255, 255, 0.95);
  border: 1px solid #c0c0c0;
  border-radius: 4px;
  padding: 2px 6px;
  color: var(--color-text-1); /* 确保文字清晰 */
}

/* ==================== 按钮 Button ==================== */
/* 主要按钮样式 - 经典 Windows 3D 按钮效果 */
.arco-btn-primary:not([class*='icon']):not([class*='circle']):not([class*='model']):not([class*='Model']),
button[type='primary']:not([class*='icon']):not([class*='circle']):not([class*='model']):not([class*='Model']) {
  background: linear-gradient(180deg, #0078d4 0%, #005a9e 100%);
  border: 2px outset #0078d4;
  border-radius: 4px; /* 方角 */
  font-weight: normal;
  color: white;
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.3),
    0 2px 4px rgba(0, 0, 0, 0.2);
  transition: all 0.2s ease;
}

.arco-btn-primary:hover:not([class*='icon']):not([class*='circle']):not([class*='model']):not([class*='Model']),
button[type='primary']:hover:not([class*='icon']):not([class*='circle']):not([class*='model']):not([class*='Model']) {
  background: linear-gradient(180deg, #1a86d9 0%, #0078d4 100%);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.4),
    0 3px 6px rgba(0, 0, 0, 0.3);
}

.arco-btn-primary:active:not([class*='icon']):not([class*='circle']):not([class*='model']):not([class*='Model']),
button[type='primary']:active:not([class*='icon']):not([class*='circle']):not([class*='model']):not([class*='Model']) {
  border: 2px inset #005a9e;
  box-shadow: inset 2px 2px 4px rgba(0, 0, 0, 0.3);
}

/* 成功状态按钮 - Windows 绿（适度使用） */
.arco-btn-success,
button[type='success'] {
  background: linear-gradient(180deg, #00a300 0%, #008000 100%);
  border: 2px outset #00a300;
  border-radius: 4px;
  color: white;
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.3),
    0 2px 4px rgba(0, 0, 0, 0.2);
  transition: all 0.2s ease;
}

.arco-btn-success:hover,
button[type='success']:hover {
  background: linear-gradient(180deg, #00c300 0%, #00a300 100%);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.4),
    0 3px 6px rgba(0, 0, 0, 0.3);
}

.arco-btn-success:active,
button[type='success']:active {
  border: 2px inset #008000;
  box-shadow: inset 2px 2px 4px rgba(0, 0, 0, 0.3);
}

/* Windows 绿点缀 - 用于成功提示、确认按钮等 */
.arco-alert[class*='success'],
[class*='alert'][class*='success'],
.arco-message-success,
[class*='message'][class*='success'] {
  background-color: rgba(0, 163, 0, 0.1);
  border: 1px solid #00a300;
  border-left: 3px solid #00a300;
}

/* 链接 hover 时可以使用 Windows 绿 */
a:not([class*='button']):not([class*='btn'])[class*='success'],
a:not([class*='button']):not([class*='btn'])[class*='confirm'] {
  color: #00a300;
}

a:not([class*='button']):not([class*='btn'])[class*='success']:hover,
a:not([class*='button']):not([class*='btn'])[class*='confirm']:hover {
  color: #00c300;
  text-decoration: underline;
}

/* Windows 绿点缀 - 复选框选中状态 */
.arco-checkbox-checked .arco-checkbox-icon,
input[type='checkbox']:checked {
  background-color: #00a300;
  border-color: #00a300;
}

.arco-checkbox-checked .arco-checkbox-icon::after {
  border-color: white;
}

/* Windows 绿点缀 - 单选框选中状态 */
.arco-radio-checked .arco-radio-button,
input[type='radio']:checked {
  border-color: #00a300;
}

.arco-radio-checked .arco-radio-button::after {
  background-color: #00a300;
}

/* Windows 绿点缀 - 进度条成功状态 */
.arco-progress-line[class*='success'],
.arco-progress-line[data-status='success'] {
  background-color: rgba(0, 163, 0, 0.1);
}

.arco-progress-line[class*='success'] .arco-progress-line-inner,
.arco-progress-line[data-status='success'] .arco-progress-line-inner {
  background-color: #00a300;
}

/* Windows 绿点缀 - 标签成功状态 */
.arco-tag[class*='success'],
.arco-tag[data-color='green'] {
  background-color: rgba(0, 163, 0, 0.1);
  border-color: #00a300;
  color: #00a300;
}

/* 明确排除模型选择按钮及其所有子元素，保持系统默认样式 */
.sendbox-model-btn,
[class*='sendbox-model'],
.sendbox-model-btn *,
[class*='sendbox-model'] * {
  /* 重置所有可能被影响的样式 */
  color: inherit;
  fill: inherit;
  background: inherit;
  border: inherit;
  border-radius: inherit;
  box-shadow: inherit;
  transform: none;
}

/* 排除发送框工具区域（包含模型选择器） */
.sendbox-tools,
[class*='sendbox-tools'],
.sendbox-tools *,
[class*='sendbox-tools'] * {
  color: inherit;
  fill: inherit;
  background: inherit;
  border: inherit;
  border-radius: inherit;
  box-shadow: inherit;
  transform: none;
}

/* ==================== 滚动条 Scrollbar ==================== */
/* 滚动条美化 - 经典 Windows 滚动条样式 */
::-webkit-scrollbar {
  width: 16px;
  height: 16px;
}

::-webkit-scrollbar-thumb {
  background: linear-gradient(180deg, #c0c0c0 0%, #808080 100%);
  border: 1px solid #808080;
  border-radius: 0; /* 方角 */
  box-shadow:
    inset 1px 1px 0 rgba(255, 255, 255, 0.5),
    inset -1px -1px 0 rgba(0, 0, 0, 0.2);
  transition: background 0.2s ease;
}

::-webkit-scrollbar-thumb:hover {
  background: linear-gradient(180deg, #d0d0d0 0%, #909090 100%);
}

/* 当容器hover时，滚动条也显示 */
*:hover::-webkit-scrollbar-thumb {
  background: linear-gradient(180deg, #c0c0c0 0%, #808080 100%);
}

*:hover::-webkit-scrollbar-thumb:hover {
  background: linear-gradient(180deg, #d0d0d0 0%, #909090 100%);
}

::-webkit-scrollbar-track {
  background: #f0f0f0;
  border: 1px solid #808080;
  border-radius: 0; /* 方角 */
  box-shadow: inset 1px 1px 0 rgba(0, 0, 0, 0.1);
}

::-webkit-scrollbar-button {
  background: #c0c0c0;
  border: 1px solid #808080;
  box-shadow:
    inset 1px 1px 0 rgba(255, 255, 255, 0.5),
    inset -1px -1px 0 rgba(0, 0, 0, 0.2);
}

::-webkit-scrollbar-button:hover {
  background: #d0d0d0;
}

/* ==================== 其他元素 ==================== */
/* 选中文字 */
::selection {
  background-color: #0078d4;
  color: white;
}

/* 链接样式 */
a:not([class*='button']):not([class*='btn']) {
  color: #0078d4;
  transition: color 0.2s ease;
}

a:hover:not([class*='button']):not([class*='btn']) {
  color: #005a9e;
  text-decoration: underline;
}

/* 次要按钮图标颜色 - 排除模型选择器 */
.arco-btn-secondary:not(.sendbox-model-btn):not([class*='model']):not([class*='Model']) svg,
button[type='secondary']:not(.sendbox-model-btn):not([class*='model']):not([class*='Model']) svg {
  color: #0078d4;
  transition: color 0.2s ease;
}

.arco-btn-secondary:not(.sendbox-model-btn):not([class*='model']):not([class*='Model']) svg:hover,
button[type='secondary']:not(.sendbox-model-btn):not([class*='model']):not([class*='Model']) svg:hover {
  color: #1a86d9;
}

/* 消息区域图标颜色 - 只针对消息气泡内的图标 */
.message-item .message-content svg,
[class*='message'] [class*='content'] svg {
  color: #404040;
  transition: color 0.2s ease;
}

.message-item:hover .message-content svg,
[class*='message']:hover [class*='content'] svg {
  color: #0078d4;
}

/* ==================== Tooltip 和 Popover ==================== */
/* Tooltip 和 Popover 保持交互层级，配色由全局 overlay token 统一控制 */
.arco-tooltip-popup,
.arco-popover-popup {
  pointer-events: none; /* 避免遮挡鼠标事件 */
  z-index: 10000 !important; /* 确保 tooltip 在最上层 */
}

/* 侧边栏 tooltip - 使用系统默认配色 */
.layout-sider ~ .arco-tooltip-popup,
.layout-sider .arco-tooltip-popup {
  z-index: 10001 !important;
}

/* ==================== 对话框 Modal ==================== */
/* 对话框背景和透明度 */
.arco-modal-body {
  background-color: rgba(240, 240, 240, 0.98);
  backdrop-filter: blur(4px);
  border: 2px outset #c0c0c0;
  color: var(--color-text-1); /* 确保文字清晰 */
}

.arco-modal-header {
  background: linear-gradient(180deg, #0078d4 0%, #005a9e 100%);
  color: white;
  border-bottom: 1px solid #005a9e;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.3);
}

.arco-modal-footer {
  background-color: rgba(240, 240, 240, 0.98);
  border-top: 1px solid #c0c0c0;
  color: var(--color-text-1); /* 确保文字清晰 */
}

/* ==================== Windows 经典元素（适度添加） ==================== */
/* 经典输入框 - Windows 95/98 风格 */
.arco-input,
input[type='text'],
input[type='password'],
input[type='email'],
input[type='number'],
input[type='search'] {
  background-color: var(--bg-2);
  border: 2px inset var(--border-base);
  box-shadow: inset 1px 1px 2px rgba(0, 0, 0, 0.2);
  border-radius: 0;
  padding: 4px 6px;
  font-size: 13px;
  color: var(--color-text-1); /* 确保文字清晰 */
  transition: all 0.1s ease;
}

.arco-input:focus,
input:focus {
  border: 2px inset var(--color-primary);
  box-shadow:
    inset 1px 1px 2px rgba(0, 0, 0, 0.3),
    0 0 0 1px var(--color-primary);
  outline: none;
}

/* 经典复选框和单选框 - Windows 95/98 风格 */
.arco-checkbox,
.arco-radio,
input[type='checkbox'],
input[type='radio'] {
  width: 13px;
  height: 13px;
  border: 2px inset var(--border-base);
  background-color: var(--bg-2);
  box-shadow: inset 1px 1px 2px rgba(0, 0, 0, 0.2);
  border-radius: 0;
  appearance: none;
  -webkit-appearance: none;
  -moz-appearance: none;
  transition: all 0.1s ease;
}

.arco-checkbox:checked,
.arco-radio:checked,
input[type='checkbox']:checked,
input[type='radio']:checked {
  background-color: var(--bg-active);
  border: 2px inset var(--border-base);
  box-shadow: inset 1px 1px 2px rgba(0, 0, 0, 0.3);
}

.arco-checkbox:checked::after,
input[type='checkbox']:checked::after {
  content: '✓';
  display: block;
  color: var(--text-primary);
  font-size: 10px;
  font-weight: bold;
  text-align: center;
  line-height: 9px;
}

.arco-radio {
  border-radius: 50%;
}

.arco-radio:checked::after,
input[type='radio']:checked::after {
  content: '';
  display: block;
  width: 5px;
  height: 5px;
  background-color: var(--text-primary);
  border-radius: 50%;
  margin: 2px auto;
}

/* ==================== 表单标签样式 - 保持简洁 ==================== */
/* 表单标签 - 移除不必要的背景色和边框，只作为标题显示 */
.arco-form-label-item,
[class*='form-label'],
[class*='arco-form-label'],
.arco-col[class*='form-label'],
.arco-form-item-label,
[class*='arco-form-item-label'] {
  background-color: transparent !important;
  border: none !important;
  box-shadow: none !important;
  padding: 0 !important;
  margin: 0 !important;
  color: var(--color-text-1) !important;
}

/* 表单标签文字 - 确保清晰可见 */
.arco-form-label-item *,
[class*='form-label'] *,
[class*='arco-form-label'] *,
.arco-col[class*='form-label'] *,
.arco-form-item-label *,
[class*='arco-form-item-label'] * {
  color: var(--color-text-1) !important;
  background-color: transparent !important;
}

/* 深色模式表单标签 */
[data-theme='dark'] .arco-form-label-item,
[data-theme='dark'] [class*='form-label'],
[data-theme='dark'] [class*='arco-form-label'],
[data-theme='dark'] .arco-col[class*='form-label'],
[data-theme='dark'] .arco-form-item-label,
[data-theme='dark'] [class*='arco-form-item-label'] {
  background-color: transparent !important;
  border: none !important;
  box-shadow: none !important;
  color: var(--color-text-1) !important;
}

[data-theme='dark'] .arco-form-label-item *,
[data-theme='dark'] [class*='form-label'] *,
[data-theme='dark'] [class*='arco-form-label'] *,
[data-theme='dark'] .arco-col[class*='form-label'] *,
[data-theme='dark'] .arco-form-item-label *,
[data-theme='dark'] [class*='arco-form-item-label'] * {
  color: var(--color-text-1) !important;
  background-color: transparent !important;
}

/* ==================== 深色模式 (Dark Mode) ==================== */
[data-theme='dark'] {
  /* 主色调 - 深色模式调亮以提高可见度 */
  --color-primary: #4da6f0;
  --primary: #4da6f0;
  --color-primary-light-1: #66b1e1;
  --color-primary-light-2: #80bce8;
  --color-primary-light-3: #99cbeb;
  --color-primary-dark-1: #3399e6;
  --primary-rgb: 77, 166, 240;

  /* 品牌色 - 深色模式 */
  --brand: #4da6f0;
  --brand-light: #1a2a3a;
  --brand-hover: #66b1e1;
  --color-brand-fill: #4da6f0;
  --color-brand-bg: #1a2a3a;

  /* AOU 品牌色板 - 深色模式反转 */
  --aou-1: #001b2c;
  --aou-2: #003052;
  --aou-3: #004578;
  --aou-4: #005a9e;
  --aou-5: #0078d4;
  --aou-6: #4da6f0;
  --aou-7: #66b1e1;
  --aou-8: #80bce8;
  --aou-9: #99cbeb;
  --aou-10: #b3d8f0;

  /* 背景色 - 深色 Windows 风格 */
  --color-bg-1: #1a1a1a;
  --bg-1: #1a1a1a;
  --color-bg-2: #262626;
  --bg-2: #262626;
  --color-bg-3: #333333;
  --bg-3: #333333;
  --color-bg-4: #404040;
  --bg-4: #404040;
  --bg-base: #0d0d0d;
  --bg-hover: #2d2d2d;
  --bg-active: #404040;
  --fill: #1a1a1a;
  --color-fill: #1a1a1a;

  /* 文字色 - 高对比度 */
  --color-text-1: #e0e0e0;
  --text-primary: #e0e0e0;
  --color-text-2: #b0b0b0;
  --text-secondary: #b0b0b0;
  --color-text-3: #808080;
  --text-disabled: #808080;
  --text-0: #ffffff;

  /* 边框色 */
  --color-border: #5a5a5a;
  --color-border-1: #5a5a5a;
  --color-border-2: #404040;
  --border-base: #5a5a5a;
  --border-light: #404040;

  /* 语义色 - 深色模式调整 */
  --success: #4caf50; /* Windows 绿（深色模式调亮） */
  --warning: #ff9800;
  --danger: #f44336;
  --info: #4da6f0;

  /* Windows 经典绿色 - 深色模式 */
  --windows-green: #4caf50;
  --windows-green-light: #66bb6a;
  --windows-green-dark: #388e3c;

  /* 消息和组件色 */
  --message-user-bg: #1a2a3a;
  --message-tips-bg: #1a1a1a;
  --workspace-btn-bg: #2d2d2d;

  /* 对话框颜色 */
  --dialog-fill-0: rgba(26, 26, 26, 0.95);
}

/* ===== 深色模式全局样式 ===== */
[data-theme='dark'] body,
[data-theme='dark'] html {
  background-color: var(--bg-1);
  color: var(--text-primary);
}

[data-theme='dark'] .arco-layout,
[data-theme='dark'] [class*='layout'] {
  background-color: var(--bg-1);
}

[data-theme='dark'] .arco-layout-content {
  background-color: var(--bg-1);
}

/* ===== 深色模式侧边栏 ===== */
[data-theme='dark'] .layout-sider {
  background-color: var(--bg-3);
  border-right: 2px solid var(--border-base);
}

[data-theme='dark'] .layout-sider.collapsed {
  overflow: hidden !important;
}

[data-theme='dark'] .layout-sider.collapsed * {
  overflow: hidden !important;
}

[data-theme='dark'] .layout-sider.collapsed::-webkit-scrollbar {
  display: none !important;
  width: 0 !important;
  height: 0 !important;
}

[data-theme='dark'] .layout-sider-header {
  background: linear-gradient(180deg, #005a9e 0%, #004578 100%);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.1),
    0 1px 2px rgba(0, 0, 0, 0.4);
}

/* ===== 深色模式背景图 ===== */
/* 参考 Hello Kitty 模式：只在 ::before 上设置半透明遮罩 */
[data-theme='dark'] .layout-content.bg-1::before {
  background: linear-gradient(135deg, rgba(26, 42, 58, 0.7) 0%, rgba(30, 46, 62, 0.75) 50%, rgba(26, 42, 58, 0.7) 100%);
}

[data-theme='dark'] [class*='chat-layout'] .arco-layout-content::before,
[data-theme='dark'] [class*='conversation'] .arco-layout-content::before {
  opacity: 0.2;
  filter: brightness(0.9) saturate(1.1);
}

/* ===== 深色模式输入框 ===== */
[data-theme='dark'] .guidInputCard textarea,
[data-theme='dark'] [class*='guidInputCard'] textarea {
  background-color: rgba(38, 38, 38, 0.98);
  color: var(--color-text-1);
}

[data-theme='dark'] .sendbox-container:not([class*='model']):not([class*='Model']),
[data-theme='dark']
  [class*='sendbox']:not([class*='input']):not([class*='textarea']):not([class*='model']):not([class*='Model']):not(
    [class*='tools']
  ) {
  background-color: rgba(38, 38, 38, 0.95);
  border: 2px outset var(--border-base);
  box-shadow:
    inset 1px 1px 0 rgba(255, 255, 255, 0.1),
    inset -1px -1px 0 rgba(0, 0, 0, 0.3),
    0 2px 4px rgba(0, 0, 0, 0.3);
}

[data-theme='dark'] .sendbox-container textarea,
[data-theme='dark'] [class*='sendbox'] textarea {
  color: var(--text-primary);
}

[data-theme='dark'] .guidInputCard {
  background-color: rgba(38, 38, 38, 0.95);
  border: 2px outset var(--border-base);
  box-shadow:
    inset 1px 1px 0 rgba(255, 255, 255, 0.1),
    inset -1px -1px 0 rgba(0, 0, 0, 0.3),
    0 2px 4px rgba(0, 0, 0, 0.3);
}

/* ===== 深色模式消息气泡 ===== */
[data-theme='dark'] .message-item.user .message-bubble,
[data-theme='dark'] [class*='message'][class*='user'] .message-content {
  background: linear-gradient(180deg, #005a9e 0%, #004578 100%);
  color: var(--text-white);
  border: 1px solid #004578;
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.2),
    0 2px 4px rgba(0, 0, 0, 0.4);
}

[data-theme='dark'] .message-item.ai .message-bubble,
[data-theme='dark'] .message-item.assistant .message-bubble,
[data-theme='dark'] [class*='message'][class*='ai'] .message-content,
[data-theme='dark'] [class*='message'][class*='assistant'] .message-content {
  background: rgba(38, 38, 38, 0.98);
  border: 1px solid var(--border-base);
  box-shadow:
    inset 1px 1px 0 rgba(255, 255, 255, 0.1),
    inset -1px -1px 0 rgba(0, 0, 0, 0.3),
    0 2px 4px rgba(0, 0, 0, 0.3);
  color: var(--text-primary);
}

[data-theme='dark'] .message-item.ai .arco-alert,
[data-theme='dark'] [class*='message'][class*='ai'] .arco-alert,
[data-theme='dark'] .message-item.ai [class*='alert'],
[data-theme='dark'] [class*='message'][class*='ai'] [class*='alert'] {
  background-color: rgba(38, 38, 38, 0.9);
  border-color: var(--border-base);
  color: var(--text-primary);
}

[data-theme='dark'] .message-item.ai .arco-card,
[data-theme='dark'] [class*='message'][class*='ai'] .arco-card,
[data-theme='dark'] .message-item.ai [class*='card'],
[data-theme='dark'] [class*='message'][class*='ai'] [class*='card'] {
  background-color: rgba(38, 38, 38, 0.9);
  border-color: var(--border-base);
  color: var(--text-primary);
}

/* ===== 深色模式按钮 ===== */
[data-theme='dark']
  .arco-btn-primary:not([class*='icon']):not([class*='circle']):not([class*='model']):not([class*='Model']),
[data-theme='dark']
  button[type='primary']:not([class*='icon']):not([class*='circle']):not([class*='model']):not([class*='Model']) {
  background: linear-gradient(180deg, #005a9e 0%, #004578 100%);
  border: 2px outset #005a9e;
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.2),
    0 2px 4px rgba(0, 0, 0, 0.4);
}

[data-theme='dark']
  .arco-btn-primary:hover:not([class*='icon']):not([class*='circle']):not([class*='model']):not([class*='Model']),
[data-theme='dark']
  button[type='primary']:hover:not([class*='icon']):not([class*='circle']):not([class*='model']):not([class*='Model']) {
  background: linear-gradient(180deg, #0078d4 0%, #005a9e 100%);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.3),
    0 3px 6px rgba(0, 0, 0, 0.5);
}

[data-theme='dark'] .arco-btn-success,
[data-theme='dark'] button[type='success'] {
  background: linear-gradient(180deg, #388e3c 0%, #2e7d32 100%);
  border: 2px outset #388e3c;
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.2),
    0 2px 4px rgba(0, 0, 0, 0.4);
}

[data-theme='dark'] .arco-btn-success:hover,
[data-theme='dark'] button[type='success']:hover {
  background: linear-gradient(180deg, #4caf50 0%, #388e3c 100%);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.3),
    0 3px 6px rgba(0, 0, 0, 0.5);
}

/* 深色模式 Windows 绿点缀 */
[data-theme='dark'] .arco-alert[class*='success'],
[data-theme='dark'] [class*='alert'][class*='success'],
[data-theme='dark'] .arco-message-success,
[data-theme='dark'] [class*='message'][class*='success'] {
  background-color: rgba(76, 175, 80, 0.15);
  border: 1px solid #4caf50;
  border-left: 3px solid #4caf50;
}

[data-theme='dark'] a:not([class*='button']):not([class*='btn'])[class*='success'],
[data-theme='dark'] a:not([class*='button']):not([class*='btn'])[class*='confirm'] {
  color: #4caf50;
}

[data-theme='dark'] a:not([class*='button']):not([class*='btn'])[class*='success']:hover,
[data-theme='dark'] a:not([class*='button']):not([class*='btn'])[class*='confirm']:hover {
  color: #66bb6a;
  text-decoration: underline;
}

/* 深色模式 Windows 绿点缀 - 复选框选中状态 */
[data-theme='dark'] .arco-checkbox-checked .arco-checkbox-icon,
[data-theme='dark'] input[type='checkbox']:checked {
  background-color: #4caf50;
  border-color: #4caf50;
}

/* 深色模式 Windows 绿点缀 - 单选框选中状态 */
[data-theme='dark'] .arco-radio-checked .arco-radio-button,
[data-theme='dark'] input[type='radio']:checked {
  border-color: #4caf50;
}

[data-theme='dark'] .arco-radio-checked .arco-radio-button::after {
  background-color: #4caf50;
}

/* 深色模式 Windows 绿点缀 - 进度条成功状态 */
[data-theme='dark'] .arco-progress-line[class*='success'],
[data-theme='dark'] .arco-progress-line[data-status='success'] {
  background-color: rgba(76, 175, 80, 0.15);
}

[data-theme='dark'] .arco-progress-line[class*='success'] .arco-progress-line-inner,
[data-theme='dark'] .arco-progress-line[data-status='success'] .arco-progress-line-inner {
  background-color: #4caf50;
}

/* 深色模式 Windows 绿点缀 - 标签成功状态 */
[data-theme='dark'] .arco-tag[class*='success'],
[data-theme='dark'] .arco-tag[data-color='green'] {
  background-color: rgba(76, 175, 80, 0.15);
  border-color: #4caf50;
  color: #4caf50;
}

/* ===== 深色模式滚动条 ===== */
[data-theme='dark'] ::-webkit-scrollbar-thumb {
  background: linear-gradient(180deg, #5a5a5a 0%, #404040 100%);
  border: 1px solid #5a5a5a;
  box-shadow:
    inset 1px 1px 0 rgba(255, 255, 255, 0.2),
    inset -1px -1px 0 rgba(0, 0, 0, 0.4);
}

[data-theme='dark'] ::-webkit-scrollbar-thumb:hover {
  background: linear-gradient(180deg, #6a6a6a 0%, #4d4d4d 100%);
}

[data-theme='dark'] ::-webkit-scrollbar-track {
  background: var(--bg-1);
  border: 1px solid var(--border-base);
  box-shadow: inset 1px 1px 0 rgba(0, 0, 0, 0.3);
}

[data-theme='dark'] ::-webkit-scrollbar-button {
  background: var(--bg-3);
  border: 1px solid var(--border-base);
  box-shadow:
    inset 1px 1px 0 rgba(255, 255, 255, 0.1),
    inset -1px -1px 0 rgba(0, 0, 0, 0.4);
}

/* ===== 深色模式其他元素 ===== */
[data-theme='dark'] ::selection {
  background-color: var(--color-primary);
  color: var(--text-white);
}

[data-theme='dark'] a:not([class*='button']):not([class*='btn']) {
  color: var(--color-primary);
}

[data-theme='dark'] a:hover:not([class*='button']):not([class*='btn']) {
  color: var(--color-primary-light-1);
}

[data-theme='dark'] .arco-tooltip-popup,
[data-theme='dark'] .arco-popover-popup {
  z-index: 10000 !important; /* 确保 tooltip 在最上层 */
}

/* 深色模式侧边栏 tooltip - 使用系统默认配色 */
[data-theme='dark'] .layout-sider ~ .arco-tooltip-popup,
[data-theme='dark'] .layout-sider .arco-tooltip-popup {
  z-index: 10001 !important;
}

[data-theme='dark'] .arco-modal-body {
  background: var(--bg-2);
  backdrop-filter: blur(8px);
  border: 2px outset var(--border-base);
  color: var(--text-primary);
}

[data-theme='dark'] .arco-modal-header {
  background: linear-gradient(180deg, #005a9e 0%, #004578 100%);
  color: var(--text-white);
  border-bottom: 1px solid rgba(0, 0, 0, 0.3);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.1);
}

[data-theme='dark'] .arco-modal-footer {
  background: var(--bg-2);
  border-top: 1px solid var(--border-base);
  color: var(--text-primary);
}

/* ===== 深色模式输入框 ===== */
[data-theme='dark'] .arco-input,
[data-theme='dark'] input[type='text'],
[data-theme='dark'] input[type='password'],
[data-theme='dark'] input[type='email'],
[data-theme='dark'] input[type='number'],
[data-theme='dark'] input[type='search'] {
  background-color: var(--bg-2);
  border: 2px inset var(--border-base);
  box-shadow: inset 1px 1px 2px rgba(0, 0, 0, 0.4);
  color: var(--text-primary);
}

[data-theme='dark'] .arco-input:focus,
[data-theme='dark'] input:focus {
  border: 2px inset var(--color-primary);
  box-shadow:
    inset 1px 1px 2px rgba(0, 0, 0, 0.5),
    0 0 0 1px var(--color-primary);
}

/* ===== 深色模式复选框和单选框 ===== */
[data-theme='dark'] .arco-checkbox,
[data-theme='dark'] .arco-radio,
[data-theme='dark'] input[type='checkbox'],
[data-theme='dark'] input[type='radio'] {
  background-color: var(--bg-2);
  border: 2px inset var(--border-base);
  box-shadow: inset 1px 1px 2px rgba(0, 0, 0, 0.4);
}

[data-theme='dark'] .arco-checkbox:checked,
[data-theme='dark'] .arco-radio:checked,
[data-theme='dark'] input[type='checkbox']:checked,
[data-theme='dark'] input[type='radio']:checked {
  background-color: var(--bg-active);
  border: 2px inset var(--border-base);
  box-shadow: inset 1px 1px 2px rgba(0, 0, 0, 0.5);
}

/* ==================== 响应式调整 ==================== */
@media (max-width: 768px) {
  .guidInputCard,
  .sendbox-container {
    border-radius: 4px;
  }

  .message-item.user .message-bubble,
  .message-item.ai .message-bubble,
  .message-item.assistant .message-bubble {
    border-radius: 4px;
    padding: 10px 14px;
  }

  .arco-btn-primary,
  .arco-btn-secondary {
    border-radius: 4px;
    padding: 8px 16px;
  }
}

/* ==================== 打印样式 ==================== */
@media print {
  .layout-content.bg-1::before,
  [class*='chat-layout'] .arco-layout-content::before {
    display: none;
  }

  * {
    box-shadow: none !important;
    text-shadow: none !important;
  }
}
`,ce=`/* ================================================
   Retroma Y2K JP — v4.2
   配色: 薰衣草紫 × 鼠尾草绿 × 浅粉 × 米黄
   ================================================ */

:root {
  --color-primary: #5a8a88;
  --primary: #5a8a88;
  --color-primary-light-1: #7aacaa;
  --color-primary-light-2: #9ecece;
  --color-primary-light-3: #c0e4e4;
  --color-primary-dark-1: #3a6a68;
  --primary-rgb: 90, 138, 136;

  --brand: #7a5898;
  --brand-light: #f2ecfa;
  --brand-hover: #9878b8;
  --color-brand-fill: #7a5898;
  --color-brand-bg: #f2ecfa;

  --aou-1: #f4f0fa;
  --aou-2: #e8e0f4;
  --aou-3: #d4c8ec;
  --aou-4: #b8a8dc;
  --aou-5: #9a84c8;
  --aou-6: #7a5898;
  --aou-7: #5c3c78;
  --aou-8: #402258;
  --aou-9: #260c38;
  --aou-10: #0e0018;

  --bg-base: #fefbf2;
  --bg-1: #f8f3e6;
  --bg-2: #f0e9d8;
  --bg-3: #e2d8c4;
  --bg-4: #c8bcaa;
  --bg-5: #aca090;
  --bg-6: #887870;
  --bg-8: #4e4038;
  --bg-9: #282018;
  --bg-10: #100c08;
  --color-bg-1: #f8f3e6;
  --color-bg-2: #f0e9d8;
  --color-bg-3: #e2d8c4;
  --color-bg-4: #c8bcaa;

  --bg-hover: #ede0f8;
  --bg-active: #ddd0f0;

  --text-primary: #2c1e38;
  --text-secondary: #6a5880;
  --text-disabled: #c0b0d0;
  --text-0: #1a0e28;
  --text-white: #fef8ff;
  --color-text-1: #2c1e38;
  --color-text-2: #6a5880;
  --color-text-3: #9878b0;
  --color-text-4: #c4b4d8;

  --border-base: #b8a8cc;
  --border-light: #d0c4e0;
  --border-special: #a898bc;
  --color-border: #b8a8cc;
  --color-border-1: #b8a8cc;
  --color-border-2: #d0c4e0;

  --fill: #f8f3e6;
  --color-fill: #f8f3e6;
  --fill-0: #fefbf2;
  --fill-white-to-black: #fefbf2;
  --dialog-fill-0: #fefbf2;
  --inverse: #2c1e38;

  --success: #4a9a88;
  --warning: #b08820;
  --danger: #c03868;
  --info: #5a8a88;

  --message-user-bg: #ead8f8;
  --message-tips-bg: #f4ede0;
  --workspace-btn-bg: #ede0f4;
  --color-guid-agent-bar: #ece2f8;
  --retroma-accent-gradient: linear-gradient(
    135deg,
    rgba(236, 225, 247, 0.96) 0%,
    rgba(242, 235, 250, 0.94) 56%,
    rgba(239, 248, 238, 0.9) 100%
  );
  --retroma-accent-gradient-hover: linear-gradient(
    135deg,
    rgba(231, 216, 245, 0.96) 0%,
    rgba(236, 228, 247, 0.94) 56%,
    rgba(229, 240, 230, 0.9) 100%
  );
  --retroma-accent-vertical: linear-gradient(180deg, #9b78b8 0%, #6a9b92 100%);
}

[data-theme='dark'] {
  --color-primary: #7ab8b4;
  --primary: #7ab8b4;
  --color-primary-light-1: #98d0cc;
  --color-primary-light-2: #b4e4e0;
  --color-primary-light-3: #cef4f2;
  --color-primary-dark-1: #58908c;
  --primary-rgb: 122, 184, 180;

  --brand: #c090e0;
  --brand-light: #38204e;
  --brand-hover: #a870c8;
  --color-brand-fill: #c090e0;
  --color-brand-bg: #38204e;

  --aou-1: #1c1028;
  --aou-2: #2c1c3e;
  --aou-3: #402854;
  --aou-4: #58386c;
  --aou-5: #745088;
  --aou-6: #9870a8;
  --aou-7: #b890c4;
  --aou-8: #d0b4dc;
  --aou-9: #e4d4ec;
  --aou-10: #f4ecf8;

  --bg-base: #1e201f;
  --bg-1: #272b29;
  --bg-2: #313734;
  --bg-3: #3e4743;
  --bg-4: #4f5a54;
  --bg-5: #65716a;
  --bg-6: #7c8880;
  --bg-8: #9aa7bf;
  --bg-9: #c2cee1;
  --bg-10: #e7edf7;
  --color-bg-1: #272b29;
  --color-bg-2: #313734;
  --color-bg-3: #3e4743;
  --color-bg-4: #4f5a54;

  --bg-hover: #3f4843;
  --bg-active: #4a5550;

  --text-primary: #eceadf;
  --text-secondary: #c8c7bb;
  --text-disabled: #8f9288;
  --text-0: #f5f2e8;
  --text-white: #f5f2e8;
  --color-text-1: #eceadf;
  --color-text-2: #c8c7bb;
  --color-text-3: #a7aa9f;
  --color-text-4: #7d8279;

  --border-base: #6f7870;
  --border-light: #595f5b;
  --border-special: #848c84;
  --color-border: #6f7870;
  --color-border-1: #6f7870;
  --color-border-2: #595f5b;

  --fill: #343a37;
  --color-fill: #343a37;
  --fill-0: rgba(236, 234, 223, 0.08);
  --fill-white-to-black: #1f2321;
  --dialog-fill-0: #373d3a;
  --inverse: #f2eee3;

  --success: #70c8b8;
  --warning: #caa56b;
  --danger: #e07098;
  --info: #7ab8b4;

  --message-user-bg: #4a335d;
  --message-tips-bg: #3b423f;
  --workspace-btn-bg: #404844;
  --color-guid-agent-bar: #3d4541;
  --retroma-accent-gradient: linear-gradient(
    135deg,
    rgba(93, 78, 109, 0.82) 0%,
    rgba(80, 91, 103, 0.8) 52%,
    rgba(73, 88, 81, 0.78) 100%
  );
  --retroma-accent-gradient-hover: linear-gradient(
    135deg,
    rgba(88, 74, 104, 0.84) 0%,
    rgba(76, 88, 99, 0.82) 52%,
    rgba(70, 85, 78, 0.8) 100%
  );
  --retroma-accent-vertical: linear-gradient(180deg, #c4b095 0%, #8db1a0 100%);
}

body {
  font-family: 'Meiryo', 'Yu Gothic UI', 'Hiragino Kaku Gothic ProN', 'Microsoft YaHei', 'PingFang SC', sans-serif;
  letter-spacing: 0.015em;
  background-color: var(--bg-base) !important;
}

body::before,
body::after {
  content: '';
  position: fixed;
  inset: -14%;
  pointer-events: none;
  z-index: 0;
  will-change: background-position, opacity, transform;
}

body::before {
  background-image:
    radial-gradient(circle at 8% 12%, rgba(172, 136, 208, 0.66) 0 1.2px, transparent 2.4px),
    radial-gradient(circle at 18% 82%, rgba(116, 178, 165, 0.62) 0 1.3px, transparent 2.5px),
    radial-gradient(circle at 36% 32%, rgba(202, 150, 198, 0.58) 0 1.1px, transparent 2.3px),
    radial-gradient(circle at 52% 14%, rgba(146, 180, 214, 0.58) 0 1.1px, transparent 2.2px),
    radial-gradient(circle at 64% 66%, rgba(152, 120, 184, 0.64) 0 1.3px, transparent 2.5px),
    radial-gradient(circle at 82% 22%, rgba(128, 182, 168, 0.6) 0 1.2px, transparent 2.4px),
    radial-gradient(circle at 90% 76%, rgba(178, 138, 210, 0.62) 0 1.2px, transparent 2.4px);
  background-size:
    340px 340px,
    420px 420px,
    520px 520px,
    620px 620px,
    460px 460px,
    560px 560px,
    700px 700px;
  background-position:
    0 0,
    0 0,
    0 0,
    0 0,
    0 0,
    0 0,
    0 0;
  animation: retroma-particle-drift 54s linear infinite;
  opacity: 0.9;
}

body::after {
  background-image:
    radial-gradient(circle at 12% 44%, rgba(236, 220, 250, 0.32) 0 2.1px, transparent 3.5px),
    radial-gradient(circle at 58% 74%, rgba(220, 245, 236, 0.26) 0 2px, transparent 3.3px),
    radial-gradient(circle at 84% 36%, rgba(242, 220, 238, 0.26) 0 2.2px, transparent 3.6px);
  background-size:
    780px 780px,
    940px 940px,
    1120px 1120px;
  animation:
    retroma-particle-drift-slow 88s linear infinite,
    retroma-particle-twinkle 6.4s ease-in-out infinite;
  opacity: 0.46;
}

.app-shell {
  position: relative;
  z-index: 1;
  background-color: transparent !important;
}

.layout-content.bg-1,
.layout-content {
  background-color: transparent !important;
}

.layout.arco-layout,
.arco-layout {
  background: transparent !important;
}

.settings-modal .bg-2.rd-16px,
.settings-modal .bg-2.rd-12px,
.settings-modal .arco-collapse-item.bg-2,
.settings-page-wrapper .bg-2.rd-16px,
.settings-page-wrapper .bg-2.rd-12px,
.settings-page-wrapper .arco-collapse-item.bg-2 {
  background: linear-gradient(180deg, #e6ede7 0%, #dde6df 100%) !important;
  border: 1px solid rgba(144, 170, 153, 0.42) !important;
}

[data-theme='dark'] .settings-modal .bg-2.rd-16px,
[data-theme='dark'] .settings-modal .bg-2.rd-12px,
[data-theme='dark'] .settings-modal .arco-collapse-item.bg-2,
[data-theme='dark'] .settings-page-wrapper .bg-2.rd-16px,
[data-theme='dark'] .settings-page-wrapper .bg-2.rd-12px,
[data-theme='dark'] .settings-page-wrapper .arco-collapse-item.bg-2 {
  background: linear-gradient(
    180deg,
    rgba(64, 72, 68, 0.95) 0%,
    rgba(55, 63, 59, 0.95) 56%,
    rgba(47, 55, 51, 0.94) 100%
  ) !important;
  border: 1px solid rgba(116, 126, 120, 0.3) !important;
}

/* WebUI settings cards: lower luminance for better dark-mode readability */
[data-theme='dark'] .settings-page-wrapper .bg-2.rd-16px .bg-fill-1,
[data-theme='dark'] .settings-page-wrapper .bg-2.rd-12px .bg-fill-1,
[data-theme='dark'] .settings-modal .bg-2.rd-16px .bg-fill-1,
[data-theme='dark'] .settings-modal .bg-2.rd-12px .bg-fill-1 {
  background: rgba(68, 76, 72, 0.7) !important;
  border-color: rgba(112, 122, 116, 0.38) !important;
}

[data-theme='dark'] .settings-page-wrapper .bg-2.rd-16px .text-t-primary,
[data-theme='dark'] .settings-page-wrapper .bg-2.rd-12px .text-t-primary,
[data-theme='dark'] .settings-modal .bg-2.rd-16px .text-t-primary,
[data-theme='dark'] .settings-modal .bg-2.rd-12px .text-t-primary {
  color: #f1ede2 !important;
}

[data-theme='dark'] .settings-page-wrapper .bg-2.rd-16px .text-t-secondary,
[data-theme='dark'] .settings-page-wrapper .bg-2.rd-12px .text-t-secondary,
[data-theme='dark'] .settings-modal .bg-2.rd-16px .text-t-secondary,
[data-theme='dark'] .settings-modal .bg-2.rd-12px .text-t-secondary {
  color: #d1cec2 !important;
}

.app-titlebar {
  background: linear-gradient(180deg, #faf4e8 0%, #f2e8d8 100%) !important;
  border-bottom: 3px solid #9878b8 !important;
  color: #5a3878 !important;
}

[data-theme='dark'] .app-titlebar {
  background: linear-gradient(180deg, #434944 0%, #3b423d 44%, #343a36 100%) !important;
  border-bottom: 2px solid #9f8fb8 !important;
  color: #cfc5dc !important;
}

.app-titlebar__brand {
  color: #5a3878 !important;
  font-weight: 700 !important;
  text-shadow: 0 1px 0 rgba(255, 255, 255, 0.8) !important;
  letter-spacing: 0.05em !important;
}

[data-theme='dark'] .app-titlebar__brand {
  color: #cbc2d8 !important;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.4) !important;
}

.app-titlebar__button {
  color: #7a5898 !important;
  border-radius: 6px !important;
}

.app-titlebar__button:hover {
  background-color: rgba(152, 120, 184, 0.15) !important;
  color: #5a3878 !important;
}

[data-theme='dark'] .app-titlebar__button {
  color: #ad9dc0 !important;
}

[data-theme='dark'] .app-titlebar__button:hover {
  background-color: rgba(173, 157, 192, 0.16) !important;
  color: #cec4dd !important;
}

.layout-sider {
  background-color: var(--bg-1) !important;
  border-right: 1px solid #ccb8dc !important;
}

[data-theme='dark'] .layout-sider {
  background-color: var(--bg-1) !important;
  border-right: 1px solid #646d66 !important;
}

#root .layout-sider,
#root .arco-layout-sider.layout-sider {
  border-right: 1px solid #ccb8dc !important;
  box-shadow:
    1px 0 0 0 #b8a0cc,
    2px 0 0 0 #f0c8d8,
    3px 0 0 0 #8ab8a8 !important;
}

[data-theme='dark'] #root .layout-sider,
[data-theme='dark'] #root .arco-layout-sider.layout-sider {
  border-right: 1px solid #646d66 !important;
  box-shadow:
    1px 0 0 0 #59635c,
    2px 0 0 0 #61586b,
    3px 0 0 0 #547066 !important;
}

.layout-sider-header {
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.82) 0%, rgba(248, 243, 230, 0.74) 100%) !important;
  border-bottom: 1px dashed rgba(184, 168, 204, 0.55) !important;
}

[data-theme='dark'] .layout-sider-header {
  background: linear-gradient(
    180deg,
    rgba(67, 75, 70, 0.92) 0%,
    rgba(58, 66, 62, 0.91) 55%,
    rgba(50, 58, 54, 0.9) 100%
  ) !important;
  border-bottom: 1px dashed rgba(132, 142, 136, 0.28) !important;
}

[data-theme='dark'] body::before {
  background-image:
    radial-gradient(circle at 8% 12%, rgba(168, 132, 196, 0.54) 0 1.2px, transparent 2.4px),
    radial-gradient(circle at 18% 82%, rgba(109, 164, 152, 0.5) 0 1.3px, transparent 2.5px),
    radial-gradient(circle at 36% 32%, rgba(162, 126, 186, 0.48) 0 1.1px, transparent 2.3px),
    radial-gradient(circle at 52% 14%, rgba(96, 146, 168, 0.42) 0 1.1px, transparent 2.2px),
    radial-gradient(circle at 64% 66%, rgba(138, 108, 170, 0.52) 0 1.3px, transparent 2.5px),
    radial-gradient(circle at 82% 22%, rgba(106, 156, 144, 0.48) 0 1.2px, transparent 2.4px),
    radial-gradient(circle at 90% 76%, rgba(150, 118, 182, 0.5) 0 1.2px, transparent 2.4px);
  opacity: 0.84;
}

[data-theme='dark'] body::after {
  background-image:
    radial-gradient(circle at 12% 44%, rgba(210, 186, 232, 0.26) 0 2.1px, transparent 3.5px),
    radial-gradient(circle at 58% 74%, rgba(172, 210, 196, 0.22) 0 2px, transparent 3.3px),
    radial-gradient(circle at 84% 36%, rgba(190, 166, 216, 0.24) 0 2.2px, transparent 3.6px);
  opacity: 0.38;
}

.chat-history__item {
  border-radius: 10px !important;
  margin: 2px 6px !important;
  border: 1px solid transparent !important;
  transition:
    background-color 0.15s ease,
    border-color 0.15s ease,
    box-shadow 0.15s ease !important;
}

.chat-history__item:hover {
  background: rgba(255, 255, 255, 0.65) !important;
}

.settings-sider__item {
  border-radius: 10px !important;
  margin: 2px 6px !important;
  border: 1px solid transparent !important;
  transition:
    background-color 0.15s ease,
    border-color 0.15s ease,
    box-shadow 0.15s ease !important;
}

.settings-sider__item:hover {
  background: rgba(255, 255, 255, 0.58) !important;
}

.chat-history__item--active,
.chat-history__item[aria-selected='true'],
.chat-history__item[class~='!bg-active'],
.settings-sider__item[class~='!bg-aou-2'] {
  background: var(--retroma-accent-gradient) !important;
  border-color: #c9b3de !important;
  box-shadow:
    inset 0 0 0 1px rgba(255, 255, 255, 0.7),
    2px 2px 0 rgba(176, 156, 202, 0.6) !important;
}

.chat-history__item--active::before,
.chat-history__item[aria-selected='true']::before,
.chat-history__item[class~='!bg-active']::before,
.settings-sider__item[class~='!bg-aou-2']::before {
  content: '';
  position: absolute;
  left: 0;
  top: 7px;
  bottom: 7px;
  width: 3px;
  border-radius: 0 4px 4px 0;
  background: var(--retroma-accent-vertical);
}

[data-theme='dark'] .chat-history__item:hover {
  background: rgba(92, 102, 96, 0.4) !important;
}

[data-theme='dark'] .chat-history__item--active,
[data-theme='dark'] .chat-history__item[aria-selected='true'],
[data-theme='dark'] .chat-history__item[class~='!bg-active'],
[data-theme='dark'] .settings-sider__item[class~='!bg-aou-2'] {
  background: var(--retroma-accent-gradient) !important;
  border-color: #839086 !important;
  box-shadow:
    inset 0 0 0 1px rgba(232, 226, 214, 0.14),
    2px 2px 0 rgba(33, 39, 36, 0.68) !important;
}

[data-theme='dark'] .settings-sider__item:hover {
  background: rgba(87, 98, 92, 0.34) !important;
}

[data-theme='dark'] .chat-history__item--active::before,
[data-theme='dark'] .chat-history__item[aria-selected='true']::before,
[data-theme='dark'] .chat-history__item[class~='!bg-active']::before,
[data-theme='dark'] .settings-sider__item[class~='!bg-aou-2']::before {
  background: var(--retroma-accent-vertical) !important;
}

.chat-history--collapsed .chat-history__item,
.settings-sider--collapsed .settings-sider__item,
.layout-sider.collapsed .chat-history__item,
.layout-sider.arco-layout-sider-collapsed .chat-history__item,
.layout-sider.collapsed .settings-sider__item,
.layout-sider.arco-layout-sider-collapsed .settings-sider__item {
  margin: 6px auto !important;
  width: 38px !important;
  min-width: 38px !important;
  min-height: 38px !important;
  padding: 0 !important;
  padding-inline: 0 !important;
  padding-block: 0 !important;
  justify-content: center !important;
  align-items: center !important;
  gap: 0 !important;
}

.chat-history--collapsed,
.settings-sider--collapsed {
  scrollbar-width: thin !important;
}

.chat-history--collapsed::-webkit-scrollbar,
.settings-sider--collapsed::-webkit-scrollbar {
  width: 4px !important;
  height: 4px !important;
}

.layout-sider.collapsed .overflow-y-auto,
.layout-sider.arco-layout-sider-collapsed .overflow-y-auto {
  scrollbar-width: thin !important;
}

.layout-sider.collapsed .overflow-y-auto::-webkit-scrollbar,
.layout-sider.arco-layout-sider-collapsed .overflow-y-auto::-webkit-scrollbar,
.layout-sider.collapsed .arco-layout-sider-children::-webkit-scrollbar,
.layout-sider.arco-layout-sider-collapsed .arco-layout-sider-children::-webkit-scrollbar {
  width: 4px !important;
  height: 4px !important;
}

.chat-history--collapsed::-webkit-scrollbar-thumb,
.settings-sider--collapsed::-webkit-scrollbar-thumb,
.layout-sider.collapsed .overflow-y-auto::-webkit-scrollbar-thumb,
.layout-sider.arco-layout-sider-collapsed .overflow-y-auto::-webkit-scrollbar-thumb,
.layout-sider.collapsed .arco-layout-sider-children::-webkit-scrollbar-thumb,
.layout-sider.arco-layout-sider-collapsed .arco-layout-sider-children::-webkit-scrollbar-thumb {
  background: linear-gradient(180deg, #c4a8d8, #8a7aaa) !important;
  border-radius: 999px !important;
}

[data-theme='dark'] .chat-history--collapsed::-webkit-scrollbar-thumb,
[data-theme='dark'] .settings-sider--collapsed::-webkit-scrollbar-thumb,
[data-theme='dark'] .layout-sider.collapsed .overflow-y-auto::-webkit-scrollbar-thumb,
[data-theme='dark'] .layout-sider.arco-layout-sider-collapsed .overflow-y-auto::-webkit-scrollbar-thumb,
[data-theme='dark'] .layout-sider.collapsed .arco-layout-sider-children::-webkit-scrollbar-thumb,
[data-theme='dark'] .layout-sider.arco-layout-sider-collapsed .arco-layout-sider-children::-webkit-scrollbar-thumb {
  background: linear-gradient(180deg, #7a5c90, #4a2e68) !important;
}

.chat-history--collapsed .chat-history__item > :first-child,
.settings-sider--collapsed .settings-sider__item > :first-child,
.layout-sider.collapsed .chat-history__item > :first-child,
.layout-sider.arco-layout-sider-collapsed .chat-history__item > :first-child,
.layout-sider.collapsed .settings-sider__item > :first-child,
.layout-sider.arco-layout-sider-collapsed .settings-sider__item > :first-child {
  margin: 0 auto !important;
}

.layout-sider.collapsed .settings-sider .settings-sider__item,
.layout-sider.arco-layout-sider-collapsed .settings-sider .settings-sider__item {
  display: flex !important;
  align-items: center !important;
  justify-content: center !important;
  gap: 0 !important;
}

/* Settings sider collapsed: remove label layout footprint and force true centering */
.settings-sider--collapsed .settings-sider__item > :nth-child(2),
.layout-sider.collapsed .settings-sider .settings-sider__item > :nth-child(2),
.layout-sider.arco-layout-sider-collapsed .settings-sider .settings-sider__item > :nth-child(2) {
  display: none !important;
}

.settings-sider--collapsed .settings-sider__item,
.layout-sider.collapsed .settings-sider .settings-sider__item,
.layout-sider.arco-layout-sider-collapsed .settings-sider .settings-sider__item {
  padding: 0 !important;
  justify-content: center !important;
  align-items: center !important;
}

.settings-sider--collapsed .settings-sider__item::before,
.layout-sider.collapsed .settings-sider .settings-sider__item::before,
.layout-sider.arco-layout-sider-collapsed .settings-sider .settings-sider__item::before {
  content: none !important;
  display: none !important;
}

/* Collapsed sidebar: normalize utility padding/justify classes and center icon wrappers */
.layout-sider.collapsed .conversation-item,
.layout-sider.arco-layout-sider-collapsed .conversation-item,
.chat-history--collapsed .conversation-item,
.settings-sider--collapsed .conversation-item {
  padding: 0 !important;
  justify-content: center !important;
  align-items: center !important;
  gap: 0 !important;
}

.layout-sider.collapsed .settings-sider .settings-sider__item > svg,
.layout-sider.arco-layout-sider-collapsed .settings-sider .settings-sider__item > svg,
.layout-sider.collapsed .settings-sider .settings-sider__item > img,
.layout-sider.arco-layout-sider-collapsed .settings-sider .settings-sider__item > img,
.layout-sider.collapsed .settings-sider .settings-sider__item > .mt-2px,
.layout-sider.collapsed .settings-sider .settings-sider__item > [class*='w-20px'],
.layout-sider.arco-layout-sider-collapsed .settings-sider .settings-sider__item > .mt-2px,
.layout-sider.arco-layout-sider-collapsed .settings-sider .settings-sider__item > [class*='w-20px'],
.layout-sider.collapsed .chat-history__item > svg,
.layout-sider.arco-layout-sider-collapsed .chat-history__item > svg,
.layout-sider.collapsed .chat-history__item > img,
.layout-sider.arco-layout-sider-collapsed .chat-history__item > img,
.layout-sider.collapsed .chat-history__item > .mt-2px,
.layout-sider.arco-layout-sider-collapsed .chat-history__item > .mt-2px,
.layout-sider.collapsed .chat-history__item > [class*='w-24px'],
.layout-sider.arco-layout-sider-collapsed .chat-history__item > [class*='w-24px'] {
  margin: 0 auto !important;
  width: 24px !important;
  height: 24px !important;
  min-width: 24px !important;
  display: flex !important;
  align-items: center !important;
  justify-content: center !important;
  transform: none !important;
}

.layout-sider.collapsed .settings-sider .settings-sider__item > svg,
.layout-sider.arco-layout-sider-collapsed .settings-sider .settings-sider__item > svg,
.layout-sider.collapsed .settings-sider .settings-sider__item > .mt-2px svg,
.layout-sider.collapsed .settings-sider .settings-sider__item > [class*='w-20px'] svg,
.layout-sider.arco-layout-sider-collapsed .settings-sider .settings-sider__item > .mt-2px svg,
.layout-sider.arco-layout-sider-collapsed .settings-sider .settings-sider__item > [class*='w-20px'] svg,
.layout-sider.collapsed .chat-history__item > svg,
.layout-sider.arco-layout-sider-collapsed .chat-history__item > svg,
.layout-sider.collapsed .chat-history__item > .mt-2px svg,
.layout-sider.arco-layout-sider-collapsed .chat-history__item > .mt-2px svg,
.layout-sider.collapsed .chat-history__item > [class*='w-24px'] svg,
.layout-sider.arco-layout-sider-collapsed .chat-history__item > [class*='w-24px'] svg {
  display: block !important;
  margin: 0 auto !important;
  width: 20px !important;
  height: 20px !important;
  transform: none !important;
}

.chat-history--collapsed .chat-history__item--active::before,
.chat-history--collapsed .chat-history__item[aria-selected='true']::before,
.chat-history--collapsed .chat-history__item[class~='!bg-active']::before,
.settings-sider--collapsed .settings-sider__item[class~='!bg-aou-2']::before {
  top: 6px;
  bottom: 6px;
  width: 2px;
}

/* Collapsed selected state: use symmetric highlight to keep icon visually centered */
.chat-history--collapsed .chat-history__item--active,
.chat-history--collapsed .chat-history__item[aria-selected='true'],
.chat-history--collapsed .chat-history__item[class~='!bg-active'],
.settings-sider--collapsed .settings-sider__item[class~='!bg-aou-2'] {
  box-shadow: inset 0 0 0 1px rgba(176, 156, 202, 0.58) !important;
}

.chat-history--collapsed .chat-history__item--active::before,
.chat-history--collapsed .chat-history__item[aria-selected='true']::before,
.chat-history--collapsed .chat-history__item[class~='!bg-active']::before,
.settings-sider--collapsed .settings-sider__item[class~='!bg-aou-2']::before {
  display: none !important;
}

.layout-sider.collapsed .conversation-item::before,
.layout-sider.arco-layout-sider-collapsed .conversation-item::before,
.chat-history--collapsed .conversation-item::before,
.settings-sider--collapsed .conversation-item::before {
  content: none !important;
  display: none !important;
  width: 0 !important;
  background: transparent !important;
}

.layout-sider.collapsed .settings-sider [data-settings-id][class*='!bg-aou-2'],
.layout-sider.arco-layout-sider-collapsed .settings-sider [data-settings-id][class*='!bg-aou-2'] {
  box-shadow: inset 0 0 0 1px rgba(176, 156, 202, 0.58) !important;
  transform: none !important;
}

[data-theme='dark'] .layout-sider.collapsed .settings-sider [data-settings-id][class*='!bg-aou-2'],
[data-theme='dark'] .layout-sider.arco-layout-sider-collapsed .settings-sider [data-settings-id][class*='!bg-aou-2'] {
  box-shadow: inset 0 0 0 1px rgba(177, 136, 212, 0.44) !important;
  transform: none !important;
}

[data-theme='dark'] .chat-history--collapsed .chat-history__item--active,
[data-theme='dark'] .chat-history--collapsed .chat-history__item[aria-selected='true'],
[data-theme='dark'] .chat-history--collapsed .chat-history__item[class~='!bg-active'],
[data-theme='dark'] .settings-sider--collapsed .settings-sider__item[class~='!bg-aou-2'] {
  box-shadow: inset 0 0 0 1px rgba(177, 136, 212, 0.44) !important;
}

.arco-tree-node-title-wrapper {
  border-radius: 8px !important;
}

.arco-tree-node-selected > .arco-tree-node-title-wrapper {
  background: var(--brand-light) !important;
  color: var(--brand) !important;
}

.chat-layout-header {
  background: linear-gradient(180deg, #f2e8d8 0%, #ede4d8 100%) !important;
  border-bottom: none !important;
  box-shadow: 0 2px 8px rgba(152, 120, 184, 0.1) !important;
}

[data-theme='dark'] .chat-layout-header {
  background: linear-gradient(180deg, #454b47 0%, #3b423e 45%, #343a36 100%) !important;
  border-bottom: none !important;
  box-shadow: 0 2px 8px rgba(20, 24, 22, 0.3) !important;
}

.bg-dialog-fill-0 {
  background-color: var(--dialog-fill-0) !important;
}

.sendbox-panel {
  background: rgba(254, 251, 242, 0.95) !important;
  border: 2px solid #c0a8d8 !important;
  border-radius: 20px !important;
  box-shadow:
    4px 4px 0 #b8a0cc,
    0 4px 20px rgba(152, 120, 184, 0.12) !important;
  backdrop-filter: blur(6px);
}

[data-theme='dark'] .sendbox-panel {
  background: linear-gradient(
    145deg,
    rgba(88, 76, 102, 0.5) 0%,
    rgba(74, 84, 82, 0.9) 46%,
    rgba(57, 65, 61, 0.96) 100%
  ) !important;
  border: 2px solid #726d7e !important;
  box-shadow:
    4px 4px 0 #545e5a,
    0 4px 20px rgba(31, 38, 35, 0.28) !important;
}

[class*='sendbox'] textarea,
.sendbox-input--mobile {
  background: transparent !important;
  caret-color: #9878b8;
}

.guid-input-card-shell {
  background: linear-gradient(
    160deg,
    rgba(255, 255, 255, 0.99) 0%,
    rgba(255, 251, 245, 0.99) 58%,
    rgba(246, 236, 252, 0.97) 100%
  ) !important;
  border: 2px solid #d4b8ea !important;
  border-width: 2px !important;
  box-shadow: none !important;
}

.guidContainer .guidInputCard textarea,
[class*='guidContainer'] [class*='guidInputCard'] textarea {
  color: #2c1e38 !important;
  font-weight: 500 !important;
}

.guidContainer .guidInputCard textarea::placeholder,
[class*='guidContainer'] [class*='guidInputCard'] textarea::placeholder {
  color: #8e78a8 !important;
  opacity: 1 !important;
}

[data-theme='dark'] .guidContainer .guidInputCard,
[data-theme='dark'] [class*='guidContainer'] [class*='guidInputCard'] {
  background: linear-gradient(
    160deg,
    rgba(56, 46, 70, 0.96) 0%,
    rgba(47, 54, 62, 0.96) 54%,
    rgba(39, 46, 50, 0.96) 100%
  ) !important;
  border: 2px solid #746a86 !important;
  border-width: 2px !important;
  box-shadow: none !important;
}

[data-theme='dark'] .guidContainer .guidInputCard textarea,
[data-theme='dark'] [class*='guidContainer'] [class*='guidInputCard'] textarea {
  color: #f4ecff !important;
}

[data-theme='dark'] .guidContainer .guidInputCard textarea::placeholder,
[data-theme='dark'] [class*='guidContainer'] [class*='guidInputCard'] textarea::placeholder {
  color: #b8a0d0 !important;
}

/* guid page controls: keep Y2K style consistent in both light/dark */
.guidContainer .sendbox-model-btn.guid-config-btn {
  border: 1.5px solid #ccb8e0 !important;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.92) 0%, rgba(243, 233, 248, 0.9) 100%) !important;
  color: #6f4b90 !important;
  box-shadow: 2px 2px 0 #d5c2e8 !important;
}

.guidContainer .sendbox-model-btn.guid-config-btn:hover {
  border-color: #b08acc !important;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.96) 0%, rgba(236, 222, 246, 0.95) 100%) !important;
  box-shadow: 3px 3px 0 #c7aede !important;
}

[data-theme='dark'] .guidContainer .sendbox-model-btn.guid-config-btn {
  border: 1.5px solid #5f477f !important;
  background: linear-gradient(180deg, rgba(55, 40, 76, 0.92) 0%, rgba(44, 32, 62, 0.9) 100%) !important;
  color: #d5b7f0 !important;
  box-shadow: 2px 2px 0 #432f61 !important;
}

[data-theme='dark'] .guidContainer .sendbox-model-btn.guid-config-btn:hover {
  border-color: #8060aa !important;
  background: linear-gradient(180deg, rgba(63, 46, 88, 0.95) 0%, rgba(50, 36, 72, 0.95) 100%) !important;
  box-shadow: 3px 3px 0 #563d78 !important;
}

.guidContainer .actionTools .arco-btn.arco-btn-text {
  border-radius: 10px !important;
  background: rgba(246, 238, 251, 0.78) !important;
}

[data-theme='dark'] .guidContainer .actionTools .arco-btn.arco-btn-text {
  background: rgba(52, 38, 74, 0.7) !important;
}

.guidContainer [data-agent-pill='true'][data-agent-selected='true'] {
  box-shadow:
    inset 0 0 0 1px rgba(122, 88, 152, 0.28),
    0 2px 8px rgba(122, 88, 152, 0.18) !important;
}

[data-theme='dark'] .guidContainer [data-agent-pill='true'][data-agent-selected='true'] {
  box-shadow:
    inset 0 0 0 1px rgba(192, 144, 224, 0.3),
    0 2px 10px rgba(30, 20, 50, 0.45) !important;
}

.guidContainer .arco-menu-item.arco-menu-selected,
.guidContainer .arco-menu-light .arco-menu-selected {
  background: var(--retroma-accent-gradient) !important;
  color: var(--brand) !important;
}

[data-theme='dark'] .guidContainer .arco-menu-item.arco-menu-selected,
[data-theme='dark'] .guidContainer .arco-menu-dark .arco-menu-selected {
  background: var(--retroma-accent-gradient) !important;
  color: #e2c8f7 !important;
}

.send-button-custom,
.send-button-custom.arco-btn,
.send-button-custom.arco-btn-primary {
  background: linear-gradient(160deg, #a878c0 0%, #7a5898 56%, #6a9b92 100%) !important;
  border: 1.5px solid #6a4888 !important;
  border-radius: 10px !important;
  box-shadow:
    3px 3px 0 #4a2a68,
    5px 5px 12px rgba(90, 50, 120, 0.25) !important;
  color: #fff !important;
  transition: all 0.12s ease !important;
}

.send-button-custom:hover,
.send-button-custom.arco-btn:hover {
  background: linear-gradient(160deg, #b888d0 0%, #8a68a8 56%, #7baea5 100%) !important;
  box-shadow:
    4px 4px 0 #4a2a68,
    6px 6px 14px rgba(90, 50, 120, 0.3) !important;
  transform: translate(-1px, -1px) !important;
}

.send-button-custom:active,
.send-button-custom.arco-btn:active {
  transform: translate(2px, 2px) !important;
  box-shadow:
    1px 1px 0 #4a2a68,
    2px 2px 6px rgba(90, 50, 120, 0.2) !important;
}

[data-theme='dark'] .send-button-custom,
[data-theme='dark'] .send-button-custom.arco-btn {
  background: linear-gradient(160deg, #9868c0 0%, #6848a0 56%, #4f7f77 100%) !important;
  border-color: #7858a8 !important;
  box-shadow:
    3px 3px 0 #2c1050,
    5px 5px 12px rgba(80, 30, 120, 0.45) !important;
}

.message-item.user .message-bubble {
  background: linear-gradient(135deg, #ead8f8 0%, #d8c4f0 100%) !important;
  color: #2c1e38 !important;
  border-radius: 16px 16px 4px 16px !important;
  border: 1.5px solid #c4a8e0 !important;
  box-shadow: 3px 3px 0 rgba(180, 140, 220, 0.4) !important;
  padding: 10px 14px !important;
}

.message-item.ai .message-bubble,
.message-item.assistant .message-bubble {
  background: rgba(254, 251, 242, 0.97) !important;
  color: #2c1e38 !important;
  border-radius: 16px 16px 16px 4px !important;
  border: 1.5px solid #d0c4e0 !important;
  box-shadow: 3px 3px 0 rgba(184, 168, 204, 0.35) !important;
  padding: 10px 14px !important;
}

.message-item.ai .arco-alert,
.message-item.ai [class*='alert'] {
  background: rgba(244, 240, 250, 0.8) !important;
  border: 1px solid #d4c8ec !important;
  border-radius: 8px !important;
}

.message-item.ai .arco-card,
.message-item.ai [class*='card'] {
  background: rgba(248, 243, 230, 0.9) !important;
  border: 1px solid #d0c4e0 !important;
  border-radius: 8px !important;
}

[data-theme='dark'] .message-item.user .message-bubble {
  background: linear-gradient(135deg, #4a335d 0%, #362645 100%) !important;
  color: #f0e8fc !important;
  border: 1.5px solid #7f5f95 !important;
  box-shadow: 3px 3px 0 rgba(84, 60, 98, 0.55) !important;
}

[data-theme='dark'] .message-item.ai .message-bubble,
[data-theme='dark'] .message-item.assistant .message-bubble {
  background: linear-gradient(
    170deg,
    rgba(74, 83, 78, 0.97) 0%,
    rgba(62, 71, 67, 0.97) 58%,
    rgba(56, 64, 60, 0.97) 100%
  ) !important;
  color: #f2eee3 !important;
  border: 1.5px solid #7d897f !important;
  box-shadow: 3px 3px 0 rgba(50, 59, 54, 0.5) !important;
}

[data-theme='dark'] .message-item.ai .arco-alert,
[data-theme='dark'] .message-item.ai [class*='alert'] {
  background: rgba(40, 30, 56, 0.85) !important;
  border: 1px solid #504068 !important;
}

[data-theme='dark'] .message-item.ai .arco-card,
[data-theme='dark'] .message-item.ai [class*='card'] {
  background: rgba(36, 28, 52, 0.92) !important;
  border: 1px solid #504068 !important;
}

.rd-20px.text-14px.pb-40px.lh-20px {
  border: 1.5px solid var(--border-base) !important;
  border-left: 3px solid var(--brand) !important;
  border-radius: 12px !important;
}

.rd-20px.text-14px.pb-40px .arco-tag {
  background: var(--aou-2) !important;
  color: var(--brand) !important;
  border-color: var(--aou-3) !important;
}

[data-theme='dark'] .rd-20px.text-14px.pb-40px .arco-tag {
  background: var(--aou-3) !important;
  color: var(--aou-8) !important;
  border-color: var(--aou-4) !important;
}

.arco-btn-primary {
  background: linear-gradient(160deg, #a878c0 0%, #7a5898 56%, #6a9b92 100%) !important;
  border: 1.5px solid #6a4888 !important;
  border-radius: 10px !important;
  color: #fff !important;
  font-weight: 600 !important;
  box-shadow:
    3px 3px 0 #4a2a68,
    5px 5px 12px rgba(90, 50, 120, 0.25) !important;
  transition: all 0.12s ease !important;
  text-shadow: 0 1px 2px rgba(40, 0, 60, 0.3) !important;
}

.arco-btn-primary:hover {
  background: linear-gradient(160deg, #b888d0 0%, #8a68a8 56%, #7baea5 100%) !important;
  box-shadow:
    4px 4px 0 #4a2a68,
    6px 6px 14px rgba(90, 50, 120, 0.3) !important;
  transform: translate(-1px, -1px) !important;
}

.arco-btn-primary:active {
  transform: translate(2px, 2px) !important;
  box-shadow:
    1px 1px 0 #4a2a68,
    2px 2px 6px rgba(90, 50, 120, 0.2) !important;
}

[data-theme='dark'] .arco-btn-primary {
  background: linear-gradient(160deg, #9868c0 0%, #6848a0 56%, #4f7f77 100%) !important;
  border-color: #7858a8 !important;
  box-shadow:
    3px 3px 0 #2c1050,
    5px 5px 12px rgba(80, 30, 120, 0.45) !important;
}

.arco-btn-secondary,
.arco-btn-outline {
  border: 1.5px solid var(--border-base) !important;
  border-radius: 10px !important;
  background: linear-gradient(180deg, var(--bg-base) 0%, var(--bg-2) 100%) !important;
  color: var(--brand) !important;
  font-weight: 600 !important;
  box-shadow:
    3px 3px 0 var(--border-base),
    4px 4px 8px rgba(100, 70, 140, 0.1) !important;
  transition: all 0.12s ease !important;
}

.arco-btn-secondary:hover,
.arco-btn-outline:hover {
  border-color: var(--brand) !important;
  box-shadow:
    4px 4px 0 var(--aou-3),
    5px 5px 10px rgba(100, 70, 140, 0.15) !important;
  transform: translate(-1px, -1px) !important;
}

.arco-btn-secondary:active,
.arco-btn-outline:active {
  transform: translate(2px, 2px) !important;
  box-shadow: 1px 1px 0 var(--border-base) !important;
}

[data-theme='dark'] .arco-btn-secondary,
[data-theme='dark'] .arco-btn-outline {
  background: linear-gradient(180deg, var(--bg-2) 0%, var(--bg-3) 100%) !important;
  border-color: var(--border-base) !important;
  color: var(--brand) !important;
  box-shadow:
    3px 3px 0 var(--bg-4),
    4px 4px 8px rgba(0, 0, 0, 0.3) !important;
}

.sendbox-tools .arco-btn,
.sendbox-tools .arco-btn:hover,
.sendbox-tools .arco-btn:active {
  transform: none !important;
  box-shadow: none !important;
}

.settings-page-wrapper .arco-input-wrapper,
.settings-page-wrapper .arco-textarea-wrapper,
.settings-page-wrapper .arco-input-inner-wrapper,
.settings-modal .arco-input-wrapper,
.settings-modal .arco-textarea-wrapper,
.settings-modal .arco-input-inner-wrapper {
  border-radius: 10px !important;
  border: 1.5px solid var(--border-base) !important;
  background: var(--bg-base) !important;
  box-shadow: inset 0 2px 4px rgba(80, 40, 100, 0.06) !important;
  transition:
    border-color 0.15s ease,
    box-shadow 0.15s ease !important;
}

.settings-page-wrapper .arco-input-wrapper:focus-within,
.settings-page-wrapper .arco-textarea-wrapper:focus-within,
.settings-page-wrapper .arco-input-inner-wrapper:focus-within,
.settings-modal .arco-input-wrapper:focus-within,
.settings-modal .arco-textarea-wrapper:focus-within,
.settings-modal .arco-input-inner-wrapper:focus-within {
  border-color: var(--brand) !important;
  box-shadow:
    inset 0 2px 4px rgba(80, 40, 100, 0.06),
    0 0 0 2.5px rgba(122, 88, 152, 0.15) !important;
}

[data-theme='dark'] .settings-page-wrapper .arco-input-wrapper,
[data-theme='dark'] .settings-page-wrapper .arco-textarea-wrapper,
[data-theme='dark'] .settings-page-wrapper .arco-input-inner-wrapper,
[data-theme='dark'] .settings-modal .arco-input-wrapper,
[data-theme='dark'] .settings-modal .arco-textarea-wrapper,
[data-theme='dark'] .settings-modal .arco-input-inner-wrapper {
  background: var(--bg-2) !important;
  border-color: var(--border-base) !important;
  box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.25) !important;
}

.arco-dropdown-menu,
.arco-select-popup {
  border-radius: 12px !important;
  border: 1.5px solid var(--border-base) !important;
  box-shadow:
    4px 4px 0 var(--border-base),
    6px 6px 18px rgba(80, 50, 120, 0.12) !important;
}

.arco-dropdown-menu {
  max-height: min(40vh, 220px) !important;
  overflow-x: hidden !important;
  overflow-y: auto !important;
  overscroll-behavior: contain !important;
  -webkit-overflow-scrolling: touch;
}

.arco-select-popup .arco-select-popup-inner {
  border-radius: inherit !important;
  max-height: min(40vh, 220px) !important;
  overflow-x: hidden !important;
  overflow-y: auto !important;
  overscroll-behavior: contain !important;
  -webkit-overflow-scrolling: touch;
}

[data-theme='dark'] .arco-dropdown-menu,
[data-theme='dark'] .arco-select-popup {
  background: var(--bg-2) !important;
  box-shadow:
    4px 4px 0 var(--bg-4),
    6px 6px 18px rgba(0, 0, 0, 0.5) !important;
}

.arco-modal {
  border-radius: 14px !important;
  overflow: visible !important;
  border: 2px solid var(--border-base) !important;
  box-shadow:
    6px 6px 0 var(--border-base),
    10px 10px 30px rgba(80, 50, 120, 0.2) !important;
}

.roseui-modal:not(.conversation-search-modal) .arco-modal-content {
  overflow: visible !important;
  max-height: calc(100vh - 32px) !important;
  box-sizing: border-box !important;
}

.roseui-modal:not(.conversation-search-modal) .roseui-modal-body-content {
  overflow: auto !important;
  max-height: calc(100vh - 148px) !important;
  box-sizing: border-box !important;
}

[data-theme='dark'] .arco-modal {
  border-color: var(--border-base) !important;
  box-shadow:
    6px 6px 0 var(--bg-4),
    10px 10px 30px rgba(0, 0, 0, 0.6) !important;
}

.arco-modal-header {
  background: linear-gradient(180deg, #faf4e8 0%, #f2e8d8 100%) !important;
  border-bottom: 3px solid #9878b8 !important;
  padding: 10px 20px !important;
}

[data-theme='dark'] .arco-modal-header {
  background: linear-gradient(180deg, #221630 0%, #1a1028 100%) !important;
  border-bottom: 3px solid #c090e0 !important;
}

.arco-modal-title {
  color: #5a3878 !important;
  font-weight: 700 !important;
  text-shadow: 0 1px 0 rgba(255, 255, 255, 0.6) !important;
}

[data-theme='dark'] .arco-modal-title {
  color: #e0cef8 !important;
  text-shadow: none !important;
}

.arco-card {
  border-radius: 12px !important;
  border: 1.5px solid var(--border-base) !important;
  box-shadow:
    3px 3px 0 var(--border-base),
    4px 4px 10px rgba(80, 50, 120, 0.08) !important;
  background: var(--bg-base) !important;
}

[data-theme='dark'] .arco-card {
  background: var(--bg-2) !important;
  border-color: var(--border-base) !important;
  box-shadow:
    3px 3px 0 var(--bg-4),
    4px 4px 10px rgba(0, 0, 0, 0.35) !important;
}

.arco-collapse {
  border-radius: 10px !important;
  border: 1px solid var(--border-base) !important;
  overflow: hidden !important;
}

.arco-tag {
  border-radius: 999px !important;
  font-weight: 500 !important;
  font-size: 12px !important;
  padding: 1px 10px !important;
  border: 1.5px solid currentColor !important;
}

.markdown-shadow-body h1,
[class*='markdown'] h1 {
  color: var(--brand);
}

.markdown-shadow-body h2,
[class*='markdown'] h2 {
  color: var(--color-primary);
}

.markdown-shadow-body h3,
[class*='markdown'] h3 {
  color: #6e8020;
}

[data-theme='dark'] .markdown-shadow-body h3,
[data-theme='dark'] [class*='markdown'] h3 {
  color: #a8c050;
}

.markdown-shadow-body h4,
[class*='markdown'] h4 {
  color: var(--success);
}

.markdown-shadow-body a,
[class*='markdown'] a {
  color: #b878a8;
  text-decoration-color: rgba(184, 120, 168, 0.4);
  text-underline-offset: 2px;
}

[data-theme='dark'] .markdown-shadow-body a,
[data-theme='dark'] [class*='markdown'] a {
  color: #d8a8e0;
}

.markdown-shadow-body pre,
[class*='markdown'] pre {
  background: var(--bg-2) !important;
  border: 1.5px solid var(--border-base) !important;
  border-radius: 8px !important;
  box-shadow: 3px 3px 0 var(--border-base) !important;
}

[data-theme='dark'] .markdown-shadow-body pre,
[data-theme='dark'] [class*='markdown'] pre {
  background: #2b3139 !important;
  border-color: var(--border-base) !important;
  box-shadow: 3px 3px 0 var(--bg-4) !important;
}

.markdown-shadow-body code:not(pre code),
[class*='markdown'] code:not(pre code) {
  background: var(--brand-light) !important;
  color: var(--brand) !important;
  border: 1px solid var(--aou-3) !important;
  border-radius: 4px !important;
  padding: 1px 5px !important;
  font-size: 0.88em !important;
}

[data-theme='dark'] .markdown-shadow-body code:not(pre code),
[data-theme='dark'] [class*='markdown'] code:not(pre code) {
  background: var(--brand-light) !important;
  color: var(--brand) !important;
  border-color: var(--aou-4) !important;
}

.aion-file-changes-panel {
  border: 1.5px solid #ccb8e0 !important;
  border-radius: 12px !important;
  box-shadow:
    0 0 0 1px rgba(168, 120, 192, 0.22),
    0 8px 18px rgba(152, 120, 184, 0.16) !important;
}

.aion-file-changes-panel > div:first-child {
  box-shadow: inset 0 -1px 0 rgba(184, 156, 210, 0.5);
}

[data-theme='dark'] .aion-file-changes-panel {
  border-color: #7f61a0 !important;
  box-shadow:
    0 0 0 1px rgba(177, 136, 212, 0.26),
    0 10px 22px rgba(58, 34, 88, 0.5) !important;
}

[data-theme='dark'] .aion-file-changes-panel > div:first-child {
  box-shadow: inset 0 -1px 0 rgba(148, 112, 186, 0.44);
}

.arco-divider {
  border-color: var(--border-base) !important;
  border-style: dashed !important;
}

::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}
::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: linear-gradient(180deg, #c4a8d8, #8a7aaa);
  border-radius: 3px;
}

::-webkit-scrollbar-thumb:hover {
  background: linear-gradient(180deg, #d4b8e8, #9a8aba);
}

[data-theme='dark'] ::-webkit-scrollbar-thumb {
  background: linear-gradient(180deg, #604878, #382250);
}

[data-theme='dark'] ::-webkit-scrollbar-thumb:hover {
  background: linear-gradient(180deg, #7a5c90, #4a2e68);
}

@keyframes retroma-particle-drift {
  0% {
    background-position:
      0 0,
      0 0,
      0 0,
      0 0,
      0 0,
      0 0,
      0 0;
  }
  25% {
    background-position:
      24px -32px,
      -22px 26px,
      28px 18px,
      -18px -12px,
      34px -20px,
      -26px 22px,
      18px -16px;
  }
  50% {
    background-position:
      52px -14px,
      -42px 48px,
      56px -20px,
      -46px 16px,
      64px -34px,
      -48px 44px,
      38px -28px;
  }
  75% {
    background-position:
      30px 20px,
      -18px 72px,
      24px -38px,
      -14px 34px,
      42px -18px,
      -22px 54px,
      20px -8px;
  }
  100% {
    background-position:
      0 0,
      0 0,
      0 0,
      0 0,
      0 0,
      0 0,
      0 0;
  }
}

@keyframes retroma-particle-drift-slow {
  0% {
    background-position:
      0 0,
      0 0,
      0 0;
    transform: translate3d(0, 0, 0);
  }
  50% {
    background-position:
      42px -28px,
      -34px 36px,
      52px 24px;
    transform: translate3d(0, -4px, 0);
  }
  100% {
    background-position:
      0 0,
      0 0,
      0 0;
    transform: translate3d(0, 0, 0);
  }
}

@keyframes retroma-particle-twinkle {
  0%,
  100% {
    opacity: 0.34;
  }
  50% {
    opacity: 0.58;
  }
}

::selection {
  background: rgba(184, 140, 220, 0.28);
}
[data-theme='dark'] ::selection {
  background: rgba(192, 144, 224, 0.32);
}

/* RoseUi Theme Background Start */
/* Preview cover only: do not auto-inject full-page background image */
/* RoseUi Theme Background End */
`,le=`/* Retroma Obsidian Theme - RoseUi Adaptation */
/* Inspired by retroma-obsidian-theme color philosophy */
/* Low-saturation organic palette with warm purples, teal blues, and olive greens */

:root {
  /* ===== Primary: Retroma Teal-Blue ===== */
  --color-primary: #0f7887;
  --primary: #0f7887;
  --color-primary-light-1: #2a9aac;
  --color-primary-light-2: #4db8c8;
  --color-primary-light-3: #80d0dc;
  --color-primary-dark-1: #0a5a66;
  --primary-rgb: 15, 120, 135;

  /* ===== Brand: Retroma Warm Purple ===== */
  --brand: #6e3a66;
  --brand-light: #f2e8f0;
  --brand-hover: #9d6094;
  --color-brand-fill: #6e3a66;
  --color-brand-bg: #f2e8f0;

  /* ===== AOU Palette: Warm Olive-Green Gradient ===== */
  --aou-1: #f4f4ec;
  --aou-2: #e8e9d8;
  --aou-3: #d0d2b0;
  --aou-4: #b4b888;
  --aou-5: #979d62;
  --aou-6: #737f16;
  --aou-7: #575f10;
  --aou-8: #3c420b;
  --aou-9: #222606;
  --aou-10: #0c0e02;

  /* ===== Backgrounds: Warm Parchment ===== */
  --bg-base: #faf9f6;
  --bg-1: #f5f4ef;
  --bg-2: #eeede5;
  --bg-3: #e2e0d4;
  --bg-4: #cbc8b8;
  --bg-5: #b0ac9a;
  --bg-6: #8c8878;
  --bg-8: #575450;
  --bg-9: #2c2b28;
  --bg-10: #111009;
  --color-bg-1: #f5f4ef;
  --color-bg-2: #eeede5;
  --color-bg-3: #e2e0d4;
  --color-bg-4: #cbc8b8;

  /* ===== Interactive States ===== */
  --bg-hover: #ebe9df;
  --bg-active: #e0ded4;

  /* ===== Text: Warm Dark ===== */
  --text-primary: #1d011d;
  --text-secondary: #6e6060;
  --text-disabled: #b8b0a8;
  --text-0: #1d011d;
  --text-white: #faf9f6;
  --color-text-1: #1d011d;
  --color-text-2: #6e6060;
  --color-text-3: #9e9490;
  --color-text-4: #c8c0bc;

  /* ===== Borders ===== */
  --border-base: #d8d4c8;
  --border-light: #e8e6dc;
  --border-special: #d0ccc0;
  --color-border: #d8d4c8;
  --color-border-1: #d8d4c8;
  --color-border-2: #e8e6dc;

  /* ===== Fill & Inverse ===== */
  --fill: #f5f4ef;
  --color-fill: #f5f4ef;
  --fill-0: #faf9f6;
  --fill-white-to-black: #faf9f6;
  --dialog-fill-0: #faf9f6;
  --inverse: #1d011d;

  /* ===== Semantic Colors ===== */
  --success: #35847e;
  --warning: #b07a10;
  --danger: #b03030;
  --info: #0f7887;

  /* ===== Message & Component ===== */
  --message-user-bg: #e9e4f0;
  --message-tips-bg: #f1edf6;
  --workspace-btn-bg: #eeece4;

  /* ===== Color GUID Agent Bar ===== */
  --color-guid-agent-bar: #eae8de;
  --hl-chip-bg: #f3eee2;
  --hl-chip-text: #5a4a3a;
  --hl-chip-border: #d7ccb7;
}

/* ===== Dark Mode Overrides ===== */
[data-theme='dark'] {
  /* Primary: Retroma Teal (dark-adjusted) */
  --color-primary: #6e8ddb;
  --primary: #6e8ddb;
  --color-primary-light-1: #8fa8e8;
  --color-primary-light-2: #aabff0;
  --color-primary-light-3: #c5d5f6;
  --color-primary-dark-1: #4f70c4;
  --primary-rgb: 110, 141, 219;

  /* Brand: Retroma Purple (dark) */
  --brand: #be80bf;
  --brand-light: #3d2840;
  --brand-hover: #9a60a0;
  --color-brand-fill: #be80bf;
  --color-brand-bg: #3d2840;

  /* AOU Palette: Dark Olive */
  --aou-1: #232318;
  --aou-2: #363525;
  --aou-3: #4a4a30;
  --aou-4: #666640;
  --aou-5: #898a54;
  --aou-6: #a2a554;
  --aou-7: #bbbf6e;
  --aou-8: #d0d490;
  --aou-9: #e4e6b8;
  --aou-10: #f2f4da;

  /* Backgrounds: Dark Obsidian with warm undertones */
  --bg-base: #0f0f0c;
  --bg-1: #18180f;
  --bg-2: #222217;
  --bg-3: #2e2e20;
  --bg-4: #3c3c2c;
  --bg-5: #4e4e3a;
  --bg-6: #606050;
  --bg-8: #848470;
  --bg-9: #b0b09a;
  --bg-10: #d8d8c8;
  --color-bg-1: #18180f;
  --color-bg-2: #222217;
  --color-bg-3: #2e2e20;
  --color-bg-4: #3c3c2c;

  /* Interactive States */
  --bg-hover: #1e1e14;
  --bg-active: #28281c;

  /* Text: Retroma warm near-white */
  --text-primary: #e8e6d8;
  --text-secondary: #a8a498;
  --text-disabled: #686458;
  --text-0: #f0ede0;
  --text-white: #f0ede0;
  --color-text-1: #e8e6d8;
  --color-text-2: #a8a498;
  --color-text-3: #787468;
  --color-text-4: #504c44;

  /* Borders */
  --border-base: #3a3a28;
  --border-light: #2a2a1e;
  --border-special: #4a4a36;
  --color-border: #3a3a28;
  --color-border-1: #3a3a28;
  --color-border-2: #2a2a1e;

  /* Fill & Inverse */
  --fill: #18180f;
  --color-fill: #18180f;
  --fill-0: rgba(255, 252, 240, 0.07);
  --fill-white-to-black: #0f0f0c;
  --dialog-fill-0: #2e2e20;
  --inverse: #f0ede0;

  /* Semantic Colors */
  --success: #68a99d;
  --warning: #d4963a;
  --danger: #c86060;
  --info: #6e8ddb;

  /* Message & Component */
  --message-user-bg: #3a3444;
  --message-tips-bg: #2f2a38;
  --workspace-btn-bg: #1e1e14;

  /* Color GUID Agent Bar */
  --color-guid-agent-bar: #2a2a1e;
  --hl-chip-bg: #d6ccb8;
  --hl-chip-text: #3f3528;
  --hl-chip-border: #b5a88f;
}

/* ===== Typography ===== */
body {
  font-family: 'Georgia', 'Palatino Linotype', 'Book Antiqua', 'Source Han Serif', serif;
  letter-spacing: 0.01em;
}

/* ===== Scrollbar Styling ===== */
::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

::-webkit-scrollbar-track {
  background: var(--bg-1);
}

::-webkit-scrollbar-thumb {
  background: var(--bg-4);
  border-radius: 3px;
}

::-webkit-scrollbar-thumb:hover {
  background: var(--brand);
}

/* ===== Message Headings (H1-H5) ===== */
.message-content h1,
.markdown-body h1 {
  color: var(--brand);
}

.message-content h2,
.markdown-body h2 {
  color: var(--color-primary);
}

.message-content h3,
.markdown-body h3 {
  color: var(--aou-6);
}

.message-content h4,
.markdown-body h4 {
  color: var(--success);
}

.message-content h5,
.markdown-body h5 {
  color: #465881;
}

[data-theme='dark'] .message-content h5,
[data-theme='dark'] .markdown-body h5 {
  color: #7a90c8;
}

/* ===== Links ===== */
.message-content a,
.markdown-body a,
.markdown-shadow-body a,
[class*='markdown'] a {
  color: #5b63cf;
  -webkit-text-fill-color: #5b63cf;
  text-decoration-color: rgba(91, 99, 207, 0.55);
  text-underline-offset: 2px;
}

[data-theme='light'] .message-content a:hover,
[data-theme='light'] .markdown-body a:hover,
[data-theme='light'] .markdown-shadow-body a:hover,
[data-theme='light'] [class*='markdown'] a:hover {
  color: #464fc0 !important;
  -webkit-text-fill-color: #464fc0 !important;
  text-decoration-color: rgba(70, 79, 192, 0.8) !important;
}

[data-theme='light'] .message-content a:visited,
[data-theme='light'] .markdown-body a:visited,
[data-theme='light'] .markdown-shadow-body a:visited,
[data-theme='light'] [class*='markdown'] a:visited {
  color: #7a63b0 !important;
  -webkit-text-fill-color: #7a63b0 !important;
  text-decoration-color: rgba(122, 99, 176, 0.65) !important;
}

[data-theme='dark'] .message-content a,
[data-theme='dark'] .markdown-body a,
[data-theme='dark'] .markdown-shadow-body a,
[data-theme='dark'] [class*='markdown'] a {
  color: #cfe0ff !important;
  -webkit-text-fill-color: #cfe0ff !important;
  text-decoration-color: rgba(207, 224, 255, 0.85) !important;
}

[data-theme='dark'] .message-content a:hover,
[data-theme='dark'] .markdown-body a:hover,
[data-theme='dark'] .markdown-shadow-body a:hover,
[data-theme='dark'] [class*='markdown'] a:hover {
  color: #e7efff !important;
  -webkit-text-fill-color: #e7efff !important;
  text-decoration-color: rgba(231, 239, 255, 0.98) !important;
}

[data-theme='dark'] .message-content a:visited,
[data-theme='dark'] .markdown-body a:visited,
[data-theme='dark'] .markdown-shadow-body a:visited,
[data-theme='dark'] [class*='markdown'] a:visited {
  color: #d4c6fa !important;
  -webkit-text-fill-color: #d4c6fa !important;
  text-decoration-color: rgba(212, 198, 250, 0.82) !important;
}

/* ===== Code Blocks ===== */
.message-content pre,
.markdown-body pre {
  background: var(--bg-2);
  border: 1px solid var(--border-base);
  border-radius: 6px;
}

[data-theme='dark'] .message-content pre,
[data-theme='dark'] .markdown-body pre {
  background: #141410;
  border-color: var(--border-base);
}

/* ===== Inline Code / Highlight Chip ===== */
.message-content code:not(pre code),
.markdown-body code:not(pre code),
.markdown-shadow-body code:not(pre code),
[class*='markdown'] code:not(pre code) {
  background: var(--hl-chip-bg) !important;
  color: var(--hl-chip-text) !important;
  border: 1px solid var(--hl-chip-border) !important;
  border-radius: 7px !important;
  padding: 1px 8px !important;
  font-size: 0.9em !important;
  font-weight: 650 !important;
  opacity: 1 !important;
  filter: none !important;
  text-shadow: none !important;
  -webkit-text-fill-color: var(--hl-chip-text) !important;
  background-clip: border-box !important;
}

[data-theme='dark'] .message-content code:not(pre code),
[data-theme='dark'] .markdown-body code:not(pre code),
[data-theme='dark'] .markdown-shadow-body code:not(pre code),
[data-theme='dark'] [class*='markdown'] code:not(pre code) {
  background: var(--hl-chip-bg) !important;
  color: var(--hl-chip-text) !important;
  border-color: var(--hl-chip-border) !important;
  box-shadow: inset 0 0 0 1px rgba(90, 66, 108, 0.12) !important;
  opacity: 1 !important;
  filter: none !important;
  text-shadow: none !important;
  -webkit-text-fill-color: var(--hl-chip-text) !important;
}

/* ===== Emphasis Highlight (Bold with Background) ===== */
.message-content strong,
.markdown-body strong,
.markdown-shadow-body strong,
[class*='markdown'] strong {
  background: var(--hl-chip-bg) !important;
  color: var(--hl-chip-text) !important;
  padding: 0 6px !important;
  border-radius: 4px !important;
  border: 1px solid var(--hl-chip-border) !important;
  box-decoration-break: clone;
  -webkit-box-decoration-break: clone;
  opacity: 1 !important;
  filter: none !important;
  text-shadow: none !important;
  -webkit-text-fill-color: var(--hl-chip-text) !important;
}

[data-theme='dark'] .message-content strong,
[data-theme='dark'] .markdown-body strong,
[data-theme='dark'] .markdown-shadow-body strong,
[data-theme='dark'] [class*='markdown'] strong {
  background: var(--hl-chip-bg) !important;
  color: var(--hl-chip-text) !important;
  border-color: var(--hl-chip-border) !important;
  box-shadow: inset 0 0 0 1px rgba(90, 66, 108, 0.1) !important;
  opacity: 1 !important;
  filter: none !important;
  text-shadow: none !important;
  -webkit-text-fill-color: var(--hl-chip-text) !important;
}

[data-theme='light'] .message-content mark,
[data-theme='light'] .markdown-body mark,
[data-theme='light'] .markdown-shadow-body mark,
[data-theme='light'] [class*='markdown'] mark,
[data-theme='dark'] .message-content mark,
[data-theme='dark'] .markdown-body mark,
[data-theme='dark'] .markdown-shadow-body mark,
[data-theme='dark'] [class*='markdown'] mark {
  background: var(--hl-chip-bg) !important;
  color: var(--hl-chip-text) !important;
  border: 1px solid var(--hl-chip-border) !important;
  border-radius: 4px !important;
  padding: 0 4px !important;
  opacity: 1 !important;
  filter: none !important;
  text-shadow: none !important;
  -webkit-text-fill-color: var(--hl-chip-text) !important;
}

.message-content strong *,
.markdown-body strong *,
.markdown-shadow-body strong *,
[class*='markdown'] strong *,
.message-content code:not(pre code) *,
.markdown-body code:not(pre code) *,
.markdown-shadow-body code:not(pre code) *,
[class*='markdown'] code:not(pre code) *,
.message-content mark *,
.markdown-body mark *,
.markdown-shadow-body mark *,
[class*='markdown'] mark * {
  color: var(--hl-chip-text) !important;
  -webkit-text-fill-color: var(--hl-chip-text) !important;
  opacity: 1 !important;
}

/* ===== Sidebar ===== */
.layout-sider {
  background-color: var(--bg-1);
  border-right: 1px solid var(--border-base);
}

/* ===== Conversation Bubble (AOU purple-gray) ===== */
.message-item.user .message-bubble {
  background: var(--message-user-bg) !important;
  border: 1px solid #cbc0da !important;
}

[data-theme='dark'] .message-item.user .message-bubble {
  background: var(--message-user-bg) !important;
  border-color: color-mix(in srgb, var(--aou-5) 46%, var(--border-base)) !important;
}

/* ===== Selection Highlight ===== */
::selection {
  background: color-mix(in srgb, var(--brand) 25%, transparent);
}

[data-theme='dark'] ::selection {
  background: color-mix(in srgb, var(--brand) 30%, transparent);
}

/* RoseUi Theme Background Start */
/* Preview cover only: do not auto-inject full-page background image */
/* RoseUi Theme Background End */
`,pe=`/* Discourse Horizon Theme - RoseUi Adaptation */
/* Inspired by the official Horizon theme for Discourse */
/* Spacious surfaces, soft indigo accents, pill controls, and quiet contrast */

:root {
  /* ===== Primary: Horizon Indigo ===== */
  --color-primary: #595bca;
  --primary: #595bca;
  --color-primary-light-1: #7678d7;
  --color-primary-light-2: #9697e4;
  --color-primary-light-3: #bbbdf0;
  --color-primary-dark-1: #4347a7;
  --primary-rgb: 89, 91, 202;

  /* ===== Brand: Soft Slate Indigo ===== */
  --brand: #6f76a9;
  --brand-light: #f0f2ff;
  --brand-hover: #9096c4;
  --color-brand-fill: #6f76a9;
  --color-brand-bg: #f0f2ff;

  /* ===== AOU Palette: Indigo Mist ===== */
  --aou-1: #f7f8ff;
  --aou-2: #eef0ff;
  --aou-3: #d7dfff;
  --aou-4: #c0c9f2;
  --aou-5: #9fa8df;
  --aou-6: #7c84d2;
  --aou-7: #595bca;
  --aou-8: #41449a;
  --aou-9: #2a2d66;
  --aou-10: #16173a;

  /* ===== Backgrounds ===== */
  --bg-base: #ffffff;
  --bg-1: #f7f8fc;
  --bg-2: #f2f4fb;
  --bg-3: #e6e9f3;
  --bg-4: #ced5e4;
  --bg-5: #aeb7c9;
  --bg-6: #8690a6;
  --bg-8: #566074;
  --bg-9: #2d3444;
  --bg-10: #141824;
  --color-bg-1: #f7f8fc;
  --color-bg-2: #f2f4fb;
  --color-bg-3: #e6e9f3;
  --color-bg-4: #ced5e4;

  /* ===== Interactive States ===== */
  --bg-hover: #eef1fa;
  --bg-active: #e4e8f5;

  /* ===== Text ===== */
  --text-primary: #1a1a1a;
  --text-secondary: #646b7c;
  --text-disabled: #a8afbf;
  --text-0: #111111;
  --text-white: #ffffff;
  --color-text-1: #1a1a1a;
  --color-text-2: #646b7c;
  --color-text-3: #8b93a7;
  --color-text-4: #b6becf;

  /* ===== Borders ===== */
  --border-base: #e4e8f3;
  --border-light: #f1f4fa;
  --border-special: #d7dfff;
  --color-border: #e4e8f3;
  --color-border-1: #e4e8f3;
  --color-border-2: #f1f4fa;

  /* ===== Fill & Inverse ===== */
  --fill: #f7f8fc;
  --color-fill: #f7f8fc;
  --fill-0: #ffffff;
  --fill-white-to-black: #ffffff;
  --dialog-fill-0: #ffffff;
  --inverse: #10121a;

  /* ===== Semantic Colors ===== */
  --success: #39845b;
  --warning: #d3881f;
  --danger: #d14b54;
  --info: #595bca;

  /* ===== Message & Component ===== */
  --message-user-bg: #eef0ff;
  --message-tips-bg: #f6f7fc;
  --workspace-btn-bg: #f2f4fb;
  --color-guid-agent-bar: #f5f7ff;
  --hl-chip-bg: #edf0ff;
  --hl-chip-text: #41449a;
  --hl-chip-border: #d4dcff;
  --horizon-shell-bg: #eef0ff;
  --horizon-pane-bg: #f5f6ff;
  --horizon-surface: #ffffff;
  --horizon-surface-soft: #f7f8fc;
  --horizon-selected: #d7dfff;
  --horizon-focus-ring: rgba(89, 91, 202, 0.18);
  --horizon-shadow-soft: 0 8px 24px -22px rgba(45, 52, 68, 0.22);
  --horizon-shadow-hover: 0 16px 28px -24px rgba(89, 91, 202, 0.24);
  --horizon-aurora-input-gradient: linear-gradient(
    90deg,
    #ff6a01 0%,
    #f8c91c 12.5%,
    #8a2be2 25%,
    #00bfff 37.5%,
    #ff6a01 50%,
    #f8c91c 62.5%,
    #8a2be2 75%,
    #00bfff 87.5%,
    #ff6a01 100%
  );
  --horizon-aurora-input-ring: rgba(255, 162, 84, 0.24);
  --horizon-aurora-input-shadow: 0 18px 40px rgba(110, 58, 102, 0.16), 0 0 28px rgba(15, 120, 135, 0.1);
  --horizon-aurora-input-shadow-strong: 0 22px 48px rgba(110, 58, 102, 0.2), 0 0 34px rgba(0, 191, 255, 0.18);
  --horizon-aurora-placeholder: #8f7c7b;
}

[data-theme='dark'] {
  /* ===== Primary: Horizon Indigo Dark ===== */
  --color-primary: #7b7ff0;
  --primary: #7b7ff0;
  --color-primary-light-1: #979af7;
  --color-primary-light-2: #b2b5fb;
  --color-primary-light-3: #ccd0ff;
  --color-primary-dark-1: #5e62da;
  --primary-rgb: 123, 127, 240;

  /* ===== Brand: Muted Mist ===== */
  --brand: #a0a5cf;
  --brand-light: #2c3044;
  --brand-hover: #b9bde1;
  --color-brand-fill: #a0a5cf;
  --color-brand-bg: #2c3044;

  /* ===== AOU Palette: Indigo Slate ===== */
  --aou-1: #242632;
  --aou-2: #2c3040;
  --aou-3: #3b3e56;
  --aou-4: #4c516d;
  --aou-5: #62698c;
  --aou-6: #7b81aa;
  --aou-7: #9ca2ca;
  --aou-8: #c0c6ea;
  --aou-9: #e1e5ff;
  --aou-10: #f5f7ff;

  /* ===== Backgrounds ===== */
  --bg-base: #1a1a1a;
  --bg-1: #202125;
  --bg-2: #262834;
  --bg-3: #333548;
  --bg-4: #43475c;
  --bg-5: #5c6278;
  --bg-6: #788099;
  --bg-8: #a8aec3;
  --bg-9: #d3d7e2;
  --bg-10: #f0f2f7;
  --color-bg-1: #202125;
  --color-bg-2: #262834;
  --color-bg-3: #333548;
  --color-bg-4: #43475c;

  /* ===== Interactive States ===== */
  --bg-hover: #2a2d3b;
  --bg-active: #323647;

  /* ===== Text ===== */
  --text-primary: #f5f6f8;
  --text-secondary: #c1c5d1;
  --text-disabled: #747c8d;
  --text-0: #ffffff;
  --text-white: #ffffff;
  --color-text-1: #f5f6f8;
  --color-text-2: #c1c5d1;
  --color-text-3: #8f96a8;
  --color-text-4: #666d7d;

  /* ===== Borders ===== */
  --border-base: #373b4d;
  --border-light: #2c3040;
  --border-special: #4a5070;
  --color-border: #373b4d;
  --color-border-1: #373b4d;
  --color-border-2: #2c3040;

  /* ===== Fill & Inverse ===== */
  --fill: #202125;
  --color-fill: #202125;
  --fill-0: rgba(255, 255, 255, 0.08);
  --fill-white-to-black: #1a1a1a;
  --dialog-fill-0: #262834;
  --inverse: #ffffff;

  /* ===== Semantic Colors ===== */
  --success: #67ac86;
  --warning: #e5a33a;
  --danger: #e06d76;
  --info: #7b7ff0;

  /* ===== Message & Component ===== */
  --message-user-bg: #303459;
  --message-tips-bg: #232632;
  --workspace-btn-bg: #252935;
  --color-guid-agent-bar: #21242f;
  --hl-chip-bg: #313651;
  --hl-chip-text: #dce0ff;
  --hl-chip-border: #474d71;
  --horizon-shell-bg: #09070f;
  --horizon-pane-bg: #110f18;
  --horizon-surface: #1c1d22;
  --horizon-surface-soft: #22242d;
  --horizon-selected: #3b3e56;
  --horizon-focus-ring: rgba(123, 127, 240, 0.22);
  --horizon-shadow-soft: 0 12px 28px -24px rgba(0, 0, 0, 0.45);
  --horizon-shadow-hover: 0 18px 32px -24px rgba(0, 0, 0, 0.58);
  --horizon-aurora-input-gradient: linear-gradient(
    90deg,
    #ff6a01 0%,
    #f8c91c 12.5%,
    #8a2be2 25%,
    #00bfff 37.5%,
    #ff6a01 50%,
    #f8c91c 62.5%,
    #8a2be2 75%,
    #00bfff 87.5%,
    #ff6a01 100%
  );
  --horizon-aurora-input-ring: rgba(166, 214, 255, 0.22);
  --horizon-aurora-input-shadow: 0 22px 48px rgba(6, 10, 18, 0.5), 0 0 32px rgba(138, 43, 226, 0.16);
  --horizon-aurora-input-shadow-strong: 0 26px 54px rgba(4, 8, 16, 0.62), 0 0 38px rgba(0, 191, 255, 0.22);
  --horizon-aurora-placeholder: #958a83;
}

body {
  font-family: 'Georgia', 'Palatino Linotype', 'Book Antiqua', 'Source Han Serif', serif;
  letter-spacing: 0.003em;
  background: linear-gradient(180deg, var(--horizon-shell-bg) 0%, var(--bg-1) 100%) !important;
  color: var(--text-primary);
}

[data-theme='dark'] body {
  background: linear-gradient(180deg, var(--horizon-shell-bg) 0%, var(--bg-base) 100%) !important;
}

.app-shell,
.layout-content.bg-1,
.layout-content,
.layout.arco-layout,
.arco-layout {
  background: transparent !important;
}

.app-titlebar {
  background: var(--horizon-pane-bg) !important;
  border-bottom: 1px solid var(--border-base) !important;
  box-shadow: none !important;
}

[data-theme='dark'] .app-titlebar {
  background: var(--horizon-pane-bg) !important;
}

.app-titlebar__brand {
  color: var(--text-primary) !important;
  font-weight: 700 !important;
  letter-spacing: 0.01em !important;
}

.app-titlebar__button {
  border-radius: 999px !important;
  color: var(--text-secondary) !important;
  transition:
    background-color 0.16s ease,
    color 0.16s ease,
    box-shadow 0.16s ease !important;
}

.app-titlebar__button:hover {
  background: color-mix(in srgb, var(--horizon-selected) 55%, var(--horizon-pane-bg)) !important;
  color: var(--color-primary) !important;
}

.layout-sider {
  background: var(--horizon-pane-bg) !important;
  border-right: 1px solid var(--border-base) !important;
  box-shadow: none !important;
}

[data-theme='dark'] .layout-sider {
  background: var(--horizon-pane-bg) !important;
}

.layout-sider-header {
  background: transparent !important;
  border-bottom: 1px solid color-mix(in srgb, var(--border-base) 80%, transparent) !important;
}

.chat-history__item,
.settings-sider__item {
  position: relative;
  margin: 4px 8px !important;
  border: 1px solid transparent !important;
  border-radius: 14px !important;
  transition:
    background-color 0.16s ease,
    border-color 0.16s ease,
    box-shadow 0.16s ease,
    color 0.16s ease !important;
}

.chat-history__item:hover,
.settings-sider__item:hover {
  background: color-mix(in srgb, var(--horizon-selected) 48%, var(--horizon-pane-bg)) !important;
  border-color: color-mix(in srgb, var(--border-base) 76%, var(--aou-3)) !important;
  box-shadow: inset 0 0 0 1px rgba(var(--primary-rgb), 0.04) !important;
}

.chat-history__item--active,
.chat-history__item[aria-selected='true'],
.chat-history__item[class~='!bg-active'],
.settings-sider__item[class~='!bg-aou-2'] {
  background: linear-gradient(180deg, var(--aou-2) 0%, var(--bg-base) 100%) !important;
  border-color: var(--aou-3) !important;
  box-shadow:
    var(--horizon-shadow-soft),
    inset 0 1px 0 rgba(255, 255, 255, 0.82) !important;
}

.chat-history__item--active::before,
.chat-history__item[aria-selected='true']::before,
.chat-history__item[class~='!bg-active']::before,
.settings-sider__item[class~='!bg-aou-2']::before {
  content: '';
  position: absolute;
  left: 8px;
  top: 9px;
  bottom: 9px;
  width: 4px;
  border-radius: 999px;
  background: linear-gradient(180deg, var(--color-primary) 0%, var(--color-primary-light-2) 100%);
}

[data-theme='dark'] .chat-history__item:hover,
[data-theme='dark'] .settings-sider__item:hover {
  background: color-mix(in srgb, var(--horizon-selected) 42%, var(--horizon-pane-bg)) !important;
}

[data-theme='dark'] .chat-history__item--active,
[data-theme='dark'] .chat-history__item[aria-selected='true'],
[data-theme='dark'] .chat-history__item[class~='!bg-active'],
[data-theme='dark'] .settings-sider__item[class~='!bg-aou-2'] {
  background: linear-gradient(135deg, rgba(59, 62, 86, 0.96) 0%, rgba(49, 54, 77, 0.98) 100%) !important;
  box-shadow:
    var(--horizon-shadow-soft),
    inset 0 1px 0 rgba(255, 255, 255, 0.06) !important;
}

.chat-history--collapsed .chat-history__item,
.settings-sider--collapsed .settings-sider__item,
.layout-sider.collapsed .chat-history__item,
.layout-sider.arco-layout-sider-collapsed .chat-history__item,
.layout-sider.collapsed .settings-sider__item,
.layout-sider.arco-layout-sider-collapsed .settings-sider__item {
  margin: 6px auto !important;
  width: 40px !important;
  min-width: 40px !important;
  min-height: 40px !important;
  padding: 0 !important;
  justify-content: center !important;
  align-items: center !important;
  gap: 0 !important;
}

.chat-history--collapsed,
.settings-sider--collapsed {
  scrollbar-width: none !important;
}

.chat-history--collapsed::-webkit-scrollbar,
.settings-sider--collapsed::-webkit-scrollbar,
.layout-sider.collapsed .overflow-y-auto::-webkit-scrollbar,
.layout-sider.arco-layout-sider-collapsed .overflow-y-auto::-webkit-scrollbar,
.layout-sider.collapsed .arco-layout-sider-children::-webkit-scrollbar,
.layout-sider.arco-layout-sider-collapsed .arco-layout-sider-children::-webkit-scrollbar {
  width: 0 !important;
  height: 0 !important;
}

.chat-history--collapsed .chat-history__item > :first-child,
.settings-sider--collapsed .settings-sider__item > :first-child,
.layout-sider.collapsed .chat-history__item > :first-child,
.layout-sider.arco-layout-sider-collapsed .chat-history__item > :first-child,
.layout-sider.collapsed .settings-sider__item > :first-child,
.layout-sider.arco-layout-sider-collapsed .settings-sider__item > :first-child {
  margin: 0 auto !important;
}

.layout-sider.collapsed .settings-sider .settings-sider__item > :nth-child(2),
.layout-sider.arco-layout-sider-collapsed .settings-sider .settings-sider__item > :nth-child(2),
.settings-sider--collapsed .settings-sider__item > :nth-child(2) {
  display: none !important;
}

.layout-sider.collapsed .conversation-item,
.layout-sider.arco-layout-sider-collapsed .conversation-item,
.chat-history--collapsed .conversation-item,
.settings-sider--collapsed .conversation-item {
  padding: 0 !important;
  justify-content: center !important;
  align-items: center !important;
  gap: 0 !important;
}

.layout-sider.collapsed .settings-sider .settings-sider__item > svg,
.layout-sider.arco-layout-sider-collapsed .settings-sider .settings-sider__item > svg,
.layout-sider.collapsed .settings-sider .settings-sider__item > img,
.layout-sider.arco-layout-sider-collapsed .settings-sider .settings-sider__item > img,
.layout-sider.collapsed .settings-sider .settings-sider__item > .mt-2px,
.layout-sider.collapsed .settings-sider .settings-sider__item > [class*='w-20px'],
.layout-sider.arco-layout-sider-collapsed .settings-sider .settings-sider__item > .mt-2px,
.layout-sider.arco-layout-sider-collapsed .settings-sider .settings-sider__item > [class*='w-20px'],
.layout-sider.collapsed .chat-history__item > svg,
.layout-sider.arco-layout-sider-collapsed .chat-history__item > svg,
.layout-sider.collapsed .chat-history__item > img,
.layout-sider.arco-layout-sider-collapsed .chat-history__item > img,
.layout-sider.collapsed .chat-history__item > .mt-2px,
.layout-sider.arco-layout-sider-collapsed .chat-history__item > .mt-2px,
.layout-sider.collapsed .chat-history__item > [class*='w-24px'],
.layout-sider.arco-layout-sider-collapsed .chat-history__item > [class*='w-24px'] {
  margin: 0 auto !important;
  width: 24px !important;
  height: 24px !important;
  min-width: 24px !important;
  display: flex !important;
  align-items: center !important;
  justify-content: center !important;
  transform: none !important;
}

.chat-history--collapsed .chat-history__item--active,
.chat-history--collapsed .chat-history__item[aria-selected='true'],
.chat-history--collapsed .chat-history__item[class~='!bg-active'],
.settings-sider--collapsed .settings-sider__item[class~='!bg-aou-2'],
.layout-sider.collapsed .settings-sider [data-settings-id][class*='!bg-aou-2'],
.layout-sider.arco-layout-sider-collapsed .settings-sider [data-settings-id][class*='!bg-aou-2'] {
  box-shadow: inset 0 0 0 1px rgba(var(--primary-rgb), 0.2) !important;
}

.chat-history--collapsed .chat-history__item--active::before,
.chat-history--collapsed .chat-history__item[aria-selected='true']::before,
.chat-history--collapsed .chat-history__item[class~='!bg-active']::before,
.settings-sider--collapsed .settings-sider__item[class~='!bg-aou-2']::before,
.layout-sider.collapsed .conversation-item::before,
.layout-sider.arco-layout-sider-collapsed .conversation-item::before,
.chat-history--collapsed .conversation-item::before,
.settings-sider--collapsed .conversation-item::before {
  content: none !important;
  display: none !important;
}

[data-theme='dark'] .chat-history--collapsed .chat-history__item--active,
[data-theme='dark'] .chat-history--collapsed .chat-history__item[aria-selected='true'],
[data-theme='dark'] .chat-history--collapsed .chat-history__item[class~='!bg-active'],
[data-theme='dark'] .settings-sider--collapsed .settings-sider__item[class~='!bg-aou-2'],
[data-theme='dark'] .layout-sider.collapsed .settings-sider [data-settings-id][class*='!bg-aou-2'],
[data-theme='dark'] .layout-sider.arco-layout-sider-collapsed .settings-sider [data-settings-id][class*='!bg-aou-2'] {
  box-shadow: inset 0 0 0 1px rgba(123, 127, 240, 0.24) !important;
}

.chat-layout-header {
  background: var(--horizon-surface) !important;
  border-bottom: 1px solid color-mix(in srgb, var(--border-base) 86%, transparent) !important;
  box-shadow: none !important;
}

[data-theme='dark'] .chat-layout-header {
  background: var(--horizon-surface-soft) !important;
}

.bg-dialog-fill-0 {
  background-color: var(--dialog-fill-0) !important;
}

.settings-modal .bg-2.rd-16px,
.settings-modal .bg-2.rd-12px,
.settings-modal .arco-collapse-item.bg-2,
.settings-page-wrapper .bg-2.rd-16px,
.settings-page-wrapper .bg-2.rd-12px,
.settings-page-wrapper .arco-collapse-item.bg-2,
.arco-card,
.aion-file-changes-panel {
  background: linear-gradient(180deg, var(--horizon-surface) 0%, var(--horizon-surface-soft) 100%) !important;
  border: 1px solid var(--border-base) !important;
  border-radius: 20px !important;
  box-shadow: var(--horizon-shadow-soft) !important;
}

[data-theme='dark'] .settings-modal .bg-2.rd-16px,
[data-theme='dark'] .settings-modal .bg-2.rd-12px,
[data-theme='dark'] .settings-modal .arco-collapse-item.bg-2,
[data-theme='dark'] .settings-page-wrapper .bg-2.rd-16px,
[data-theme='dark'] .settings-page-wrapper .bg-2.rd-12px,
[data-theme='dark'] .settings-page-wrapper .arco-collapse-item.bg-2,
[data-theme='dark'] .arco-card,
[data-theme='dark'] .aion-file-changes-panel {
  background: linear-gradient(180deg, var(--horizon-surface-soft) 0%, var(--horizon-surface) 100%) !important;
  box-shadow: var(--horizon-shadow-soft) !important;
}

.arco-modal,
.arco-dropdown-menu,
.arco-select-popup {
  background: var(--horizon-surface) !important;
  border-radius: 20px !important;
  border: 1px solid var(--border-base) !important;
  box-shadow: var(--horizon-shadow-soft) !important;
}

[data-theme='dark'] .arco-modal,
[data-theme='dark'] .arco-dropdown-menu,
[data-theme='dark'] .arco-select-popup {
  background: var(--horizon-surface-soft) !important;
  box-shadow: var(--horizon-shadow-soft) !important;
}

.arco-btn {
  border-radius: 999px !important;
  font-weight: 600 !important;
  transition:
    background-color 0.16s ease,
    border-color 0.16s ease,
    color 0.16s ease,
    box-shadow 0.16s ease !important;
}

.arco-btn-primary,
.send-button-custom,
.send-button-custom.arco-btn,
.send-button-custom.arco-btn-primary {
  background: var(--color-primary) !important;
  border-color: transparent !important;
  color: #ffffff !important;
  box-shadow: none !important;
}

.arco-btn-primary:hover,
.arco-btn-primary:focus-visible,
.send-button-custom:hover,
.send-button-custom.arco-btn:hover,
.send-button-custom:focus-visible,
.send-button-custom.arco-btn:focus-visible {
  background: var(--color-primary) !important;
  box-shadow: 0 0 0 4px var(--horizon-focus-ring) !important;
}

.arco-btn-secondary,
.arco-btn-outline {
  background: color-mix(in srgb, var(--horizon-surface) 90%, var(--aou-1)) !important;
  border: 1px solid var(--border-base) !important;
  color: var(--text-primary) !important;
  box-shadow: none !important;
}

.arco-btn-secondary:hover,
.arco-btn-secondary:focus-visible,
.arco-btn-outline:hover,
.arco-btn-outline:focus-visible {
  border-color: var(--aou-4) !important;
  color: var(--color-primary) !important;
  box-shadow: 0 0 0 4px color-mix(in srgb, var(--horizon-focus-ring) 82%, transparent) !important;
}

[data-theme='dark'] .arco-btn-secondary,
[data-theme='dark'] .arco-btn-outline {
  background: color-mix(in srgb, var(--horizon-surface-soft) 92%, var(--aou-2)) !important;
}

.arco-input-wrapper,
.arco-textarea-wrapper,
.arco-input-inner-wrapper {
  border-radius: 8px !important;
  border: 1px solid var(--border-base) !important;
  background: color-mix(in srgb, var(--horizon-surface) 92%, var(--aou-1)) !important;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.88) !important;
  transition:
    border-color 0.16s ease,
    box-shadow 0.16s ease !important;
}

.arco-input-wrapper:focus-within,
.arco-textarea-wrapper:focus-within,
.arco-input-inner-wrapper:focus-within {
  border-color: var(--color-primary) !important;
  box-shadow: 0 0 0 4px var(--horizon-focus-ring) !important;
}

[data-theme='dark'] .arco-input-wrapper,
[data-theme='dark'] .arco-textarea-wrapper,
[data-theme='dark'] .arco-input-inner-wrapper {
  background: color-mix(in srgb, var(--horizon-surface-soft) 92%, var(--aou-2)) !important;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04) !important;
}

.arco-input-wrapper input::placeholder,
.arco-textarea-wrapper textarea::placeholder,
.arco-input-inner-wrapper input::placeholder {
  color: var(--text-secondary) !important;
  opacity: 0.82 !important;
}

.arco-select-view,
.arco-picker,
.arco-picker-input,
.arco-picker-focused .arco-picker-input,
.arco-select-view-single,
.arco-select-view-multiple {
  position: relative;
  border-radius: 8px !important;
  border: 1px solid var(--border-base) !important;
  background: color-mix(in srgb, var(--horizon-surface) 92%, var(--aou-1)) !important;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.88) !important;
  transition:
    border-color 0.16s ease,
    box-shadow 0.16s ease !important;
}

.arco-select-view:hover,
.arco-picker:hover,
.arco-picker-input:hover {
  border-color: color-mix(in srgb, var(--color-primary) 40%, var(--border-base)) !important;
}

.arco-select-view:focus-within,
.arco-select-view.arco-select-view-focus,
.arco-picker-focused,
.arco-picker-focused .arco-picker-input {
  border-color: var(--color-primary) !important;
  box-shadow: 0 0 0 4px var(--horizon-focus-ring) !important;
}

[data-theme='dark'] .arco-select-view,
[data-theme='dark'] .arco-picker,
[data-theme='dark'] .arco-picker-input,
[data-theme='dark'] .arco-picker-focused .arco-picker-input,
[data-theme='dark'] .arco-select-view-single,
[data-theme='dark'] .arco-select-view-multiple {
  background: color-mix(in srgb, var(--horizon-surface-soft) 92%, var(--aou-2)) !important;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04) !important;
}

.arco-select-view-value,
.arco-picker input,
.arco-picker-value,
.arco-picker-suffix-icon,
.arco-select-view-arrow-icon,
.arco-select-view-clear-icon {
  color: var(--text-primary) !important;
}

.arco-select-view input::placeholder,
.arco-picker input::placeholder {
  color: var(--text-secondary) !important;
  opacity: 0.82 !important;
}

[class*='guidInputCard'] textarea,
[class*='guidContainer'] [class*='guidInputCard'] textarea,
.relative.p-16px.border-3.b.bg-dialog-fill-0.b-solid.rd-20px.flex.flex-col:has(.sendbox-tools) textarea,
.sendbox-input--mobile {
  background: transparent !important;
  color: var(--text-primary) !important;
  caret-color: var(--color-primary);
}

.relative.p-16px.border-3.b.bg-dialog-fill-0.b-solid.rd-20px.flex.flex-col:has(.sendbox-tools),
[class*='guidInputCard'],
.guidContainer .guidInputCard {
  position: relative;
  background: color-mix(in srgb, var(--horizon-surface) 94%, var(--aou-1)) !important;
  border: 1px solid var(--border-base) !important;
  border-radius: 20px !important;
  box-shadow: var(--horizon-shadow-soft) !important;
}

.relative.p-16px.border-3.b.bg-dialog-fill-0.b-solid.rd-20px.flex.flex-col:has(.sendbox-tools):focus-within,
[class*='guidInputCard']:focus-within,
.guidContainer .guidInputCard:focus-within {
  overflow: visible !important;
  border: 1px solid transparent !important;
  background-color: var(--dialog-fill-0) !important;
  background-image:
    linear-gradient(var(--dialog-fill-0), var(--dialog-fill-0)), var(--horizon-aurora-input-gradient) !important;
  background-size:
    100% 100%,
    220% 100% !important;
  background-repeat: no-repeat, no-repeat !important;
  background-position:
    center center,
    0% 50% !important;
  background-origin: border-box !important;
  background-clip: padding-box, border-box !important;
  box-shadow:
    0 0 0 1px var(--horizon-aurora-input-ring),
    0 0 16px rgba(255, 106, 1, 0.1),
    0 0 18px rgba(138, 43, 226, 0.1),
    0 0 20px rgba(0, 191, 255, 0.12),
    var(--horizon-aurora-input-shadow) !important;
  animation:
    horizonAuroraFlow 2.8s linear infinite,
    horizonAuroraGlow 3.2s ease-in-out infinite;
}

[class*='searchInput'] {
  box-shadow: none !important;
}

[class*='searchInput']:focus-within,
[class*='searchInput']:hover {
  box-shadow: none !important;
}

[class*='searchInput'] .arco-input-group,
[class*='searchInput'] .arco-input,
[class*='searchInput'] .arco-input-group-prefix {
  background: transparent !important;
  box-shadow: none !important;
}

[class*='searchInput'] .arco-input-inner-wrapper,
[class*='searchInput'] .arco-input-inner-wrapper:hover,
[class*='searchInput'] .arco-input-inner-wrapper:focus-within,
[class*='searchInput'] .arco-input-inner-wrapper.arco-input-inner-wrapper-focus {
  border: none !important;
  border-radius: 0 !important;
  background: transparent !important;
  background-image: none !important;
  box-shadow: none !important;
  animation: none !important;
}

.sendbox-tools .arco-btn,
.sendbox-tools .arco-btn:hover,
.sendbox-tools .arco-btn:active,
.sendbox-tools .arco-btn:focus-visible {
  transform: none !important;
  box-shadow: none !important;
}

.sendbox-model-btn,
.header-model-btn,
.agent-mode-compact-pill,
.guidContainer .sendbox-model-btn.guid-config-btn {
  border: 1px solid var(--border-base) !important;
  background: color-mix(in srgb, var(--horizon-surface) 92%, var(--aou-1)) !important;
  color: var(--text-primary) !important;
}

.sendbox-model-btn:hover,
.sendbox-model-btn:focus-visible,
.header-model-btn:hover,
.header-model-btn:focus-visible,
.agent-mode-compact-pill:hover,
.agent-mode-compact-pill:focus-visible,
.guidContainer .sendbox-model-btn.guid-config-btn:hover,
.guidContainer .sendbox-model-btn.guid-config-btn:focus-visible {
  border-color: var(--color-primary) !important;
  color: var(--color-primary) !important;
  box-shadow: 0 0 0 4px color-mix(in srgb, var(--horizon-focus-ring) 82%, transparent) !important;
}

[data-theme='dark'] .sendbox-model-btn,
[data-theme='dark'] .header-model-btn,
[data-theme='dark'] .agent-mode-compact-pill,
[data-theme='dark'] .guidContainer .sendbox-model-btn.guid-config-btn {
  background: color-mix(in srgb, var(--horizon-surface-soft) 92%, var(--aou-2)) !important;
}

.guidContainer .actionTools .arco-btn.arco-btn-text {
  border-radius: 999px !important;
  background: color-mix(in srgb, var(--horizon-surface) 90%, var(--aou-1)) !important;
}

[data-theme='dark'] .guidContainer .actionTools .arco-btn.arco-btn-text {
  background: color-mix(in srgb, var(--horizon-surface-soft) 90%, var(--aou-2)) !important;
}

.guidContainer [data-agent-pill='true'][data-agent-selected='true'] {
  box-shadow:
    inset 0 0 0 1px rgba(var(--primary-rgb), 0.22),
    0 6px 18px -14px rgba(var(--primary-rgb), 0.32) !important;
}

[data-theme='dark'] .guidContainer [data-agent-pill='true'][data-agent-selected='true'] {
  box-shadow:
    inset 0 0 0 1px rgba(123, 127, 240, 0.28),
    0 10px 20px -16px rgba(0, 0, 0, 0.52) !important;
}

.guidContainer .arco-menu-item.arco-menu-selected,
.guidContainer .arco-menu-light .arco-menu-selected {
  background: color-mix(in srgb, var(--horizon-selected) 72%, var(--horizon-surface)) !important;
  color: var(--color-primary) !important;
}

[data-theme='dark'] .guidContainer .arco-menu-item.arco-menu-selected,
[data-theme='dark'] .guidContainer .arco-menu-dark .arco-menu-selected {
  background: color-mix(in srgb, var(--horizon-selected) 76%, var(--horizon-surface-soft)) !important;
  color: #dce0ff !important;
}

.arco-dropdown-menu-item,
.arco-select-option,
.arco-cascader-option,
.arco-menu-item,
.arco-menu-inline-header,
.arco-menu-pop-header {
  border-radius: 10px !important;
  color: var(--text-primary) !important;
  transition:
    background-color 0.16s ease,
    color 0.16s ease !important;
}

.arco-dropdown-menu {
  padding: 0 !important;
  border-radius: 14px !important;
  max-height: min(40vh, 220px) !important;
  overflow-x: hidden !important;
  overflow-y: auto !important;
  overscroll-behavior: contain !important;
  -webkit-overflow-scrolling: touch;
}

.arco-select-popup .arco-select-popup-inner {
  border-radius: inherit !important;
  max-height: min(40vh, 220px) !important;
  overflow-x: hidden !important;
  overflow-y: auto !important;
  overscroll-behavior: contain !important;
  -webkit-overflow-scrolling: touch;
}

.arco-dropdown-menu-item {
  border-radius: 0 !important;
}

.arco-dropdown-menu-item:first-child {
  border-top-left-radius: inherit !important;
  border-top-right-radius: inherit !important;
}

.arco-dropdown-menu-item:last-child {
  border-bottom-left-radius: inherit !important;
  border-bottom-right-radius: inherit !important;
}

.arco-dropdown-menu-item:hover,
.arco-select-option:not(.arco-select-option-disabled):hover,
.arco-cascader-option:hover,
.arco-menu-item:hover,
.arco-menu-inline-header:hover,
.arco-menu-pop-header:hover {
  background: color-mix(in srgb, var(--horizon-selected) 62%, var(--horizon-surface)) !important;
  color: var(--color-primary) !important;
}

.arco-select-option-active,
.arco-dropdown-menu-item-active,
.arco-menu-selected,
.arco-menu-item.arco-menu-selected {
  background: color-mix(in srgb, var(--horizon-selected) 78%, var(--horizon-surface)) !important;
  color: var(--color-primary) !important;
}

[data-theme='dark'] .arco-dropdown-menu-item:hover,
[data-theme='dark'] .arco-select-option:not(.arco-select-option-disabled):hover,
[data-theme='dark'] .arco-cascader-option:hover,
[data-theme='dark'] .arco-menu-item:hover,
[data-theme='dark'] .arco-menu-inline-header:hover,
[data-theme='dark'] .arco-menu-pop-header:hover,
[data-theme='dark'] .arco-select-option-active,
[data-theme='dark'] .arco-dropdown-menu-item-active,
[data-theme='dark'] .arco-menu-selected,
[data-theme='dark'] .arco-menu-item.arco-menu-selected {
  background: color-mix(in srgb, var(--horizon-selected) 82%, var(--horizon-surface-soft)) !important;
  color: #dce0ff !important;
}

.arco-tabs-nav {
  border-bottom: 1px solid color-mix(in srgb, var(--border-base) 86%, transparent) !important;
}

.arco-tabs-nav-tab {
  color: var(--text-secondary) !important;
  border-radius: 999px !important;
  padding: 8px 14px !important;
  margin-inline: 4px !important;
  transition:
    background-color 0.16s ease,
    color 0.16s ease !important;
}

.arco-tabs-nav-tab:hover {
  background: color-mix(in srgb, var(--horizon-selected) 45%, var(--horizon-surface)) !important;
  color: var(--color-primary) !important;
}

.arco-tabs-nav-tab-active {
  color: var(--color-primary) !important;
  font-weight: 700 !important;
}

.arco-tabs-nav-ink {
  height: 2px !important;
  background: var(--color-primary) !important;
}

.arco-collapse-item-header {
  border-radius: 14px !important;
  color: var(--text-primary) !important;
  transition:
    background-color 0.16s ease,
    color 0.16s ease !important;
}

.arco-collapse-item-header:hover {
  background: color-mix(in srgb, var(--horizon-selected) 40%, var(--horizon-surface)) !important;
}

.arco-collapse-item-content {
  border-top-color: color-mix(in srgb, var(--border-base) 86%, transparent) !important;
}

.arco-collapse-item-content-box {
  color: var(--text-primary) !important;
}

.arco-tree-node-title-wrapper {
  border-radius: 10px !important;
  transition:
    background-color 0.16s ease,
    color 0.16s ease,
    box-shadow 0.16s ease !important;
}

.workspace-tree .arco-tree-node:hover .arco-tree-node-title,
.arco-tree-node:hover > .arco-tree-node-title-wrapper {
  background: color-mix(in srgb, var(--horizon-selected) 42%, var(--horizon-surface)) !important;
}

.arco-tree-node-selected > .arco-tree-node-title-wrapper {
  background: color-mix(in srgb, var(--horizon-selected) 72%, var(--horizon-surface)) !important;
  color: var(--color-primary) !important;
  box-shadow: inset 0 0 0 1px rgba(var(--primary-rgb), 0.14) !important;
}

[data-theme='dark'] .workspace-tree .arco-tree-node:hover .arco-tree-node-title,
[data-theme='dark'] .arco-tree-node:hover > .arco-tree-node-title-wrapper,
[data-theme='dark'] .arco-tree-node-selected > .arco-tree-node-title-wrapper {
  background: color-mix(in srgb, var(--horizon-selected) 76%, var(--horizon-surface-soft)) !important;
}

.arco-switch {
  background-color: color-mix(in srgb, var(--border-base) 78%, var(--bg-3)) !important;
}

.arco-switch-checked {
  background-color: var(--color-primary) !important;
}

.arco-switch-handle {
  box-shadow: 0 2px 6px rgba(15, 23, 42, 0.18) !important;
}

.arco-checkbox-icon,
.arco-radio-mask {
  border-color: var(--border-base) !important;
  background: color-mix(in srgb, var(--horizon-surface) 92%, var(--aou-1)) !important;
}

.arco-checkbox-checked .arco-checkbox-icon,
.arco-radio-checked .arco-radio-mask {
  border-color: var(--color-primary) !important;
  background: var(--color-primary) !important;
}

.arco-radio-checked .arco-radio-mask::after {
  background: #ffffff !important;
}

.message-item.user .message-bubble {
  background: linear-gradient(135deg, #eef0ff 0%, #e7ebff 100%) !important;
  border: 1px solid #d7dfff !important;
  border-radius: 20px 20px 8px 20px !important;
  box-shadow: 0 16px 28px -26px rgba(89, 91, 202, 0.32) !important;
}

.message-item.ai .message-bubble,
.message-item.assistant .message-bubble {
  background: color-mix(in srgb, var(--horizon-surface) 96%, var(--bg-1)) !important;
  border: 1px solid var(--border-base) !important;
  border-radius: 20px 20px 20px 8px !important;
  box-shadow: 0 10px 24px -22px rgba(15, 23, 42, 0.18) !important;
}

[data-theme='dark'] .message-item.user .message-bubble {
  background: linear-gradient(135deg, #313557 0%, #2a2d4a 100%) !important;
  border-color: #4a5070 !important;
  box-shadow: 0 18px 30px -26px rgba(0, 0, 0, 0.42) !important;
}

[data-theme='dark'] .message-item.ai .message-bubble,
[data-theme='dark'] .message-item.assistant .message-bubble {
  background: linear-gradient(180deg, rgba(38, 40, 52, 0.98) 0%, rgba(32, 33, 37, 1) 100%) !important;
  box-shadow: 0 16px 28px -24px rgba(0, 0, 0, 0.42) !important;
}

.message-item.ai .arco-alert,
.message-item.ai [class*='alert'] {
  background: color-mix(in srgb, var(--horizon-surface-soft) 88%, var(--aou-1)) !important;
  border: 1px solid var(--border-base) !important;
  border-radius: 12px !important;
}

.message-item.ai .arco-card,
.message-item.ai [class*='card'],
.message-item.ai [class*='status']:not([class*='message']):not([class*='bubble']) {
  background: linear-gradient(180deg, var(--horizon-surface) 0%, var(--horizon-surface-soft) 100%) !important;
  border: 1px solid var(--border-base) !important;
  border-radius: 16px !important;
  box-shadow: none !important;
}

[data-theme='dark'] .message-item.ai .arco-alert,
[data-theme='dark'] .message-item.ai [class*='alert'] {
  background: color-mix(in srgb, var(--horizon-surface-soft) 86%, var(--aou-2)) !important;
}

[data-theme='dark'] .message-item.ai .arco-card,
[data-theme='dark'] .message-item.ai [class*='card'],
[data-theme='dark'] .message-item.ai [class*='status']:not([class*='message']):not([class*='bubble']) {
  background: linear-gradient(180deg, var(--horizon-surface-soft) 0%, var(--horizon-surface) 100%) !important;
}

.message-content,
.markdown-body,
.markdown-shadow-body,
[class*='markdown'] {
  line-height: 1.74;
}

.message-content h1,
.markdown-body h1,
.markdown-shadow-body h1,
[class*='markdown'] h1 {
  color: var(--text-primary);
  letter-spacing: -0.02em;
}

.message-content h2,
.markdown-body h2,
.markdown-shadow-body h2,
[class*='markdown'] h2 {
  color: var(--color-primary);
  letter-spacing: -0.015em;
}

.message-content h3,
.markdown-body h3,
.markdown-shadow-body h3,
[class*='markdown'] h3 {
  color: var(--brand);
}

.message-content h4,
.markdown-body h4,
.markdown-shadow-body h4,
[class*='markdown'] h4 {
  color: var(--success);
}

.message-content h5,
.markdown-body h5,
.markdown-shadow-body h5,
[class*='markdown'] h5 {
  color: var(--text-secondary);
}

.message-content a,
.markdown-body a,
.markdown-shadow-body a,
[class*='markdown'] a {
  color: var(--color-primary);
  -webkit-text-fill-color: var(--color-primary);
  text-decoration-color: rgba(var(--primary-rgb), 0.38);
  text-underline-offset: 2px;
}

.message-content a:hover,
.markdown-body a:hover,
.markdown-shadow-body a:hover,
[class*='markdown'] a:hover {
  color: var(--color-primary-dark-1) !important;
  -webkit-text-fill-color: var(--color-primary-dark-1) !important;
  text-decoration-color: rgba(var(--primary-rgb), 0.68) !important;
}

[data-theme='dark'] .message-content a,
[data-theme='dark'] .markdown-body a,
[data-theme='dark'] .markdown-shadow-body a,
[data-theme='dark'] [class*='markdown'] a {
  color: #cfd3ff !important;
  -webkit-text-fill-color: #cfd3ff !important;
}

[data-theme='dark'] .message-content a:hover,
[data-theme='dark'] .markdown-body a:hover,
[data-theme='dark'] .markdown-shadow-body a:hover,
[data-theme='dark'] [class*='markdown'] a:hover {
  color: #eef0ff !important;
  -webkit-text-fill-color: #eef0ff !important;
}

[data-theme='light'] .message-content a:visited,
[data-theme='light'] .markdown-body a:visited,
[data-theme='light'] .markdown-shadow-body a:visited,
[data-theme='light'] [class*='markdown'] a:visited {
  color: #7c72b8 !important;
  -webkit-text-fill-color: #7c72b8 !important;
  text-decoration-color: rgba(124, 114, 184, 0.6) !important;
}

[data-theme='dark'] .message-content a:visited,
[data-theme='dark'] .markdown-body a:visited,
[data-theme='dark'] .markdown-shadow-body a:visited,
[data-theme='dark'] [class*='markdown'] a:visited {
  color: #d7cfff !important;
  -webkit-text-fill-color: #d7cfff !important;
  text-decoration-color: rgba(215, 207, 255, 0.72) !important;
}

.message-content pre,
.markdown-body pre,
.markdown-shadow-body pre,
[class*='markdown'] pre {
  background: linear-gradient(180deg, var(--bg-2) 0%, var(--bg-1) 100%) !important;
  border: 1px solid var(--border-base) !important;
  border-radius: 18px !important;
  box-shadow: 0 20px 32px -28px rgba(15, 23, 42, 0.22) !important;
}

[data-theme='dark'] .message-content pre,
[data-theme='dark'] .markdown-body pre,
[data-theme='dark'] .markdown-shadow-body pre,
[data-theme='dark'] [class*='markdown'] pre {
  background: linear-gradient(180deg, #262834 0%, #202125 100%) !important;
  box-shadow: 0 20px 32px -28px rgba(0, 0, 0, 0.56) !important;
}

.message-content pre:has(> div),
.markdown-body pre:has(> div),
.markdown-shadow-body pre:has(> div),
[class*='markdown'] pre:has(> div) {
  background: transparent !important;
  border: none !important;
  border-radius: 0 !important;
  box-shadow: none !important;
  padding: 0 !important;
  overflow: visible !important;
}

.message-content code:not(pre code),
.markdown-body code:not(pre code),
.markdown-shadow-body code:not(pre code),
[class*='markdown'] code:not(pre code) {
  background: var(--hl-chip-bg) !important;
  color: var(--hl-chip-text) !important;
  border: 1px solid var(--hl-chip-border) !important;
  border-radius: 999px !important;
  padding: 2px 9px !important;
  font-size: 0.88em !important;
  font-weight: 600 !important;
  -webkit-text-fill-color: var(--hl-chip-text) !important;
}

.message-content strong,
.markdown-body strong,
.markdown-shadow-body strong,
[class*='markdown'] strong,
.message-content mark,
.markdown-body mark,
.markdown-shadow-body mark,
[class*='markdown'] mark {
  background: var(--hl-chip-bg) !important;
  color: var(--hl-chip-text) !important;
  border: 1px solid var(--hl-chip-border) !important;
  border-radius: 7px !important;
  padding: 0 6px !important;
  box-decoration-break: clone;
  -webkit-box-decoration-break: clone;
  text-shadow: none !important;
  -webkit-text-fill-color: var(--hl-chip-text) !important;
}

.message-content strong *,
.markdown-body strong *,
.markdown-shadow-body strong *,
[class*='markdown'] strong *,
.message-content mark *,
.markdown-body mark *,
.markdown-shadow-body mark *,
[class*='markdown'] mark *,
.message-content code:not(pre code) *,
.markdown-body code:not(pre code) *,
.markdown-shadow-body code:not(pre code) *,
[class*='markdown'] code:not(pre code) * {
  color: var(--hl-chip-text) !important;
  -webkit-text-fill-color: var(--hl-chip-text) !important;
}

.message-content blockquote,
.markdown-body blockquote,
.markdown-shadow-body blockquote,
[class*='markdown'] blockquote {
  margin-inline: 0 !important;
  padding: 10px 16px !important;
  border-left: 3px solid var(--color-primary) !important;
  background: color-mix(in srgb, var(--bg-base) 86%, var(--aou-1)) !important;
  border-radius: 0 16px 16px 0 !important;
}

[data-theme='dark'] .message-content blockquote,
[data-theme='dark'] .markdown-body blockquote,
[data-theme='dark'] .markdown-shadow-body blockquote,
[data-theme='dark'] [class*='markdown'] blockquote {
  background: color-mix(in srgb, var(--bg-2) 88%, var(--aou-2)) !important;
}

.message-content table,
.markdown-body table,
.markdown-shadow-body table,
[class*='markdown'] table {
  width: 100%;
  border-collapse: separate !important;
  border-spacing: 0 !important;
  overflow: hidden;
  border: 1px solid var(--border-base) !important;
  border-radius: 16px !important;
  box-shadow: var(--horizon-shadow-soft) !important;
  background: var(--horizon-surface) !important;
}

.message-content thead,
.markdown-body thead,
.markdown-shadow-body thead,
[class*='markdown'] thead {
  background: color-mix(in srgb, var(--horizon-selected) 58%, var(--horizon-surface)) !important;
}

.message-content th,
.message-content td,
.markdown-body th,
.markdown-body td,
.markdown-shadow-body th,
.markdown-shadow-body td,
[class*='markdown'] th,
[class*='markdown'] td {
  border-bottom: 1px solid color-mix(in srgb, var(--border-base) 82%, transparent) !important;
  padding: 10px 12px !important;
  text-align: left;
}

.message-content tr:last-child td,
.markdown-body tr:last-child td,
.markdown-shadow-body tr:last-child td,
[class*='markdown'] tr:last-child td {
  border-bottom: none !important;
}

.rd-20px.text-14px.pb-40px.lh-20px {
  border: 1px solid var(--border-base) !important;
  border-left: 3px solid var(--color-primary) !important;
  border-radius: 16px !important;
}

.arco-modal-header {
  background: linear-gradient(
    180deg,
    color-mix(in srgb, var(--horizon-selected) 58%, var(--horizon-surface)) 0%,
    var(--horizon-surface) 100%
  ) !important;
  border-bottom: 1px solid color-mix(in srgb, var(--border-base) 82%, transparent) !important;
  color: var(--text-primary) !important;
}

.arco-modal-body {
  background: linear-gradient(180deg, var(--horizon-surface) 0%, var(--horizon-surface-soft) 100%) !important;
  color: var(--text-primary) !important;
}

.arco-modal-footer {
  background: linear-gradient(180deg, var(--horizon-surface-soft) 0%, var(--horizon-surface) 100%) !important;
  border-top: 1px solid color-mix(in srgb, var(--border-base) 82%, transparent) !important;
}

.roseui-modal:not(.conversation-search-modal) .arco-modal-content {
  overflow: visible !important;
  padding-bottom: 12px !important;
  box-sizing: border-box !important;
}

.roseui-modal:has(.cm-editor),
.roseui-modal:has(.arco-steps) {
  height: auto !important;
  min-height: min(560px, calc(100vh - 32px)) !important;
  max-height: calc(100vh - 32px) !important;
}

.roseui-modal-wrapper,
.roseui-modal-wrapper .roseui-modal-title,
.roseui-modal-wrapper .roseui-modal-body-content,
.roseui-modal-wrapper .arco-btn,
.roseui-modal-wrapper .arco-input,
.roseui-modal-wrapper .arco-textarea,
.roseui-modal-wrapper .arco-alert,
.roseui-modal-wrapper .arco-alert-content {
  font-family: 'Plus Jakarta Sans', 'Segoe UI Variable', 'Segoe UI', sans-serif !important;
}

.roseui-modal-wrapper .cm-theme-light,
.roseui-modal-wrapper .cm-editor,
.roseui-modal-wrapper .cm-scroller,
.roseui-modal-wrapper .cm-content,
.roseui-modal-wrapper .cm-line,
.roseui-modal-wrapper .cm-gutters {
  font-family:
    ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace !important;
}

.roseui-modal-wrapper .arco-btn {
  border-radius: 8px !important;
}

.roseui-modal-wrapper .roseui-modal-body-content .cm-theme-light,
.roseui-modal-wrapper .roseui-modal-body-content .cm-editor {
  box-shadow: none !important;
}

.roseui-modal-wrapper:has(.cm-editor),
.roseui-modal-wrapper:has(.arco-steps) {
  display: flex;
  flex-direction: column;
  min-height: 0;
  height: 100% !important;
}

.roseui-modal-wrapper:has(.cm-editor) .roseui-modal-body-content,
.roseui-modal-wrapper:has(.arco-steps) .roseui-modal-body-content {
  height: auto !important;
  min-height: 360px;
  padding-bottom: 16px !important;
  overflow: auto !important;
}

.roseui-modal-wrapper:has(.cm-editor) > .flex-shrink-0.bg-transparent,
.roseui-modal-wrapper:has(.arco-steps) > .flex-shrink-0.bg-transparent {
  margin-top: auto;
  padding-top: 12px;
  padding-bottom: 12px;
}

.roseui-modal-wrapper:has(.arco-steps) .roseui-modal-body-content > .flex.flex-col {
  height: auto !important;
  min-height: 320px;
}

.roseui-modal-wrapper:has(.arco-steps) .roseui-modal-body-content > .flex.flex-col > .mb-6.flex-1 {
  min-height: 220px !important;
  padding-bottom: 8px;
}

.arco-table-container,
.arco-table,
.arco-table th,
.arco-table td {
  background: transparent !important;
}

.arco-table {
  border: 1px solid var(--border-base) !important;
  border-radius: 16px !important;
  overflow: hidden !important;
}

.arco-table-th {
  background: color-mix(in srgb, var(--horizon-selected) 56%, var(--horizon-surface)) !important;
  color: var(--text-primary) !important;
  border-bottom: 1px solid color-mix(in srgb, var(--border-base) 82%, transparent) !important;
}

.arco-table-td {
  border-bottom: 1px solid color-mix(in srgb, var(--border-base) 80%, transparent) !important;
}

.arco-table-tr:hover .arco-table-td {
  background: color-mix(in srgb, var(--horizon-selected) 28%, var(--horizon-surface)) !important;
}

.arco-alert {
  border-radius: 14px !important;
}

.arco-alert-info {
  background: color-mix(in srgb, var(--horizon-selected) 54%, var(--horizon-surface)) !important;
  border: 1px solid color-mix(in srgb, var(--color-primary) 24%, var(--border-base)) !important;
}

.arco-alert-warning {
  background: color-mix(in srgb, var(--warning) 16%, var(--horizon-surface)) !important;
  border: 1px solid color-mix(in srgb, var(--warning) 32%, var(--border-base)) !important;
}

.arco-alert-error {
  background: color-mix(in srgb, var(--danger) 15%, var(--horizon-surface)) !important;
  border: 1px solid color-mix(in srgb, var(--danger) 32%, var(--border-base)) !important;
}

.arco-alert-success {
  background: color-mix(in srgb, var(--success) 15%, var(--horizon-surface)) !important;
  border: 1px solid color-mix(in srgb, var(--success) 32%, var(--border-base)) !important;
}

.arco-tag,
.rd-20px.text-14px.pb-40px .arco-tag {
  background: color-mix(in srgb, var(--bg-base) 84%, var(--aou-1)) !important;
  border: 1px solid var(--aou-3) !important;
  border-radius: 999px !important;
  color: var(--brand) !important;
  font-weight: 600 !important;
}

.arco-divider {
  border-color: color-mix(in srgb, var(--border-base) 86%, transparent) !important;
}

.aion-file-changes-panel > div:first-child {
  box-shadow: inset 0 -1px 0 color-mix(in srgb, var(--border-base) 82%, transparent);
}

::-webkit-scrollbar {
  width: 7px;
  height: 7px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: linear-gradient(180deg, #b7bdf0 0%, #7c84d2 100%);
  border-radius: 999px;
}

::-webkit-scrollbar-thumb:hover {
  background: linear-gradient(180deg, #9ca5e3 0%, #595bca 100%);
}

[data-theme='dark'] ::-webkit-scrollbar-thumb {
  background: linear-gradient(180deg, #5f658b 0%, #3b3e56 100%);
}

[data-theme='dark'] ::-webkit-scrollbar-thumb:hover {
  background: linear-gradient(180deg, #7b81aa 0%, #50567b 100%);
}

::selection {
  background: rgba(89, 91, 202, 0.18);
}

[data-theme='dark'] ::selection {
  background: rgba(123, 127, 240, 0.24);
}

@keyframes horizonAuroraFlow {
  0% {
    background-position:
      center center,
      0% 50%;
  }

  100% {
    background-position:
      center center,
      220% 50%;
  }
}

@keyframes horizonAuroraGlow {
  0%,
  100% {
    box-shadow:
      0 0 0 1px var(--horizon-aurora-input-ring),
      0 0 16px rgba(255, 106, 1, 0.1),
      0 0 18px rgba(138, 43, 226, 0.1),
      0 0 20px rgba(0, 191, 255, 0.12),
      var(--horizon-aurora-input-shadow);
  }

  50% {
    box-shadow:
      0 0 0 1px color-mix(in srgb, #f8c91c 30%, transparent),
      0 0 18px rgba(255, 106, 1, 0.14),
      0 0 22px rgba(138, 43, 226, 0.14),
      0 0 24px rgba(0, 191, 255, 0.18),
      var(--horizon-aurora-input-shadow-strong);
  }
}

/* RoseUi Theme Background Start */
/* Preview cover only: do not auto-inject full-page background image */
/* RoseUi Theme Background End */
`,be=`/* Glittering Input Field Theme - RoseUi Adaptation */
/* Warm serif surfaces with a vivid aurora input accent */

:root {
  /* ===== Primary: Retroma Teal-Blue ===== */
  --color-primary: #0f7887;
  --primary: #0f7887;
  --color-primary-light-1: #2a9aac;
  --color-primary-light-2: #4db8c8;
  --color-primary-light-3: #80d0dc;
  --color-primary-dark-1: #0a5a66;
  --primary-rgb: 15, 120, 135;

  /* ===== Brand: Retroma Warm Purple ===== */
  --brand: #6e3a66;
  --brand-light: #f2e8f0;
  --brand-hover: #9d6094;
  --color-brand-fill: #6e3a66;
  --color-brand-bg: #f2e8f0;

  /* ===== AOU Palette: Warm Olive-Green Gradient ===== */
  --aou-1: #f4f4ec;
  --aou-2: #e8e9d8;
  --aou-3: #d0d2b0;
  --aou-4: #b4b888;
  --aou-5: #979d62;
  --aou-6: #737f16;
  --aou-7: #575f10;
  --aou-8: #3c420b;
  --aou-9: #222606;
  --aou-10: #0c0e02;

  /* ===== Backgrounds: Warm Parchment ===== */
  --bg-base: #faf9f6;
  --bg-1: #f5f4ef;
  --bg-2: #eeede5;
  --bg-3: #e2e0d4;
  --bg-4: #cbc8b8;
  --bg-5: #b0ac9a;
  --bg-6: #8c8878;
  --bg-8: #575450;
  --bg-9: #2c2b28;
  --bg-10: #111009;
  --color-bg-1: #f5f4ef;
  --color-bg-2: #eeede5;
  --color-bg-3: #e2e0d4;
  --color-bg-4: #cbc8b8;

  /* ===== Interactive States ===== */
  --bg-hover: #ebe9df;
  --bg-active: #e0ded4;

  /* ===== Text: Warm Dark ===== */
  --text-primary: #1d011d;
  --text-secondary: #6e6060;
  --text-disabled: #b8b0a8;
  --text-0: #1d011d;
  --text-white: #faf9f6;
  --color-text-1: #1d011d;
  --color-text-2: #6e6060;
  --color-text-3: #9e9490;
  --color-text-4: #c8c0bc;

  /* ===== Borders ===== */
  --border-base: #d8d4c8;
  --border-light: #e8e6dc;
  --border-special: #d0ccc0;
  --color-border: #d8d4c8;
  --color-border-1: #d8d4c8;
  --color-border-2: #e8e6dc;

  /* ===== Fill & Inverse ===== */
  --fill: #f5f4ef;
  --color-fill: #f5f4ef;
  --fill-0: #faf9f6;
  --fill-white-to-black: #faf9f6;
  --dialog-fill-0: #faf9f6;
  --inverse: #1d011d;

  /* ===== Semantic Colors ===== */
  --success: #35847e;
  --warning: #b07a10;
  --danger: #b03030;
  --info: #0f7887;

  /* ===== Message & Component ===== */
  --message-user-bg: #e9e4f0;
  --message-tips-bg: #f1edf6;
  --workspace-btn-bg: #eeece4;

  /* ===== Color GUID Agent Bar ===== */
  --color-guid-agent-bar: #eae8de;
  --hl-chip-bg: #f3eee2;
  --hl-chip-text: #5a4a3a;
  --hl-chip-border: #d7ccb7;

  /* ===== Aurora Input Accent ===== */
  --retroma-aurora-input-gradient: linear-gradient(
    90deg,
    #ff6a01 0%,
    #f8c91c 12.5%,
    #8a2be2 25%,
    #00bfff 37.5%,
    #ff6a01 50%,
    #f8c91c 62.5%,
    #8a2be2 75%,
    #00bfff 87.5%,
    #ff6a01 100%
  );
  --retroma-aurora-input-ring: rgba(255, 162, 84, 0.24);
  --retroma-aurora-input-shadow: 0 18px 40px rgba(110, 58, 102, 0.16), 0 0 28px rgba(15, 120, 135, 0.1);
  --retroma-aurora-input-shadow-strong: 0 22px 48px rgba(110, 58, 102, 0.2), 0 0 34px rgba(0, 191, 255, 0.18);
  --retroma-aurora-placeholder: #8f7c7b;
}

/* ===== Dark Mode Overrides ===== */
[data-theme='dark'] {
  /* Primary: Retroma Teal (dark-adjusted) */
  --color-primary: #6e8ddb;
  --primary: #6e8ddb;
  --color-primary-light-1: #8fa8e8;
  --color-primary-light-2: #aabff0;
  --color-primary-light-3: #c5d5f6;
  --color-primary-dark-1: #4f70c4;
  --primary-rgb: 110, 141, 219;

  /* Brand: Retroma Purple (dark) */
  --brand: #be80bf;
  --brand-light: #3d2840;
  --brand-hover: #9a60a0;
  --color-brand-fill: #be80bf;
  --color-brand-bg: #3d2840;

  /* AOU Palette: Dark Olive */
  --aou-1: #232318;
  --aou-2: #363525;
  --aou-3: #4a4a30;
  --aou-4: #666640;
  --aou-5: #898a54;
  --aou-6: #a2a554;
  --aou-7: #bbbf6e;
  --aou-8: #d0d490;
  --aou-9: #e4e6b8;
  --aou-10: #f2f4da;

  /* Backgrounds: Dark Obsidian with warm undertones */
  --bg-base: #0f0f0c;
  --bg-1: #18180f;
  --bg-2: #222217;
  --bg-3: #2e2e20;
  --bg-4: #3c3c2c;
  --bg-5: #4e4e3a;
  --bg-6: #606050;
  --bg-8: #848470;
  --bg-9: #b0b09a;
  --bg-10: #d8d8c8;
  --color-bg-1: #18180f;
  --color-bg-2: #222217;
  --color-bg-3: #2e2e20;
  --color-bg-4: #3c3c2c;

  /* Interactive States */
  --bg-hover: #1e1e14;
  --bg-active: #28281c;

  /* Text: Retroma warm near-white */
  --text-primary: #e8e6d8;
  --text-secondary: #a8a498;
  --text-disabled: #686458;
  --text-0: #f0ede0;
  --text-white: #f0ede0;
  --color-text-1: #e8e6d8;
  --color-text-2: #a8a498;
  --color-text-3: #787468;
  --color-text-4: #504c44;

  /* Borders */
  --border-base: #3a3a28;
  --border-light: #2a2a1e;
  --border-special: #4a4a36;
  --color-border: #3a3a28;
  --color-border-1: #3a3a28;
  --color-border-2: #2a2a1e;

  /* Fill & Inverse */
  --fill: #18180f;
  --color-fill: #18180f;
  --fill-0: rgba(255, 252, 240, 0.07);
  --fill-white-to-black: #0f0f0c;
  --dialog-fill-0: #2e2e20;
  --inverse: #f0ede0;

  /* Semantic Colors */
  --success: #68a99d;
  --warning: #d4963a;
  --danger: #c86060;
  --info: #6e8ddb;

  /* Message & Component */
  --message-user-bg: #3a3444;
  --message-tips-bg: #2f2a38;
  --workspace-btn-bg: #1e1e14;

  /* Color GUID Agent Bar */
  --color-guid-agent-bar: #2a2a1e;
  --hl-chip-bg: #d6ccb8;
  --hl-chip-text: #3f3528;
  --hl-chip-border: #b5a88f;

  /* Aurora Input Accent */
  --retroma-aurora-input-ring: rgba(166, 214, 255, 0.22);
  --retroma-aurora-input-shadow: 0 22px 48px rgba(6, 10, 18, 0.5), 0 0 32px rgba(138, 43, 226, 0.16);
  --retroma-aurora-input-shadow-strong: 0 26px 54px rgba(4, 8, 16, 0.62), 0 0 38px rgba(0, 191, 255, 0.22);
  --retroma-aurora-placeholder: #958a83;
}

/* ===== Typography ===== */
body {
  font-family: 'Georgia', 'Palatino Linotype', 'Book Antiqua', 'Source Han Serif', serif;
  letter-spacing: 0.01em;
}

/* ===== Scrollbar Styling ===== */
::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

::-webkit-scrollbar-track {
  background: var(--bg-1);
}

::-webkit-scrollbar-thumb {
  background: var(--bg-4);
  border-radius: 3px;
}

::-webkit-scrollbar-thumb:hover {
  background: var(--brand);
}

/* ===== Message Headings (H1-H5) ===== */
.message-content h1,
.markdown-body h1 {
  color: var(--brand);
}

.message-content h2,
.markdown-body h2 {
  color: var(--color-primary);
}

.message-content h3,
.markdown-body h3 {
  color: var(--aou-6);
}

.message-content h4,
.markdown-body h4 {
  color: var(--success);
}

.message-content h5,
.markdown-body h5 {
  color: #465881;
}

[data-theme='dark'] .message-content h5,
[data-theme='dark'] .markdown-body h5 {
  color: #7a90c8;
}

/* ===== Links ===== */
.message-content a,
.markdown-body a,
.markdown-shadow-body a,
[class*='markdown'] a {
  color: #5b63cf;
  -webkit-text-fill-color: #5b63cf;
  text-decoration-color: rgba(91, 99, 207, 0.55);
  text-underline-offset: 2px;
}

[data-theme='light'] .message-content a:hover,
[data-theme='light'] .markdown-body a:hover,
[data-theme='light'] .markdown-shadow-body a:hover,
[data-theme='light'] [class*='markdown'] a:hover {
  color: #464fc0 !important;
  -webkit-text-fill-color: #464fc0 !important;
  text-decoration-color: rgba(70, 79, 192, 0.8) !important;
}

[data-theme='light'] .message-content a:visited,
[data-theme='light'] .markdown-body a:visited,
[data-theme='light'] .markdown-shadow-body a:visited,
[data-theme='light'] [class*='markdown'] a:visited {
  color: #7a63b0 !important;
  -webkit-text-fill-color: #7a63b0 !important;
  text-decoration-color: rgba(122, 99, 176, 0.65) !important;
}

[data-theme='dark'] .message-content a,
[data-theme='dark'] .markdown-body a,
[data-theme='dark'] .markdown-shadow-body a,
[data-theme='dark'] [class*='markdown'] a {
  color: #cfe0ff !important;
  -webkit-text-fill-color: #cfe0ff !important;
  text-decoration-color: rgba(207, 224, 255, 0.85) !important;
}

[data-theme='dark'] .message-content a:hover,
[data-theme='dark'] .markdown-body a:hover,
[data-theme='dark'] .markdown-shadow-body a:hover,
[data-theme='dark'] [class*='markdown'] a:hover {
  color: #e7efff !important;
  -webkit-text-fill-color: #e7efff !important;
  text-decoration-color: rgba(231, 239, 255, 0.98) !important;
}

[data-theme='dark'] .message-content a:visited,
[data-theme='dark'] .markdown-body a:visited,
[data-theme='dark'] .markdown-shadow-body a:visited,
[data-theme='dark'] [class*='markdown'] a:visited {
  color: #d4c6fa !important;
  -webkit-text-fill-color: #d4c6fa !important;
  text-decoration-color: rgba(212, 198, 250, 0.82) !important;
}

/* ===== Code Blocks ===== */
.message-content pre,
.markdown-body pre {
  background: var(--bg-2);
  border: 1px solid var(--border-base);
  border-radius: 6px;
}

[data-theme='dark'] .message-content pre,
[data-theme='dark'] .markdown-body pre {
  background: #141410;
  border-color: var(--border-base);
}

/* ===== Inline Code / Highlight Chip ===== */
.message-content code:not(pre code),
.markdown-body code:not(pre code),
.markdown-shadow-body code:not(pre code),
[class*='markdown'] code:not(pre code) {
  background: var(--hl-chip-bg) !important;
  color: var(--hl-chip-text) !important;
  border: 1px solid var(--hl-chip-border) !important;
  border-radius: 7px !important;
  padding: 1px 8px !important;
  font-size: 0.9em !important;
  font-weight: 650 !important;
  opacity: 1 !important;
  filter: none !important;
  text-shadow: none !important;
  -webkit-text-fill-color: var(--hl-chip-text) !important;
  background-clip: border-box !important;
}

[data-theme='dark'] .message-content code:not(pre code),
[data-theme='dark'] .markdown-body code:not(pre code),
[data-theme='dark'] .markdown-shadow-body code:not(pre code),
[data-theme='dark'] [class*='markdown'] code:not(pre code) {
  background: var(--hl-chip-bg) !important;
  color: var(--hl-chip-text) !important;
  border-color: var(--hl-chip-border) !important;
  box-shadow: inset 0 0 0 1px rgba(90, 66, 108, 0.12) !important;
  opacity: 1 !important;
  filter: none !important;
  text-shadow: none !important;
  -webkit-text-fill-color: var(--hl-chip-text) !important;
}

/* ===== Emphasis Highlight (Bold with Background) ===== */
.message-content strong,
.markdown-body strong,
.markdown-shadow-body strong,
[class*='markdown'] strong {
  background: var(--hl-chip-bg) !important;
  color: var(--hl-chip-text) !important;
  padding: 0 6px !important;
  border-radius: 4px !important;
  border: 1px solid var(--hl-chip-border) !important;
  box-decoration-break: clone;
  -webkit-box-decoration-break: clone;
  opacity: 1 !important;
  filter: none !important;
  text-shadow: none !important;
  -webkit-text-fill-color: var(--hl-chip-text) !important;
}

[data-theme='dark'] .message-content strong,
[data-theme='dark'] .markdown-body strong,
[data-theme='dark'] .markdown-shadow-body strong,
[data-theme='dark'] [class*='markdown'] strong {
  background: var(--hl-chip-bg) !important;
  color: var(--hl-chip-text) !important;
  border-color: var(--hl-chip-border) !important;
  box-shadow: inset 0 0 0 1px rgba(90, 66, 108, 0.1) !important;
  opacity: 1 !important;
  filter: none !important;
  text-shadow: none !important;
  -webkit-text-fill-color: var(--hl-chip-text) !important;
}

[data-theme='light'] .message-content mark,
[data-theme='light'] .markdown-body mark,
[data-theme='light'] .markdown-shadow-body mark,
[data-theme='light'] [class*='markdown'] mark,
[data-theme='dark'] .message-content mark,
[data-theme='dark'] .markdown-body mark,
[data-theme='dark'] .markdown-shadow-body mark,
[data-theme='dark'] [class*='markdown'] mark {
  background: var(--hl-chip-bg) !important;
  color: var(--hl-chip-text) !important;
  border: 1px solid var(--hl-chip-border) !important;
  border-radius: 4px !important;
  padding: 0 4px !important;
  opacity: 1 !important;
  filter: none !important;
  text-shadow: none !important;
  -webkit-text-fill-color: var(--hl-chip-text) !important;
}

.message-content strong *,
.markdown-body strong *,
.markdown-shadow-body strong *,
[class*='markdown'] strong *,
.message-content code:not(pre code) *,
.markdown-body code:not(pre code) *,
.markdown-shadow-body code:not(pre code) *,
[class*='markdown'] code:not(pre code) *,
.message-content mark *,
.markdown-body mark *,
.markdown-shadow-body mark *,
[class*='markdown'] mark * {
  color: var(--hl-chip-text) !important;
  -webkit-text-fill-color: var(--hl-chip-text) !important;
  opacity: 1 !important;
}

/* ===== Sidebar ===== */
.layout-sider {
  background-color: var(--bg-1);
  border-right: 1px solid var(--border-base);
}

/* ===== Conversation Bubble (AOU purple-gray) ===== */
.message-item.user .message-bubble {
  background: var(--message-user-bg) !important;
  border: 1px solid #cbc0da !important;
}

[data-theme='dark'] .message-item.user .message-bubble {
  background: var(--message-user-bg) !important;
  border-color: color-mix(in srgb, var(--aou-5) 46%, var(--border-base)) !important;
}

/* ===== Selection Highlight ===== */
::selection {
  background: color-mix(in srgb, var(--brand) 25%, transparent);
}

[data-theme='dark'] ::selection {
  background: color-mix(in srgb, var(--brand) 30%, transparent);
}

/* ===== Aurora Inputs ===== */
.guidContainer .guidInputCard,
[class*='guidContainer'] [class*='guidInputCard'],
.relative.p-16px.border-3.b.bg-dialog-fill-0.b-solid.rd-20px.flex.flex-col:has(.sendbox-tools) {
  position: relative;
}

.guidContainer .guidInputCard:focus-within,
[class*='guidContainer'] [class*='guidInputCard']:focus-within,
.relative.p-16px.border-3.b.bg-dialog-fill-0.b-solid.rd-20px.flex.flex-col:has(.sendbox-tools):focus-within {
  overflow: visible !important;
  border: 2px solid transparent !important;
  background-color: var(--dialog-fill-0) !important;
  background-image:
    linear-gradient(var(--dialog-fill-0), var(--dialog-fill-0)), var(--retroma-aurora-input-gradient) !important;
  background-size:
    100% 100%,
    220% 100% !important;
  background-repeat: no-repeat, no-repeat !important;
  background-position:
    center center,
    0% 50% !important;
  background-origin: border-box !important;
  background-clip: padding-box, border-box !important;
  box-shadow:
    0 0 0 1px var(--retroma-aurora-input-ring),
    0 0 16px rgba(255, 106, 1, 0.1),
    0 0 18px rgba(138, 43, 226, 0.1),
    0 0 20px rgba(0, 191, 255, 0.12),
    var(--retroma-aurora-input-shadow) !important;
  animation:
    elegantFlow 2.4s linear infinite,
    softGlow 3s ease-in-out infinite;
}

.guidContainer .guidInputCard textarea::placeholder,
[class*='guidContainer'] [class*='guidInputCard'] textarea::placeholder,
.relative.p-16px.border-3.b.bg-dialog-fill-0.b-solid.rd-20px.flex.flex-col:has(.sendbox-tools) textarea::placeholder {
  color: var(--retroma-aurora-placeholder) !important;
  opacity: 1 !important;
}

@keyframes elegantFlow {
  0% {
    background-position:
      center center,
      0% 50%;
  }
  100% {
    background-position:
      center center,
      220% 50%;
  }
}

@keyframes softGlow {
  0%,
  100% {
    box-shadow:
      0 0 0 1px var(--retroma-aurora-input-ring),
      0 0 16px rgba(255, 106, 1, 0.1),
      0 0 18px rgba(138, 43, 226, 0.1),
      0 0 20px rgba(0, 191, 255, 0.12),
      var(--retroma-aurora-input-shadow);
  }
  50% {
    box-shadow:
      0 0 0 1px color-mix(in srgb, #f8c91c 30%, transparent),
      0 0 18px rgba(255, 106, 1, 0.14),
      0 0 22px rgba(138, 43, 226, 0.14),
      0 0 24px rgba(0, 191, 255, 0.18),
      var(--retroma-aurora-input-shadow-strong);
  }
}

/* RoseUi Theme Background Start */
/* Preview cover only: do not auto-inject full-page background image */
/* RoseUi Theme Background End */
`;/**
 * @license
 * Copyright 2025 DoDidDoneUi (dodiddoneui.com)
 * SPDX-License-Identifier: Apache-2.0
 */const R=0,A=(n,a,e,r,t)=>({id:n,name:a,appearance:e,css:r,cover:t,builtin:!0,created_at:R,updated_at:R}),Sn=[{id:V,name:"Light",appearance:"light",cover:ne,builtin:!0,created_at:R,updated_at:R},{id:Cn,name:"Dark",appearance:"dark",builtin:!0,created_at:R,updated_at:R},A("misaka-mikoto-theme","Misaka Mikoto Theme","light",se,ae),A("hello-kitty","Hello Kitty","light",ie,ee),A("retro-windows","Retro Windows","light",de,re),A("retroma-y2k-jp-v42-pure","Y2K电子账本 by 椰树女王","light",ce,oe),A("retroma-obsidian-book","Retroma Obsidian Book","dark",le,te),A("discourse-horizon","Discourse Horizon","light",pe),A("glittering-input-field","Glittering Input Field","light",be)];new Set(Sn.map(n=>n.id));/**
 * @license
 * Copyright 2025 DoDidDoneUi (dodiddoneui.com)
 * SPDX-License-Identifier: Apache-2.0
 */const me=n=>!n||!n.trim()?"":n.replace(/([a-zA-Z-]+)\s*:\s*([^;!}]+);/g,(a,e,r)=>{const t=r.trim();return t.endsWith("!important")?a:`${e}: ${t} !important;`}),ge=n=>!n||!n.trim()?"":`
/* 用户自定义样式 - 自动添加 !important 提升优先级 */
/* User Custom Styles - Auto !important for highest priority */
${n}
  `.trim(),he=n=>{const a=me(n);return ge(a)};/**
 * @license
 * Copyright 2025 DoDidDoneUi (dodiddoneui.com)
 * SPDX-License-Identifier: Apache-2.0
 */const ue="(prefers-color-scheme: dark)";function In(){return typeof window>"u"||typeof window.matchMedia!="function"?null:window.matchMedia(ue)}function fe(){return In()?.matches??!1}function ze(n){const a=In();if(!a)return()=>{};const e=r=>n(r.matches);return a.addEventListener("change",e),()=>a.removeEventListener("change",e)}/**
 * @license
 * Copyright 2025 DoDidDoneUi (dodiddoneui.com)
 * SPDX-License-Identifier: Apache-2.0
 */const xe="theme-tokens",ke="theme-decoration";function hn(n,a,e=document){const r=e.getElementById(n);if(!a){r?.remove();return}const t=r??e.createElement("style");t.id=n,t.textContent=a,e.head.appendChild(t)}function ve(n){return!n||Object.keys(n).length===0?null:`:root {
${Object.entries(n).map(([e,r])=>`  ${e}: ${r};`).join(`
`)}
}`}function ye(n,a=document){a.documentElement.setAttribute("data-theme",n.appearance),a.body?.setAttribute("arco-theme",n.appearance),hn(xe,ve(n.tokens),a),hn(ke,n.css?he(n.css):null,a)}async function Ae(n){const a=gn.get("theme.userThemes")??[],e=Va(n,[...Sn,...a],fe());ye(e),await gn.set("theme.activeId",n),await _n.setActive.invoke(e)}export{Se as $,ma as A,Sn as B,Ka as C,Cn as D,aa as E,Ja as F,ba as G,ka as H,i as I,Ie as J,ua as K,V as L,ze as M,Ae as N,Va as O,fe as P,Mn as Q,va as R,Xa as S,Ea as T,Aa as U,Wa as V,Ta as W,Ua as X,$a as Y,Ra as Z,d as _,la as a,Ba as a0,Oa as a1,Da as a2,pa as a3,La as b,gn as c,fa as d,Na as e,xa as f,D as g,M as h,Ce as i,Sa as j,ye as k,Pa as l,Ia as m,ya as n,wa as o,Ca as p,ha as q,ca as r,ja as s,_n as t,ga as u,me as v,Ga as w,Yn as x,Fa as y,Ya as z};
