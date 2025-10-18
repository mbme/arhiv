# Repository Guidelines

## Project Structure & Module Organization
Use the Rust workspace in the repo root as the source of truth. Focus on the crates under `baza/`, `arhiv/`, `binutils/`, and `rs-utils/`; platform wrappers live in `arhiv-android/` and `arhiv-desktop/`, while shared static assets sit in `resources/`.
```
repo root
├─ arhiv/          # Rust core services + embedded web UI assets
├─ binutils/       # CLI binaries (web server, tooling)
├─ baza/           # domain & persistence layer
├─ rs-utils/       # cross-cutting Rust utilities
├─ arhiv-desktop/  # Electron shell
├─ arhiv-android/  # Android wrapper & Gradle project
└─ justfile        # orchestration entry points
```
The `arhiv/src/ui/` subtree mixes Rust module glue with React/TypeScript; keep DTOs mirrored in both `dto.rs` and `dto.ts`.

## Build, Test, and Development Commands
- `just check` — run full Rust + TypeScript lint/test suite (`cargo clippy`, `cargo test`, `npm run check`).
- `just run` — launch the dev server with live JS/CSS rebuilds (tmux panes, Chromium browser).
- `npm run build --workspace arhiv` — rebuild frontend assets for embedding or static serving.
- `cargo run -p binutils --bin arhiv server` — start the backend manually; set `DEV_ARHIV_ROOT`, `SERVER_PORT`, `RUST_LOG` as needed.
- `just desktop` — package UI, rebuild Electron wrapper, and start the desktop shell.

## Coding Style & Naming Conventions
Global `.editorconfig` enforces LF endings, trailing whitespace trim, and 2-space indents (Rust: 4). Run `cargo fmt` with the repo’s Rust 2024 configuration before committing. TypeScript uses ESLint strict configs with Prettier; prefer `npm run fmt`/`npm run check-fmt` rather than ad-hoc formatting. React components stay in PascalCase directories (`Workspace/SessionList.tsx`), hooks in `camelCase`, and Rust modules follow snake_case filenames.

## Testing Guidelines
Rust crates rely on `cargo test`; snapshot-heavy areas use `insta`, so record updates deliberately (`INSTA_ACCEPT=overwrite`). The frontend uses Node’s built-in test runner via `npm run test --workspace arhiv`, backed by `tsx` and `global-jsdom`; tests live alongside components as `*.test.ts[x]`. Add regression coverage whenever patching bugs, and keep headless tests hermetic (no network, deterministic fixtures).

## Commit & Pull Request Guidelines
History favors short imperative summaries (`bump deps`, `fix theme toggle`). Keep scope focused per commit; squash fixups locally. Before opening a PR, ensure `just check` passes, describe intent + key changes, link issues, and attach UI screenshots for visible tweaks. Note any configuration or migration steps in the PR body so reviewers can reproduce locally without guesswork.

## Security & Configuration Tips
Use OS keyrings via the `keyring` crate for secrets; never hardcode credentials. For local runs, default `DEV_ARHIV_ROOT=~/temp/arhiv`, `SERVER_PORT=8443`, and override via env vars per shell. Preserve `RUST_LOG` granularity (`debug,h2=info,…`) when debugging to avoid noisy traces in committed configs.
