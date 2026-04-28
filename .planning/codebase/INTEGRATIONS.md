# External Integrations

**Analysis Date:** 2026-04-28

## APIs & External Services

**AI Model Download Sources (3 sources):**
- **ModelScope (default)** - Primary model source for Chinese users, direct access without VPN
  - URL: `https://modelscope.cn/models/onnx-community/BiRefNet-ONNX/resolve/main/onnx/model.onnx`
  - Implementation: `src-tauri/src/commands/download.rs` (uses reqwest with custom User-Agent and Referer headers for ModelScope compatibility)
  - Fallback CDN in CI: `https://mocdn.mopng.cn/models/model.onnx`

- **HuggingFace** - Secondary source for international users
  - URL: `https://huggingface.co/onnx-community/BiRefNet-ONNX/resolve/main/onnx/model.onnx`
  - Implementation: `src-tauri/src/commands/download.rs` and `src-tauri/src/models/birefnet.rs`

- **HF Mirror** - HuggingFace Chinese mirror for users behind GFW
  - URL: `https://hf-mirror.com/onnx-community/BiRefNet-ONNX/resolve/main/onnx/model.onnx`
  - Implementation: `src-tauri/src/commands/download.rs` and `src-tauri/src/models/birefnet.rs`

**ONNX Runtime Binary (CI only):**
- Downloaded from GitHub releases for macOS Intel cross-compilation
  - URL: `https://github.com/microsoft/onnxruntime/releases/download/v1.18.1/onnxruntime-osx-universal2-1.18.1.tgz`
  - Implementation: `.github/workflows/release.yml` (macOS Intel job)
  - Dynamic linking with `ORT_PREFER_DYNAMIC_LINK=1`

## Data Storage

**Databases:**
- Not applicable. No external databases. All image processing is file-based.

**File Storage:**
- Local filesystem only
- Output directory: `~/Downloads/mopng_output/` (default, configurable via native dialog)
- Model storage: `{app_data_dir}/models/` (Tauri's platform-specific app data directory)
  - macOS: `~/Library/Application Support/cn.mopng.desktop/models/`

**Caching:**
- Zustand persist middleware uses `localStorage` via Tauri's WebView for app settings persistence (`src/store/useStore.ts`)
- Model download supports resume from partial download (temp file with `.tmp` extension) (`src-tauri/src/commands/download.rs`)
- Image thumbnails generated on-demand, not cached to disk

## Authentication & Identity

**Auth Provider:**
- None. The application is fully local with no user authentication.
- Apple Developer ID signing is CI-only for macOS builds, not a runtime auth.

## Monitoring & Observability

**Error Tracking:**
- None. No Sentry, Datadog, or similar service integrated.
- Rust backend uses `log` crate with `env_logger` outputting to stdout/stderr
- Console warnings for non-critical frontend errors

**Logs:**
- Rust: `env_logger` with default level `info`, configurable via `RUST_LOG` environment variable
  - Initialized in `src-tauri/src/main.rs` with `.default_filter_or("info")`
- Frontend: `console.warn` for non-critical errors during model checking/download

## CI/CD & Deployment

**Hosting:**
- GitHub releases (binaries distributed via GitHub)
- URLs to download: `https://github.com/jkin8010/mopng-desktop/releases`

**CI Pipeline:**
- GitHub Actions with two workflows in `.github/workflows/`:
  - `build.yml`: CI on push/PR to main/master branches (frontend build + Rust compile check on Linux)
  - `release.yml`: Full release pipeline triggered by `v*.*.*` tags

**Release pipeline includes:**
- Matrix build for 4 targets: macOS Apple Silicon, macOS Intel, Linux, Windows
- macOS Developer ID code signing and notarization via Apple API:
  - Signing identity: `Developer ID Application: Jian Jin (K75279AUL7)`
  - Uses `tauri-apps/tauri-action@v0` for Tauri build
  - Requires secrets: `MACOS_CERTIFICATE`, `MACOS_CERTIFICATE_PWD`, `KEYCHAIN_PWD`, `APPLE_API_KEY_ID`, `APPLE_API_ISSUER_ID`, `APPLE_API_KEY`
  - Notarization via `xcrun notarytool submit` with stapling
- CI-level model URL override via GitHub Secrets:
  - `MODEL_URL` (default: `https://mocdn.mopng.cn/models/`)
  - `MODEL_FILENAME` (default: `model.onnx`)

## Environment Configuration

**Required env vars:**
- None at runtime (the app works with defaults)
- Optional overrides:
  - `MODEL_URL` - Custom model download URL
  - `MODEL_FILENAME` - Custom model filename

**Secrets location:**
- CI secrets managed via GitHub Secrets (not stored in repository)
- macOS signing certificates encoded as base64 environment variables
- No runtime secrets or API keys required

## Webhooks & Callbacks

**Incoming:**
- None. The application has no server component.

**Outgoing:**
- None. All operations are local filesystem-based.

## External Dependencies in Frontend

**Runtime:**
- `@tauri-apps/api` (v2) - IPC communication with Rust backend via `invoke()`, event listeners via `listen()`, file URL conversion via `convertFileSrc()`
- `@tauri-apps/plugin-dialog` - File open/save dialogs, directory picker
- `@tauri-apps/plugin-fs` - Read file bytes from system (used in `useTauri.ts` for thumbnail generation)
- `@tauri-apps/plugin-shell` - Shell command execution

---

*Integration audit: 2026-04-28*
