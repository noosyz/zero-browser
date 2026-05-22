# Zero Browser Roadmap

Living document. Phases ordered, not dated. Tick boxes as work lands on `main`.

## Stage 1 — wry-based Shell

Native chrome over WebKitGTK (Linux) / WebView2 (Windows) / WKWebView (macOS) via `wry`. Goal: usable daily-driver browser that proves the UX before swapping the engine.

### 1.0 — Project Bootstrap
- [x] Cargo workspace + `shell` binary crate
- [x] License, repo metadata
- [x] Toolchain pinned to stable

### 1.1 — Foundation & Window Shell
- [x] `tao` window + `wry::WebView` mounted in default vbox
- [x] Address bar (`gtk::Entry`) with Enter-to-load
- [x] Back / Forward / Reload / Home buttons with sensitivity sync
- [x] Document-title → window-title sync
- [x] WebKitGTK env workarounds (`WEBKIT_DISABLE_DMABUF_RENDERER`, `GDK_BACKEND=x11`) baked in
- [x] Chrome font override via GTK CSS provider
- [x] Keyboard shortcuts: Ctrl+L, Ctrl+R, Ctrl+T, Ctrl+W, Ctrl+Tab, Ctrl+1..9, Alt+Left/Right
- [x] Loading progress indicator (`gtk::ProgressBar` driven by `estimated_load_progress`)
- [x] Status bar with hover URL (`mouse_target_changed` → `HitTestResult::link_uri`)

### 1.2 — Tabs
- [x] `gtk::Notebook`-backed `TabManager`
- [x] Per-tab `WebView`, shared `WebContext`
- [x] Close button per tab; close-last → quit
- [x] `zero://newtab/` custom protocol + static HTML
- [x] URL normalizer (scheme detect, loopback→http, bare-host→https, search fallback)
- [x] Ctrl+T new tab, Ctrl+W close, Ctrl+Tab cycle, Ctrl+1..9 jump
- [x] Middle-click link → background tab (`decide-policy` + `mouse_button == 2`)
- [x] Tab reordering (drag) with `Vec<Tab>` resync via `page-reordered`
- [x] `target=_blank` / `window.open` → foreground tab (`connect_create` returns `None`)

### 1.3 — History *(current)*
- [ ] Persist `(url, title, visited_at)` to SQLite (`rusqlite` or `sqlx`)
- [ ] In-memory LRU for current session
- [ ] Omnibox suggestions from history (prefix + frecency)
- [ ] `zero://history/` viewer page
- [ ] Clear-history action

### 1.4 — Bookmarks
- [ ] SQLite-backed bookmark store (folder tree)
- [ ] Star icon in address bar to toggle current page
- [ ] `zero://bookmarks/` manager
- [ ] Bookmark bar (toggleable)
- [ ] Import from Firefox / Chrome JSON

### 1.5 — Settings & Persistence
- [ ] On-disk config (TOML in `$XDG_CONFIG_HOME/zero-browser/`)
- [ ] Home page, search engine, fonts, theme
- [ ] `zero://settings/` UI
- [ ] Profile dir + multi-profile launch flag

### 1.6 — Downloads & Find-in-page
- [ ] Download manager (`zero://downloads/`)
- [ ] Find-in-page (Ctrl+F) using WebKit's text-find API
- [ ] Zoom in/out/reset (Ctrl +/-/0)

### 1.7 — Cross-Platform Packaging
- [ ] CI matrix: Linux (Arch + Ubuntu), macOS, Windows
- [ ] `.deb` + AppImage (Linux)
- [ ] `.dmg` (macOS, codesigned)
- [ ] `.msi` / NSIS installer (Windows)
- [ ] Auto-update channel (stable / nightly)

## Stage 2 — Custom Rendering Engine

Replace `wry` piece by piece. Each milestone keeps the browser shippable.

### 2.0 — Networking
- [ ] HTTP/1.1 + HTTP/2 client (likely `hyper` + `rustls`)
- [ ] Cookie jar, cache, TLS pinning hooks
- [ ] Replace WebKit's network stack first (still WebKit-rendered)

### 2.1 — HTML Parser & DOM
- [ ] HTML5 tokenizer + tree construction (likely fork `html5ever`)
- [ ] Owned DOM (Rust arena, no `Rc<RefCell>`)

### 2.2 — CSS Parser & Cascade
- [ ] `cssparser` integration
- [ ] Selector matching, specificity, cascade, computed style

### 2.3 — Layout
- [ ] Block / inline formatting contexts
- [ ] Flexbox
- [ ] Grid (later)

### 2.4 — Painting & Compositing
- [ ] Display lists
- [ ] GPU compositor (`wgpu`)

### 2.5 — JavaScript Engine
- [ ] Integrate `boa` or embed V8 — decision deferred
- [ ] Bind DOM ↔ JS

## Out-of-Scope (for now)
- Mobile (Android/iOS)
- Extensions / WebExtensions API
- DevTools protocol parity

## Open Questions
- Storage: single SQLite DB vs per-feature files?
- Multi-process architecture (Chromium-style site isolation) — Stage 1 or Stage 2?
- Renderer process sandboxing strategy
