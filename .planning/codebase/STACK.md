# Technology Stack

**Analysis Date:** 2026-04-28

## Languages

**Primary:**
- TypeScript 5.3 - Frontend application code, all React components and state management (`src/`)
- Rust 2021 edition - Backend/desktop runtime, image processing, ONNX inference (`src-tauri/`)

**Secondary:**
- JavaScript (ESNext) - Vite/PostCSS/Tailwind config files

## Runtime

**Environment:**
- Node.js >= 18 (development build, set to 22 in CI)
- Rust >= 1.70 via rustup (compiled to native binary for target platform)

**Package Manager:**
- npm (CI uses `npm ci`, lockfile is `package-lock.json`)
- Note: CLAUDE.md references pnpm but the project actually uses npm in both `package.json` scripts and CI workflows.

## Frameworks

**Core:**
- React 18 - UI framework (`src/main.tsx`, all components)
- Tauri v2 (2.3) - Desktop application shell (`src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`)
- Vite 5 - Frontend build tool and dev server (`vite.config.ts`)

**UI Components:**
- Radix UI - Primitives: AlertDialog, Dialog, DropdownMenu, Label, Progress, ScrollArea, Select, Slider, Switch, Tooltip
- shadcn/ui - Higher-level component library built on Radix (`src/components/ui/`, `components.json`)
- Tailwind CSS 3 - Utility-first styling (`tailwind.config.js`, `src/index.css`)
- CVA (class-variance-authority) - Component variant management
- Lucide React - Icon library

**Canvas/Rendering:**
- Konva 10 / react-konva 18 - Canvas rendering for preview and export (`src/components/konva/mattingEngine.ts`)
- react-dropzone - Drag-and-drop file upload

**State Management:**
- Zustand 4 - Lightweight state management with persist middleware (`src/store/useStore.ts`)

**AI Inference (Rust):**
- ORT (ONNX Runtime for Rust) 2.0.0-rc.12 - ONNX model inference with `download-binaries` and `ndarray` features
- ndarray 0.17 - N-dimensional array computation for tensor operations
- BiRefNet model - Bilateral Reference Network for image matting

**Image Processing (Rust):**
- image 0.25 - Rust image library with PNG, JPEG, WebP support and rayon parallelism

**HTTP Client (Rust):**
- reqwest 0.12 - HTTP client with streaming support for model downloads

## Key Dependencies

**Critical:**
- `@tauri-apps/api` - Frontend IPC bridge to invoke Rust commands
- `@tauri-apps/plugin-dialog` - Native file/directory picker dialogs
- `@tauri-apps/plugin-fs` - Native filesystem access
- `@tauri-apps/plugin-shell` - Shell command execution
- `ort` (ONNX Runtime) - Core AI inference engine, ~900 MB model download
- `image` crate - Image decoding, encoding, resizing, compositing
- `reqwest` crate - Streaming model downloads with progress events

**Infrastructure:**
- `serde` / `serde_json` - Serialization for Tauri IPC between Rust and TypeScript
- `tokio` - Async runtime for Rust (multi-threaded, file I/O)
- `dotenvy` - .env file loading for runtime configuration
- `env_logger` - Rust logging framework
- `base64` - Base64 encoding for data URL generation in both frontend and backend
- `zustand/middleware` (persist) - LocalStorage-based state persistence
- `tailwind-merge` / `clsx` - CN utility for class name merging

## Configuration

**Environment:**
- `.env` file at `src-tauri/.env` (loaded at runtime, gitignored)
- Environment variables at compile time (via Cargo `env!()` / `option_env!()`):
  - `MODEL_URL` - Override model download URL (default: ModelScope)
  - `MODEL_FILENAME` - Override model filename (default: `birefnet.onnx`)
- Runtime environment variables (via `std::env::var()`):
  - `MODEL_URL` - Override model download source
  - `MODEL_FILENAME` - Override model filename

**Build/Dev Config:**
- `vite.config.ts`: Port 1420, strict port, `@` path alias to `./src`
- `tsconfig.json`: ES2020 target, ESNext modules, bundler resolution, react-jsx, strict mode
- `tsconfig.node.json`: Composite mode for Vite config
- `tailwind.config.js`: Custom CSS variable theme, animate plugin, dark mode via class
- `postcss.config.js`: Tailwind CSS + Autoprefixer
- `components.json`: shadcn/ui config (base color: slate, CSS variables, non-RSC)

## Platform Requirements

**Development:**
- macOS 10.13+ / Windows 10+ / Linux with webkit2gtk
- Node.js >= 18 (22 in CI)
- Rust >= 1.70 with target toolchains
- Linux: libgtk-3-dev, libwebkit2gtk-4.1-dev, librsvg2-dev, patchelf

**Production:**
- Standalone native binary per platform (macOS .dmg, Windows .exe/.msi, Linux .AppImage/.deb)
- Built via Tauri v2 framework
- macOS requires Developer ID signing and notarization via Apple API

---

*Stack analysis: 2026-04-28*
