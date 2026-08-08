# Development Guide (DoDidDoneUi Frontend)

> This document covers the **frontend** (`upstream-AionUi`) only. The project is a
> single-binary web app: the Rust backend (`upstream-AionCore`) embeds the built
> frontend via `rust-embed` and serves it on the same origin. There is **no
> Electron desktop shell** and **no separate `RoseCore` repository** to clone.

## Prerequisites

- **Node.js 24** (use `npm`, not bun — the repo dropped bun in favor of npm)
- **Rust 1.95.0** (edition 2024) with the `roseui-app` workspace — install via [rustup](https://rustup.rs)
- On Windows, use the Rust **MSVC** toolchain.

## Repository Layout

The monorepo root (`rust-aion-ui`) contains:

```text
rust-aion-ui/
|-- upstream-AionUi/     # this package: React + Vite frontend
|-- upstream-AionCore/   # Rust backend (binary: dodiddoneui)
`-- scripts/             # one-shot build helpers (build-binary.sh)
```

## Quick Start (hot-reload dev mode)

Run the backend and frontend in two terminals:

```bash
# Terminal 1 — backend (server is the default action; no `serve` subcommand)
cd upstream-AionCore
cargo run -- --port 3080 --host 127.0.0.1 --identity-mode owner

# Terminal 2 — frontend (Vite dev server with HMR)
cd upstream-AionUi
npm install
npm run dev
# open http://127.0.0.1:5173
```

Vite proxies `/api` → `:3080` and `/ws` → `ws://:3080`, so the dev frontend
talks to the backend same-origin.

## Production single-binary build

```bash
# from the repo root
./scripts/build-binary.sh --release
# output: upstream-AionCore/target/release/dodiddoneui
```

The script runs `npm run build` (frontend), copies `dist/` into
`upstream-AionCore/crates/roseui-webhost/assets/`, then `cargo build --release`.
Run the binary and open `http://127.0.0.1:3080`.

## Code Quality

| Command              | Description                                |
| -------------------- | ------------------------------------------ |
| `npm run lint`       | Lint with oxlint (0 errors required)       |
| `npm run lint:fix`   | Auto-fix lint issues                       |
| `npm run format`     | Format with oxfmt                          |
| `npm run format:check` | Check formatting without writing files   |
| `npm run i18n:types` | Generate TypeScript types for i18n keys    |

## Testing

| Command                    | Description                     |
| -------------------------- | ------------------------------- |
| `npm run test`             | Run unit tests (vitest)         |
| `npm run test:watch`       | Watch mode                      |
| `npm run test:coverage`    | Coverage report                 |
| `npm run test:contract`    | Contract tests                  |
| `npm run test:integration` | Integration tests               |
| `npm run test:e2e`         | End-to-end tests (Playwright)   |

## Tech Stack

- **React 19** — UI framework
- **TypeScript** — type safety
- **Vite** — bundler (MPA, `vite.config.ts`)
- **UnoCSS** — atomic CSS engine
- **Arco Design** — component library
- **SQLx** (via backend) — local SQLite storage
- **vitest** — testing framework

## Backend changes

If you modify the backend (`upstream-AionCore`), see its own checks:

```bash
cd upstream-AionCore
cargo fmt --all -- --check
cargo clippy --workspace --exclude rupoo -- -D warnings
cargo nextest run --workspace
```

`rupoo` (under `extern/rupoo`) is a vendored external engine (MIT); it is
excluded from the project's clippy gate.
