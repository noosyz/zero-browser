---
name: wry-debugging
description: Use this skill whenever a wry/tao/WebKitGTK issue appears — blank window, GBM errors, "window handle kind not supported", missing renderer, or any cross-version dependency confusion between wry and tao.
---

# wry/tao debugging playbook

The Linux wry stack is fragile. Diagnose in this order.

## 1. Confirm session type
```bash
echo $XDG_SESSION_TYPE
```
Wayland is the usual culprit. XWayland fallback fixes most issues:
```bash
export GDK_BACKEND=x11
```

## 2. Check for version skew
```bash
cargo tree -p shell -i tao
cargo tree -p shell -i wry
```
If `tao` appears twice at different versions, that's the bug. Either remove the direct `tao` dependency and use `wry`'s re-exported version (older wry), or pin both crates to a known-compatible pair (newer wry).

## 3. Environment variables that fix common rendering issues
- `WEBKIT_DISABLE_DMABUF_RENDERER=1` — GBM buffer allocation failures
- `WEBKIT_DISABLE_COMPOSITING_MODE=1` — totally blank window with no errors
- `LIBGL_ALWAYS_SOFTWARE=1` — GPU driver doesn't support what WebKit wants

## 4. Permanent fix
Set the env vars at the top of `main()`:
```rust
fn main() -> Result<()> {
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    std::env::set_var("GDK_BACKEND", "x11");
    // ...
}
```

## Diagnostic checklist
- Window opens, content white, no errors → version skew (step 2)
- Window opens, content white, GBM errors → DMABUF (step 3)
- "window handle kind is not supported" → version skew (step 2)
- Build fails on libwebkit2gtk symbols → reinstall `webkit2gtk-4.1` via pacman