import{R as y,r as c,j as e,X as Z,z as Q,Y as A,Z as ee,q as te,a as re}from"./vendor-react-BBeMfEpA.js";import{k as oe}from"./vendor-katex-CrIz_Pv6.js";import{a8 as ne,A as ae,I as T,u as D,y as se,a9 as U,aa as V,ab as ie,b as le,ac as ce,ad as de,ae as pe,af as ue,ag as me,n as he}from"./index-NlN9zNgo.js";import{f as xe,t as N,v as ge}from"./applyTheme-DjbdwZGr.js";import{b7 as X}from"./mermaid.core-kJWml8rw.js";import"./vendor-diff-AIAqKCH4.js";import"./vendor-editor-DX5t_mwd.js";import{M}from"./vendor-arco-C17IKtUQ.js";import{h as B,v as K,a as G,r as fe,b as W,M as be,c as ye,d as ve,e as we,f as ke}from"./vendor-markdown-Dl3FZDcy.js";const Se=t=>{const r=y.createContext({value:t,setValue(){console.warn("")}}),s=()=>y.useContext(r).value,a=()=>y.useContext(r).setValue,d=t;return[s,p=>{const[m,i]=c.useState(p.value||JSON.parse(JSON.stringify(d))),o=c.useRef(!0);return c.useEffect(()=>{o.current||(i(p.value),o.current=!1)},[p.value]),e.jsx(r.Provider,{value:{value:m,setValue:i},children:p.children})},a]},je=T(Z),[Ce,Te,Le]=Se({root:""}),$=({src:t,alt:r,className:s})=>{const[a,d]=c.useState(!0),[x,p]=c.useState(t),{root:m}=Ce(),i=c.useMemo(()=>!m||t.startsWith("http")||t.startsWith("data:")||t.startsWith("/")||t.startsWith("file:")||t.startsWith("\\")||/^[A-Za-z]:/.test(t)?t:ne(m,t),[t,m]);return c.useEffect(()=>{d(!0),xe.getImageBase64.invoke({path:i,workspace:m||void 0}).then(o=>{o&&p(o),d(!1)}).catch(o=>{console.error("[LocalImageView] Failed to load image:",{path:i,error:o}),d(!1)})},[i]),a?e.jsxs("span",{style:{display:"flex",alignItems:"center",gap:"4px"},children:[e.jsx(je,{className:"loading",style:{display:"flex"},theme:"outline",size:"14",fill:ae.primary,strokeWidth:2}),e.jsx("span",{children:r})]}):e.jsx("img",{src:x,alt:r,className:s})};$.Provider=Te;$.useUpdateLocalImage=Le;const Re=T(A),ze=T(Q);let O=null;const Ee=t=>{O!==t&&(X.initialize({startOnLoad:!1,securityLevel:"strict",suppressErrorRendering:!0,theme:t==="dark"?"dark":"default",fontFamily:"inherit"}),O=t)},Ie=t=>t.replace(/<svg\b([^>]*)>/i,(r,s)=>/style\s*=/.test(s)?`<svg${s.replace(/style\s*=\s*(["'])(.*?)\1/i,(a,d,x)=>` style=${d}${x};max-width: 100%; height: auto; display: block;${d}`)}>`:`<svg${s} style="max-width: 100%; height: auto; display: block;">`);function Me({code:t,style:r,showOpenInPanelButton:s=!0}){const{t:a}=D(),{openPreview:d}=se(),x=c.useRef(`mermaid-${Math.random().toString(36).slice(2,10)}`),p=c.useRef(null),[m,i]=c.useState(null),[o,l]=c.useState(!1),[v,n]=c.useState("source"),[h,u]=c.useState(t),[f,w]=c.useState(()=>document.documentElement.getAttribute("data-theme")||"light");c.useEffect(()=>{const g=setTimeout(()=>u(t),300);return()=>clearTimeout(g)},[t]),c.useEffect(()=>{const g=()=>{const E=document.documentElement.getAttribute("data-theme")||"light";w(E)},b=new MutationObserver(g);return b.observe(document.documentElement,{attributes:!0,attributeFilter:["data-theme"]}),()=>b.disconnect()},[]),c.useEffect(()=>{let g=!1;const b=h.trim();return b?(i(null),l(!0),(async()=>{try{Ee(f);const{svg:L}=await X.render(`${x.current}-${Date.now()}`,b);g||(i(Ie(L)),l(!1),n(p.current==="source"?"source":"preview"))}catch{g||(i(null),l(!1),n("source"))}})(),()=>{g=!0}):(i(null),l(!1),n("source"),()=>{g=!0})},[h,f]);const C=f==="dark"?K:G,z=o&&p.current!=="source",k=t.split(/\r?\n/).map(g=>g.trim()).find(Boolean),P=k&&k.length>0?`${a("preview.mermaidTitle")}: ${k.slice(0,48)}${k.length>48?"...":""}`:a("preview.mermaidTitle");return e.jsx("div",{style:{width:"100%",minWidth:0,maxWidth:"100%",...r},children:e.jsxs("div",{style:{border:"1px solid var(--bg-3)",borderRadius:"0.3rem",overflow:"hidden",overflowX:"auto"},children:[e.jsxs("div",{style:{display:"flex",justifyContent:"space-between",alignItems:"center",gap:"8px",backgroundColor:"var(--bg-2)",borderTopLeftRadius:"0.3rem",borderTopRightRadius:"0.3rem",padding:"6px 10px",borderBottom:"1px solid var(--bg-3)"},children:[e.jsxs("div",{style:{display:"flex",alignItems:"center",gap:"8px"},children:[e.jsx("span",{style:{textDecoration:"none",color:"var(--text-secondary)",fontSize:"12px",lineHeight:"20px"},children:"<mermaid>"}),m&&e.jsxs("div",{style:{display:"flex",alignItems:"center",gap:"4px"},children:[e.jsx("div",{style:{cursor:"pointer",color:v==="preview"?"var(--text-primary)":"var(--text-secondary)",fontSize:"12px",lineHeight:"20px"},onMouseDown:g=>{g.button===0&&(g.preventDefault(),p.current="preview",n("preview"))},children:a("preview.preview")}),e.jsx("span",{style:{color:"var(--text-secondary)",fontSize:"12px",lineHeight:"20px"},children:"/"}),e.jsx("div",{style:{cursor:"pointer",color:v==="source"?"var(--text-primary)":"var(--text-secondary)",fontSize:"12px",lineHeight:"20px"},onMouseDown:g=>{g.button===0&&(g.preventDefault(),p.current="source",n("source"))},children:a("preview.source")})]})]}),e.jsxs("div",{style:{display:"flex",alignItems:"center",gap:"8px",flexShrink:0},children:[s&&e.jsx(ze,{"data-testid":"mermaid-open-in-panel",theme:"outline",size:"18",style:{cursor:"pointer",flexShrink:0},fill:"var(--text-secondary)",title:a("preview.openInPanelTooltip"),onClick:()=>{d(`\`\`\`mermaid
${t}
\`\`\``,"markdown",{title:P,editable:!1})}}),e.jsx(Re,{"data-testid":"mermaid-copy",theme:"outline",size:"18",style:{cursor:"pointer",flexShrink:0},fill:"var(--text-secondary)",onClick:()=>{U(t).then(()=>{M.success(a("common.copySuccess"))}).catch(()=>{M.error(a("common.copyFailed"))})}})]})]}),m&&v==="preview"?e.jsx("div",{"data-testid":"mermaid-diagram",style:{backgroundColor:"var(--bg-1)",padding:"12px",overflowX:"auto",display:"flex",justifyContent:"center"},dangerouslySetInnerHTML:{__html:m}}):z?e.jsxs("div",{"data-testid":"mermaid-loading",style:{backgroundColor:"var(--bg-1)",padding:"16px 12px",display:"flex",alignItems:"center",justifyContent:"center",gap:"8px",color:"var(--text-secondary)",fontSize:"13px",lineHeight:"20px"},children:[e.jsx("div",{"aria-hidden":"true",className:"loading",style:{width:"12px",height:"12px",borderRadius:"999px",border:"2px solid var(--bg-3)",borderTopColor:"var(--text-secondary)",flexShrink:0}}),e.jsx("span",{children:a("preview.loading")})]}):e.jsx(B,{children:t,language:"mermaid",style:C,PreTag:"div",customStyle:{margin:0,borderRadius:0,border:"none",background:"transparent",color:"var(--text-primary)",overflowX:"auto",maxWidth:"100%"},codeTagProps:{style:{color:"var(--text-primary)"}}})]})})}const Pe=y.memo(Me),_e=T(A),F=T(te),H=T(ee),_=3,De=20,$e=13,Ve=_*De+$e;function Ne(t){const{t:r}=D(),[s,a]=c.useState(!1),d=c.useRef(null),[x,p]=c.useState(()=>document.documentElement.getAttribute("data-theme")||"light");y.useEffect(()=>{const S=()=>{p(document.documentElement.getAttribute("data-theme")||"light")},R=new MutationObserver(S);return R.observe(document.documentElement,{attributes:!0,attributeFilter:["data-theme"]}),()=>R.disconnect()},[]);const m=()=>{const S=s;a(R=>!R),S&&d.current&&requestAnimationFrame(()=>{d.current?.scrollIntoView({block:"nearest",behavior:"auto"})})},{children:i,className:o,node:l,hiddenCodeCopyButton:v,codeStyle:n,...h}=t,f=/language-(\w+)/.exec(o||"")?.[1]||"text";if(f==="latex"||f==="math"||f==="tex"){const S=String(i).replace(/\n$/,"");if(!/\\(documentclass|begin\{document\}|usepackage)\b/.test(S))try{const Y=oe.renderToString(S,{displayMode:!0,throwOnError:!1});return e.jsx("div",{className:"katex-display",dangerouslySetInnerHTML:{__html:Y}})}catch{}}if(f==="mermaid")return e.jsx(Pe,{code:V(i),style:t.codeStyle});if(!String(i).includes(`
`))return e.jsx("code",{...h,className:o,style:{fontWeight:"bold"},children:i});const w=f==="diff",C=V(i),z=C.split(`
`).length,k=z>_,P=x==="dark"?K:G,g=w?C.split(`
`):[],b=x==="dark",E=()=>{U(C).then(()=>{try{M.success(r("common.copySuccess"))}catch{}}).catch(()=>{try{M.error(r("common.copyFailed"))}catch{}})},L=b?"rgba(255,255,255,0.55)":"rgba(0,0,0,0.45)",I=b?"rgba(255,255,255,0.45)":"rgba(0,0,0,0.35)",q=b?"rgba(255,255,255,0.08)":"rgba(0,0,0,0.06)",J=b?"rgba(255,255,255,0.04)":"var(--bg-2)";return e.jsx("div",{ref:d,style:{width:"100%",minWidth:0,maxWidth:"100%",...t.codeStyle},className:"group",children:e.jsxs("div",{style:{backgroundColor:J,borderRadius:"8px",overflow:"hidden"},children:[e.jsxs("div",{style:{display:"flex",justifyContent:"space-between",alignItems:"center",padding:"8px 12px"},children:[e.jsx("span",{style:{color:I,fontSize:"12px",lineHeight:"16px"},children:f.toLocaleLowerCase()}),e.jsxs("div",{className:"opacity-100 md:opacity-0 md:group-hover:opacity-100 transition-opacity",style:{display:"flex",alignItems:"center",gap:"8px"},children:[k&&e.jsx("span",{title:r(s?"common.collapse":"common.expand"),style:{display:"flex"},children:s?e.jsx(H,{theme:"outline",size:"14",style:{cursor:"pointer",display:"block"},fill:L,onClick:m}):e.jsx(F,{theme:"outline",size:"14",style:{cursor:"pointer",display:"block"},fill:L,onClick:m})}),e.jsx("span",{title:r("common.copy"),style:{display:"flex"},children:e.jsx(_e,{theme:"outline",size:"14",style:{cursor:"pointer",display:"block"},fill:L,onClick:E})})]})]}),e.jsx("div",{style:{maxHeight:k&&!s?`${Ve}px`:"none",overflowY:"hidden",overflowX:"visible"},children:e.jsx(B,{children:C,language:f,style:P,PreTag:"div",wrapLines:w,lineProps:w?S=>({style:{display:"block",...ie(g[S-1]||"",b)}}):void 0,customStyle:{margin:0,padding:"0 12px 8px",borderRadius:0,border:"none",background:"transparent",color:"var(--text-primary)",overflowX:"auto",maxWidth:"100%"},codeTagProps:{style:{color:"var(--text-primary)",background:"transparent"}}})}),k&&e.jsxs("div",{style:{display:"flex",justifyContent:"center",alignItems:"center",padding:"6px 12px",cursor:"pointer",gap:"4px",borderTop:`1px solid ${q}`},onClick:m,children:[e.jsx("span",{style:{color:I,fontSize:"12px"},children:s?r("common.collapse"):r("common.viewMoreLines",{count:z-_})}),s?e.jsx(H,{theme:"outline",size:"12",fill:I}):e.jsx(F,{theme:"outline",size:"12",fill:I})]})]})})}const We=(t="light",r,s,a)=>{const d=document.createElement("style"),x=r?Object.entries(r).map(([o,l])=>`${o}: ${l};`).join(`
    `):"",p=a?"19.6px":"24px",m=a?"var(--chat-font-size, 14px)":"var(--chat-font-size, 16px)",i=a?"16px":"12px";return d.innerHTML=`
  /* Shadow DOM CSS variable definitions */
  :host {
    ${x}
  }

  * {
    line-height:${p};
    font-size:${m};
    color: inherit;
  }

  .markdown-shadow-body {
    word-break: break-word;
    overflow-wrap: anywhere;
    color: var(--text-primary);
    max-width: 100%;
  }
  .markdown-shadow-body>p:first-child
  {
    margin-top:0px;
  }
  h1,h2,h3,h4,h5,h6{
    margin-block-start:0px;
    margin-block-end:0px;
  }
  .markdown-shadow-body p {
    margin-block-start: ${i};
    margin-block-end: ${i};
  }
  .markdown-shadow-body li {
    margin-block-start: 6px;
    margin-block-end: 6px;
  }
  a{
    color:var(--primary);
    text-decoration: none;
    cursor: pointer;
    word-break: break-all;
    overflow-wrap: anywhere;
  }
  .markdown-local-file-link {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    max-width: 100%;
    min-height: 22px;
    padding: 2px 6px;
    background: var(--bg-2);
    color: var(--text-primary);
    border: 1px solid transparent;
    border-radius: 6px;
    box-shadow: none;
    font: inherit;
    line-height: inherit;
    vertical-align: baseline;
    cursor: pointer;
    transition:
      background-color 0.15s ease,
      border-color 0.15s ease;
  }
  span.markdown-local-file-link {
    cursor: default;
  }
  .markdown-local-file-link:hover {
    background: var(--bg-3);
    color: var(--text-primary);
    text-decoration: none;
  }
  .markdown-local-file-link .truncate {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .markdown-local-file-line {
    padding: 0 4px;
    border-radius: 4px;
    background: var(--bg-3);
    color: var(--text-secondary);
  }
  .markdown-local-file-copy {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    width: 20px;
    height: 20px;
    min-width: 20px;
    padding: 1px;
    border: 0;
    border-radius: 4px;
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
  }
  .markdown-local-file-copy:hover {
    background: var(--bg-3);
    color: var(--text-primary);
  }
  h1{
    font-size: 24px;
    line-height: 32px;
    font-weight: bold;
  }
  h2,h3,h4,h5,h6{
    font-size: 16px;
    line-height: 24px;
    font-weight: bold;
    margin-top: 20px;
    margin-bottom: 12px;
  }
  code span{
    font-size:var(--code-font-size, 13px);
    line-height:20px;
    font-family: var(--font-mono);
  }

  .markdown-shadow-body>p:last-child{
    margin-bottom:0px;
  }
  ol, ul {
    padding-inline-start:24px;
  }
  hr {
    border: none;
    border-top: 1px solid var(--bg-3);
    margin: 28px 0;
  }
  strong {
    font-weight: 600;
    color: var(--text-primary);
  }
  .markdown-shadow-body code:not(pre code) {
    background: var(--bg-3);
    color: var(--text-primary);
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 0.875em;
    font-family: var(--font-mono);
  }
  blockquote {
    border-left: 3px solid var(--bg-3);
    padding-left: 12px;
    color: var(--text-primary);
    margin: 16px 0;
  }
  pre {
    max-width: 100%;
    overflow-x: auto;
    margin-block-start: 8px;
    margin-block-end: 8px;
  }
  /* Code block horizontal scrollbar — blends with bg-2 */
  pre,
  .hljs {
    scrollbar-width: thin;
    scrollbar-color: ${t==="dark"?"rgba(255, 255, 255, 0.18)":"rgba(0, 0, 0, 0.1)"} transparent;
  }
  pre::-webkit-scrollbar,
  .hljs::-webkit-scrollbar {
    height: 6px;
    background: transparent;
  }
  pre::-webkit-scrollbar-track,
  .hljs::-webkit-scrollbar-track,
  pre::-webkit-scrollbar-corner,
  .hljs::-webkit-scrollbar-corner {
    background: transparent;
  }
  pre::-webkit-scrollbar-thumb,
  .hljs::-webkit-scrollbar-thumb {
    background-color: ${t==="dark"?"rgba(255, 255, 255, 0.18)":"rgba(0, 0, 0, 0.1)"};
    border-radius: 3px;
  }
  pre::-webkit-scrollbar-thumb:hover,
  .hljs::-webkit-scrollbar-thumb:hover {
    background-color: ${t==="dark"?"rgba(255, 255, 255, 0.28)":"rgba(0, 0, 0, 0.2)"};
  }
  img {
    max-width: 100%;
    height: auto;
  }
   /* Table border styles */
  table {
    border-collapse: collapse;
    th{
      padding: 8px;
      border: 1px solid var(--bg-3);
      background-color: var(--bg-1);
      font-weight: bold;
    }
    td{
        padding: 8px;
        border: 1px solid var(--bg-3);
        min-width: 120px;
    }
  }
  /* Inline code should wrap on small screens to avoid horizontal overflow */
  .markdown-shadow-body code {
    word-break: break-word;
    overflow-wrap: anywhere;
    max-width: 100%;
  }
  /* Allow KaTeX to use its own line-height for proper fraction/superscript rendering */
  .katex,
  .katex * {
    line-height: normal;
  }

  /* Display math: only scroll horizontally when formula exceeds container width */
  .katex-display {
    overflow-x: auto;
    overflow-y: hidden;
    padding: 0.5em 0;
  }

  .loading {
    animation: loading 1s linear infinite;
  }


  @keyframes loading {
    0% {
      transform: rotate(0deg);
    }
    100% {
      transform: rotate(360deg);
    }
  }

  /* User Custom CSS (injected into Shadow DOM) */
  ${s||""}
  `,d};let j=null;const Oe=()=>{if(j)return j;try{const t=[...document.styleSheets].find(s=>s.href?.includes("katex")||s.ownerNode?.dataset?.katex);if(t){const s=[...t.cssRules].map(a=>a.cssText).join(`
`);return j=new CSSStyleSheet,j.replaceSync(s),j}const r=[...document.styleSheets];for(const s of r)try{const a=[...s.cssRules];if(a.some(x=>x.cssText.includes(".katex"))){const x=a.map(p=>p.cssText).join(`
`);return j=new CSSStyleSheet,j.replaceSync(x),j}}catch{continue}}catch(t){console.warn("Failed to create KaTeX stylesheet for Shadow DOM:",t)}return null},Fe=({children:t})=>{const[r,s]=c.useState(null),a=y.useRef(null),[d,x]=c.useState(""),m=le()?.isMobile??!1;y.useEffect(()=>{let o=!0;const l=n=>{o&&x(n?.css?ge(n.css):"")};N.requestCurrent.invoke().then(l).catch(()=>{});const v=N.changed.on(n=>l(n));return()=>{o=!1,v?.()}},[]);const i=y.useCallback(o=>{const l=getComputedStyle(document.documentElement),v=document.documentElement.getAttribute("data-theme")||"light",n={"--bg-1":l.getPropertyValue("--bg-1"),"--bg-2":l.getPropertyValue("--bg-2"),"--bg-3":l.getPropertyValue("--bg-3"),"--color-text-1":l.getPropertyValue("--color-text-1"),"--color-text-2":l.getPropertyValue("--color-text-2"),"--color-text-3":l.getPropertyValue("--color-text-3"),"--text-primary":l.getPropertyValue("--text-primary"),"--text-secondary":l.getPropertyValue("--text-secondary"),"--chat-font-size":l.getPropertyValue("--chat-font-size"),"--code-font-size":l.getPropertyValue("--code-font-size")};a.current&&a.current.remove();const h=We(v,n,d,m);a.current=h,o.appendChild(h);const u=Oe();u&&!o.adoptedStyleSheets.includes(u)&&(o.adoptedStyleSheets=[...o.adoptedStyleSheets,u])},[d,m]);return y.useEffect(()=>{r&&i(r)},[r,d,i]),y.useEffect(()=>{if(!r)return;const o=new MutationObserver(()=>{i(r)});return o.observe(document.documentElement,{attributes:!0,attributeFilter:["data-theme","class","style"]}),()=>o.disconnect()},[r,i]),e.jsx("div",{ref:o=>{if(!o||o.__init__shadow)return;o.__init__shadow=!0;const l=o.attachShadow({mode:"open"});i(l),s(l)},className:"markdown-shadow",style:{width:"100%",flex:"1 1 auto",minWidth:0},children:r&&re.createPortal(t,r)})},He=[ye,ve,we],Ae=t=>!(t.startsWith("http://")||t.startsWith("https://")||t.startsWith("data:")),Ue=y.memo(({hiddenCodeCopyButton:t,codeStyle:r,className:s,onRef:a,onLocalFileLink:d,allowHtml:x,children:p})=>{const{t:m}=D(),i=c.useMemo(()=>{if(typeof p=="string"){let n=p.replace(/file:\/\//g,"");return n=ce(n),n}return p},[p]),o=c.useCallback(n=>{n.preventDefault(),n.stopPropagation();const h=n.currentTarget.href;h&&de(h).catch(u=>{console.error(m("messages.openLinkFailed"),u)})},[m]),l=c.useMemo(()=>({span:({node:n,className:h,children:u,...f})=>e.jsx("span",{...f,className:h,children:u}),code:n=>e.jsx(Ne,{...n,codeStyle:r,hiddenCodeCopyButton:t}),a:({node:n,...h})=>{const u=h,f=typeof u.href=="string"?u.href:"",w=pe(f);return w?e.jsx(ue,{reference:w,onOpen:d,children:u.children}):e.jsx("a",{...u,href:u.href,target:"_blank",rel:"noreferrer",onClick:o})},table:({node:n,...h})=>e.jsx("div",{style:{overflowX:"auto",maxWidth:"100%"},children:e.jsx("table",{...h,style:{...h.style,borderCollapse:"collapse",border:"1px solid var(--bg-3)",minWidth:"100%"}})}),td:({node:n,...h})=>e.jsx("td",{...h,style:{...h.style,padding:"8px",border:"1px solid var(--bg-3)",minWidth:"120px"}}),img:({node:n,...h})=>{const u=h;if(Ae(u.src||"")){const f=decodeURIComponent(u.src||"");return e.jsx($,{src:f,alt:u.alt||"",className:u.className})}return e.jsx("img",{...u,alt:u.alt||""})}}),[r,t,o,d]),v=c.useMemo(()=>x?[fe,W]:[W],[x]);return e.jsx("div",{className:he("relative w-full",s),children:e.jsx(Fe,{children:e.jsx("div",{ref:a,className:"markdown-shadow-body",children:e.jsx(be,{remarkPlugins:He,rehypePlugins:v,components:l,urlTransform:n=>me(n)?n:ke(n),children:i})})})})});Ue.displayName="MarkdownView";export{$ as L,Ue as M,Se as c};
