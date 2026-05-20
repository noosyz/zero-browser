# Zero Browser

A Rust web browser built in two stages:
- **Stage 1 (now):** wry-based shell with native UI, tabs, history, bookmarks.
- **Stage 2 (later):** custom rendering engine replacing wry piece by piece.

## My background
- CS Master's student, new to Rust.
- Prefers mechanism-first explanations before analogies.
- Minimal code comments (one line max per comment, only when non-obvious).

## Stack
- **Workspace** at repo root, member crates under `crates/`.
- **Current crates:** `shell` (binary, wry + tao).
- **Toolchain:** stable Rust via rustup.
- **Linux:** Arch, X11 session, WebKitGTK 4.1 installed.

## Build & run

```bash
cargo check                                          # fast type-check, run constantly
cargo build -p shell                                 # debug build
WEBKIT_DISABLE_DMABUF_RENDERER=1 GDK_BACKEND=x11 cargo run -p shell
cargo fmt                                            # format before commit
cargo clippy --workspace --all-targets -- -D warnings
```

## Conventions
- Use `anyhow::Result` in application code, `thiserror` later in library crates.
- One module per file. Prefer `pub(crate)` over `pub` unless a type is truly public.
- No `unwrap()` or `expect()` outside tests and `main()`.
- All errors propagated with `?`, never swallowed.
- Window/WebView env-var workarounds are set in code via `std::env::set_var` at the top of `main()`, not relied on from the shell.

## Working style
- Before any non-trivial change: run `cargo check`.
- Before declaring a task done: run `cargo fmt && cargo clippy && cargo build -p shell`.
- When stuck on a `wry`/`tao` issue, check `cargo tree -p shell -i tao` and `cargo tree -p shell -i wry` for version skew.
- If a task requires touching more than 3 files, write a brief plan first and confirm.

## Out of scope (do not touch)
- The custom rendering engine. We're in Stage 1.
- Cross-platform packaging (deb/dmg/exe). Stage 1.7 deals with that.
- JavaScript engine integration. Stage 2.

## Phase tracking
See `docs/ROADMAP.md`. Current phase: **1.1 — Foundation & Window Shell**.