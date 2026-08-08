import{h as n}from"./applyTheme-DjbdwZGr.js";/**
 * @license
 * Copyright 2025 DoDidDoneUi (dodiddoneui.com)
 * SPDX-License-Identifier: Apache-2.0
 */async function i(t){return(await n("GET",`/api/settings/client?keys=${encodeURIComponent(t)}`))?.[t]}async function a(t,s){await n("PUT","/api/settings/client",{[t]:s})}async function c(t){await n("PUT","/api/settings/client",{[t]:null})}export{i as g,c as r,a as s};
