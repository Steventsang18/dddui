# DoDidDoneUi — Local Vertical-Industry Agent Platform

> A local-first, vertical-industry Agent orchestration platform. A single Rust binary + your browser, with no data leaving your machine. Derived from upstream open-source code distributed under the Apache-2.0 License (attribution in the License section below).

DoDidDoneUi is **not a general-purpose AI chat app**. It is a **local Agent solution for specific vertical industries** — legal, education, healthcare, finance, and similar — with domain knowledge, workflows, and compliance tuning built in.

## Why DoDidDoneUi?

- **Your data never leaves your machine.** A single Rust binary serves the whole app in your browser. There is no cloud backend (except the model APIs you choose to connect with your own key).
- **Vertical-industry ready.** Domain knowledge bases, workflows, and compliance guardrails are tuned for specific markets rather than one-size-fits-all chat.
- **Local-first & zero desktop client.** It runs as a pure web app — no Tauri, no Electron, no `.exe`/`.dmg` installer. One person can deploy it on a laptop.

## Features

### 🤖 Multi-Agent Orchestration
Chain and coordinate multiple specialized agents to solve complex, multi-step tasks. Each agent can be given a role, tools, and a knowledge scope, then composed into a workflow.

### 📚 Vertical-Industry Knowledge Base (Wiki)
A built-in Markdown knowledge base for your domain:
- Full-text search powered by SQLite **FTS5** (fast, deterministic, offline).
- **Bidirectional links** and typed edges between notes for knowledge graphs.
- Read/write/search from agents via the MCP `wiki_*` tools, or manage it through the UI.
- Drag in `pdf` / `docx` / `md` / `txt` — they are parsed locally (pure Rust) into a summary page plus searchable chunks. Raw files are never modified.

### 🔌 MCP (Model Context Protocol)
Extend the platform with MCP servers for external tools and data sources. Use the built-in wiki MCP tools, or bring your own.

### 💬 Conversations
Chat with models and agents in a clean conversation UI. If no model is configured yet, the app shows a friendly guide instead of an error.

### 🗂 Files & Office
Work with files on your machine and Office documents directly through the Agent — the agent can read, edit, and reason over your local files.

### ⏰ Scheduled Tasks
Set up recurring or timed jobs that run agents/workflows automatically.

### 👥 Team (future)
A team/collaboration layer is planned. In the current single-owner mode the app opens and works immediately with no login wall.

### 📦 Single Binary, No Client
The React frontend is compiled into the Rust binary at build time via `rust-embed`, then served same-origin at runtime. No separate web server, no desktop client, no extra directories to manage.

## Quick Start

### Option A — Just run it (recommended for most users)

If you want a production-style single binary with the UI already built in:

```bash
# 1. Build the single binary (frontend is embedded automatically)
./scripts/build-binary.sh --release
#    Output: backend/target/release/dodiddoneui

# 2. Start it (binds to 127.0.0.1 by default, opens your browser on first launch)
./backend/target/release/dodiddoneui --port 3080 --host 127.0.0.1

# 3. Open in your browser
#    http://127.0.0.1:3080
```

That's it — no Node, no extra server. The binary serves everything.

> **First-time setup:** Before you can chat or use agents, go to **Settings → Models** and configure a mainstream Chinese model (e.g. DeepSeek) with your own API Key. The app will then light up the conversation / Agent features.

### Option B — Development mode (hot reload)

Use this if you are a developer and want live-reload while editing the frontend.

```bash
# Terminal 1 — Backend
cd backend && cargo run -- --port 3080 --host 127.0.0.1 --identity-mode owner

# Terminal 2 — Frontend (Vite dev server with HMR)
cd frontend && npm install && npm run dev
#    Open http://127.0.0.1:5173 in your browser
```

> First-time model setup (Option A step above) applies here too: configure a model in **Settings → Models** before using chat / Agent features.

### Option C — Serve a pre-built frontend folder (advanced)

If you already have a frontend `dist` and want to skip the embedded copy:

```bash
./backend/target/release/dodiddoneui --port 3080 --host 127.0.0.1 --static-dir <path/to/frontend/dist>
```

## Architecture

```
dodiddoneui (single Rust binary)
 ├─ roseui-webhost   rust-embed embedded frontend dist + SPA static hosting (with history fallback)
 ├─ 26 business crates (Agent / MCP / Conversation / Files / Office / Team / Scheduler / Wiki ...)
 └─ roseui-wiki      vertical-industry knowledge base (FTS5 full-text search + bidirectional links + typed edges)

Browser  ←→  REST /api/*  +  WebSocket /ws  ←→  dodiddoneui (same origin)
```

## License & Attribution

- Original code: Apache License 2.0 — Copyright 2025 AionUi (aionui.com)
- This project's modifications and new code: Apache License 2.0 — Copyright 2026 DoDidDoneUi Team
- Bundled Rupoo agent engine: MIT License — Copyright 2026 Steventsang18 (see [`NOTICE`](./NOTICE))

See [`NOTICE`](./NOTICE) and [`LICENSE`](./LICENSE) (upstream copies: [`frontend/LICENSE`](./frontend/LICENSE) / [`backend/LICENSE`](./backend/LICENSE)).

This project is derived from Apache-2.0 upstream code and is distributed under the **independent name DoDidDoneUi**. Per Apache 2.0 Section 6, the upstream name and trademarks are not granted by the license, and this project does not imply any affiliation with the upstream.

## Disclaimer

This software is provided "AS IS", without warranty of any kind, express or implied. Before use, comply with the regulatory requirements of your industry (data privacy, professional licensing, etc.).

<!-- keep-test -->
