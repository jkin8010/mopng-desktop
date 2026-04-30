# CLAUDE.md

## 语言偏好

- 使用简体中文进行所有对话交流
- 代码、变量名、commit 信息可使用英文

## 项目概述

**Mopng Desktop** — 基于 Tauri v2 的桌面端智能抠图应用，使用 ONNX 模型进行图像背景移除。

## 技术栈

| 层 | 技术 |
|---|---|
| 前端 | React + TypeScript + Vite |
| 画布渲染 | Konva (react-konva) |
| UI 组件 | Radix UI + Tailwind CSS + CVA |
| 桌面壳 | Tauri v2 (Rust) |
| AI 推理 | ONNX Runtime (BiRefNet 模型) |
| 模型来源 | ModelScope |
| 包管理 | pnpm |

## 项目结构

```
src/
├── App.tsx                  # 主应用入口
├── components/
│   ├── ControlPanel.tsx     # 控制面板（尺寸模板、背景色、导出选项）
│   ├── PreviewCanvas.tsx    # Konva 画布预览
│   ├── ModelDialog.tsx      # 模型下载/选择对话框
│   ├── TaskBar.tsx          # 工具栏
│   ├── ThumbnailList.tsx    # 缩略图列表
│   ├── BatchProgress.tsx    # 批量处理进度
│   ├── DropZone.tsx         # 拖拽上传区域
│   └── konva/
│       └── mattingEngine.ts # 抠图渲染引擎
├── store/useStore.ts        # Zustand 全局状态
└── types/index.ts           # TypeScript 类型定义

src-tauri/src/
├── main.rs                  # Tauri 入口
├── commands/                # Tauri 命令层
│   ├── mod.rs
│   ├── download.rs          # 模型下载
│   └── file.rs              # 文件操作
└── models/                  # Rust 数据模型
    ├── mod.rs
    └── birefnet.rs          # BiRefNet 推理
```

## 开发命令

```bash
pnpm dev              # Vite 开发服务器
pnpm tauri dev        # Tauri 开发模式（含 Rust 后端）
pnpm tauri build      # 生产构建
```

## 用户习惯与偏好

- 喜欢快速迭代，直接修改代码并验证，偏好视觉/UI 层面的确认
- 可组合的 UI 组件优先，保持组件职责单一
- 尺寸模板功能（含自定义尺寸、等比锁定）是近期完成的核心功能，类似参数暴露模式可作为参考
- Git 工作流：在 develop 分支开发，合并到 main，打 tag 发 release
- CI/CD 通过 GitHub Actions 进行 macOS 签名公证构建

<!-- GSD:project-start source:PROJECT.md -->
## Project

**Mopng Desktop — 通用智能抠图平台**

Mopng Desktop 是一款基于 ONNX 模型的桌面端智能抠图应用，支持多种 AI 模型进行图像背景移除。通过插件化架构，用户使用统一的界面和操作习惯即可切换不同抠图模型，并支持谷子/徽章制作、精确测量尺标、工业化批量生产等周边应用场景。

**Core Value:** **一键抠图，多模型可选，统一体验** — 用户不需要了解 AI 模型差异，拖入图片即可获得高质量抠图结果，模型选择和参数调整由平台自动管理。

### Constraints

- **技术栈：** Rust (Tauri v2) + React/TypeScript + ONNX Runtime（不可随意更换）
- **模型格式：** ONNX（需支持 FP32/FP16 精度选择）
- **桌面平台：** macOS 优先，Windows 次之
- **模型来源：** 优先国内可访问（ModelScope），兼容 HuggingFace
- **性能：** 单次推理 < 5 秒（1024px 输入），批量处理可后台执行
- **包体积：** 应用包 < 100MB（不含模型），模型独立下载
<!-- GSD:project-end -->

<!-- GSD:stack-start source:codebase/STACK.md -->
## Technology Stack

## Languages
- TypeScript 5.3 - Frontend application code, all React components and state management (`src/`)
- Rust 2021 edition - Backend/desktop runtime, image processing, ONNX inference (`src-tauri/`)
- JavaScript (ESNext) - Vite/PostCSS/Tailwind config files
## Runtime
- Node.js >= 18 (development build, set to 22 in CI)
- Rust >= 1.70 via rustup (compiled to native binary for target platform)
- npm (CI uses `npm ci`, lockfile is `package-lock.json`)
- Note: CLAUDE.md references pnpm but the project actually uses npm in both `package.json` scripts and CI workflows.
## Frameworks
- React 18 - UI framework (`src/main.tsx`, all components)
- Tauri v2 (2.3) - Desktop application shell (`src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`)
- Vite 5 - Frontend build tool and dev server (`vite.config.ts`)
- Radix UI - Primitives: AlertDialog, Dialog, DropdownMenu, Label, Progress, ScrollArea, Select, Slider, Switch, Tooltip
- shadcn/ui - Higher-level component library built on Radix (`src/components/ui/`, `components.json`)
- Tailwind CSS 3 - Utility-first styling (`tailwind.config.js`, `src/index.css`)
- CVA (class-variance-authority) - Component variant management
- Lucide React - Icon library
- Konva 10 / react-konva 18 - Canvas rendering for preview and export (`src/components/konva/mattingEngine.ts`)
- react-dropzone - Drag-and-drop file upload
- Zustand 4 - Lightweight state management with persist middleware (`src/store/useStore.ts`)
- ORT (ONNX Runtime for Rust) 2.0.0-rc.12 - ONNX model inference with `download-binaries` and `ndarray` features
- ndarray 0.17 - N-dimensional array computation for tensor operations
- BiRefNet model - Bilateral Reference Network for image matting
- image 0.25 - Rust image library with PNG, JPEG, WebP support and rayon parallelism
- reqwest 0.12 - HTTP client with streaming support for model downloads
## Key Dependencies
- `@tauri-apps/api` - Frontend IPC bridge to invoke Rust commands
- `@tauri-apps/plugin-dialog` - Native file/directory picker dialogs
- `@tauri-apps/plugin-fs` - Native filesystem access
- `@tauri-apps/plugin-shell` - Shell command execution
- `ort` (ONNX Runtime) - Core AI inference engine, ~900 MB model download
- `image` crate - Image decoding, encoding, resizing, compositing
- `reqwest` crate - Streaming model downloads with progress events
- `serde` / `serde_json` - Serialization for Tauri IPC between Rust and TypeScript
- `tokio` - Async runtime for Rust (multi-threaded, file I/O)
- `dotenvy` - .env file loading for runtime configuration
- `env_logger` - Rust logging framework
- `base64` - Base64 encoding for data URL generation in both frontend and backend
- `zustand/middleware` (persist) - LocalStorage-based state persistence
- `tailwind-merge` / `clsx` - CN utility for class name merging
## Configuration
- `.env` file at `src-tauri/.env` (loaded at runtime, gitignored)
- Environment variables at compile time (via Cargo `env!()` / `option_env!()`):
- Runtime environment variables (via `std::env::var()`):
- `vite.config.ts`: Port 1420, strict port, `@` path alias to `./src`
- `tsconfig.json`: ES2020 target, ESNext modules, bundler resolution, react-jsx, strict mode
- `tsconfig.node.json`: Composite mode for Vite config
- `tailwind.config.js`: Custom CSS variable theme, animate plugin, dark mode via class
- `postcss.config.js`: Tailwind CSS + Autoprefixer
- `components.json`: shadcn/ui config (base color: slate, CSS variables, non-RSC)
## Platform Requirements
- macOS 10.13+ / Windows 10+ / Linux with webkit2gtk
- Node.js >= 18 (22 in CI)
- Rust >= 1.70 with target toolchains
- Linux: libgtk-3-dev, libwebkit2gtk-4.1-dev, librsvg2-dev, patchelf
- Standalone native binary per platform (macOS .dmg, Windows .exe/.msi, Linux .AppImage/.deb)
- Built via Tauri v2 framework
- macOS requires Developer ID signing and notarization via Apple API
<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->
## Conventions

## Naming Patterns
- PascalCase for React components: `ControlPanel.tsx`, `PreviewCanvas.tsx`, `ThumbnailList.tsx`, `ModelDialog.tsx`
- camelCase for utility/hook files: `useStore.ts`, `useTauri.ts`, `utils.ts`, `id.ts`
- Lowercase for Konva engine files: `mattingEngine.ts`, `types.ts`
- Lowercase for UI primitive files: `button.tsx`, `dialog.tsx`, `select.tsx`, `slider.tsx`
- camelCase for all function names: `handleFiles`, `handleDrop`, `applyBackground`, `generateId`, `createMattingEngine`, `cn`
- React components use PascalCase and are declared as `export function Foo()` (not arrow functions, not default exports, except `App.tsx`)
- Event handlers prefixed with `handle` (e.g., `handleModeChange`, `handleBgTypeChange`, `handleProcess`, `handleExport`)
- Pointer events use `on` prefix naming within callbacks (e.g., `onPointerDown`, `onPointerMove`)
- Helper functions defined as module-level named functions (e.g., `formatFileSize()`, `formatBytes()`, `formatSpeed()`, `angleToCoords()`, `coordsToAngle()`)
- camelCase for all variables: `engineRef`, `selectedTask`, `currentSettings`
- `ref` postfix for refs created via `useRef()`: `containerRef`, `konvaHostRef`, `engineRef`, `svgRef`
- Boolean state variables often prefixed with descriptive verbs: `isProcessing`, `engineReady`, `showOriginal`, `aspectLocked`, `dragging`
- Short but descriptive names preferred: `w`, `h`, `s` for widths/height/settings in local scope
- PascalCase for interfaces and type aliases: `MattingTask`, `MattingSettings`, `MattingResult`, `MattingEngineApi`, `AppSettings`, `ModelStatus`, `ModelInfo`
- Type unions with PascalCase: `MattingMode`, `OutputFormat`, `BgType`, `GradientType`, `SizeTemplateId`
- Props interfaces named `{ComponentName}Props`: `PreviewCanvasProps`, `ControlPanelProps`, `TaskBarProps`, `ThumbnailListProps`
- Event payload interfaces prefixed descriptively: `DownloadProgressEvent`, `ModelCompleteEvent`
- UPPER_SNAKE_CASE for module-level constants: `MIN_ZOOM`, `MAX_ZOOM`, `CHECKER_CELL`, `SNAP_ANGLE`, `RADIUS`, `VIEWBOX`, `MODEL_SIZE_MB`
- Default settings constants: `DEFAULT_SETTINGS`, `DEFAULT_APP_SETTINGS`
## Code Style
- No Prettier config detected; inconsistent formatting across files:
- No ESLint configuration detected
- TypeScript strict mode enabled in `tsconfig.json` (`"strict": true`)
- `noUnusedLocals` and `noUnusedParameters` are set to `false` (lenient)
## Import Organization
- `@` maps to `./src/` (configured in `vite.config.ts` and `tsconfig.json`)
## Error Handling
- All Tauri `invoke()` calls are wrapped in `try/catch` blocks
- Non-critical errors use `console.warn()`:
- Critical errors use `console.error()`:
- Error state propagates to UI via task status: `task.status === "error"` with `task.error` string
- Some errors are coerced to string: `error: String(error)`, `error: String(e)`
- Other errors use optional chaining: `err?.message || ""`
- Tauri event errors have catch-all handlers: `.catch(() => { /* ignore */ })`
- Canvas context operations often use null-check with early return:
- Global composite operations in `mattingEngine.ts` return early on null context
- Model download errors set `modelStatus.error` and `modelStatus.downloading: false`
- Processing errors update task status to `"error"` with error message
- Image loading errors are handled via `onerror` callbacks
- `finally` blocks used to reset processing state
## Logging
- `console.warn()` for non-critical failures (model init failure, thumbnail generation failure, directory selection failure)
- `console.error()` for file read failures and task-level failures
- `console.log()` for export debugging with `[prefix]` convention: `[export]`, `[TaskBar export]`, `[PreviewCanvas]`
- Some `.catch(() => { /* ignore */ })` patterns for expected/recoverable failures
## Comments
- Chinese for UI-related comments (labels, tooltips, user-facing text explanations)
- English for technical implementation comments (algorithm, logic, edge cases)
- Mixing occurs freely based on developer preference
- Complex algorithms and rendering logic: matting engine, canvas compositing, gradient calculations
- Component initialization and lifecycle intent
- Platform-specific workarounds (macOS window padding for traffic light buttons)
- Tauri event setup and teardown
- Not detected. No JSDoc or TSDoc annotations anywhere in the codebase.
- Inline comments used instead of formal documentation annotations.
## Function Design
- React components accept a single `Props` interface parameter
- Event handlers accept typed event parameters (e.g., `React.DragEvent`, `React.PointerEvent`, `React.ChangeEvent`)
- Helper functions use simple typed parameters (`(bytes: number): string`)
- Engine factory accepts positional parameters with an options bag as the last parameter
- React components return JSX (conditional rendering for empty/loading/error states)
- Helper utilities return primitives (string, number, boolean)
- The matting engine factory returns `{ api: KonvaEngineApi; destroy: () => void }`
- Custom hooks return an object of methods: `return { selectFiles, selectOutputFolder, startProcessing, ... }`
## Module Design
- **Named exports for all components:** `export function ControlPanel(...)`, `export function PreviewCanvas(...)`
- **Default export only for `App.tsx`** (top-level app component)
- **Named exports for UI primitives:** `export { Button, buttonVariants }`, `export { Slider }`, `export { Input }`
- **Named exports for helper functions:** `export function cn(...)`, `export function generateId()`, `export function angleToCoords(...)`
- Simple re-exports via index files: `src/store/index.ts` exports `useStore` from `./useStore`
- No comprehensive barrel file for all components (imports use direct paths from `@/components/ComponentName`)
- Types are imported directly from `@/types` (single barrel file)
## React Patterns
- Global state via Zustand store (`src/store/useStore.ts`), with `persist` middleware for localStorage
- Component-local state via `useState()` for UI concerns (dialogs, tabs, zoom)
- Selector pattern: `const currentSettings = useStore((s) => s.currentSettings)`
- `useEffect` for initialization/async data loading (model checks, event listeners)
- `useEffect` cleanup returns disposer functions (event unlisteners, engine destroy)
- `useCallback` for all event handlers and functions passed as prop dependencies
- `useRef` for DOM references and mutable state that shouldn't trigger re-renders (drag state, engine references)
- Loading state: `if (!initialized) return <Loader />`
- Empty state: `if (!task) return <EmptyState />`
- Error overlay: `{task.status === "error" && <div>...</div>}`
- Conditional sections within JSX: `{selectedTemplateId === "custom" && (...)}`
## CSS/Styling Conventions
- `cn()` utility from `@/lib/utils` is used for conditional class merging (uses `clsx` + `tailwind-merge`)
- HSL CSS variables for theming (light/dark mode via `.dark` class)
- Radix UI animations via Tailwind CSS data-state selectors (e.g., `data-[state=open]:animate-in`)
- Custom CSS keyframes in `src/index.css` for specialized animations (drop pulse)
- Custom scrollbar styles in `index.css`
- Inline `style` for dynamic values (e.g., `style={{ width: \`${pct}%\` }}`)
## UI Component Conventions
- Wrapped in custom component files in `src/components/ui/`
- All use `React.forwardRef` pattern
- All have `.displayName` set
- All use `cn()` for className merging
- Pattern: import primitive, create forwarded component, apply Tailwind styling via `cn()`, set displayName, export
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->
## Architecture

## Pattern Overview
- Frontend (React + TypeScript) handles all UI rendering, state management, and Konva canvas composition
- Backend (Rust) handles image processing, ONNX model inference, file I/O, and download orchestration
- IPC boundary crossed exclusively via `@tauri-apps/api/core` `invoke()` calls -- no custom protocol
- Frontend state managed exclusively via Zustand single store, persisted to localStorage for settings
- Konva rendering engine is self-contained in a factory module, decoupled from React lifecycle via ref-based management
- Model system uses a Rust trait-based plugin pattern (`MattingModel` trait) with a global registry singleton (`Lazy<Mutex<RegistryInner>>`)
- Download system uses Tauri event emission (`app.emit()`) for real-time frontend progress updates
## Layers
- Purpose: All UI rendering, user interaction, canvas compositing, state orchestration
- Location: `src/`
- Contains: React components, hooks, store, Konva engine (factory), utility libs, type definitions
- Depends on: Tauri JS API (`@tauri-apps/api/*`), React, Konva/react-konva, Radix UI, Tailwind CSS
- Used by: User interaction (UI entry point)
- Purpose: Translates frontend `invoke()` calls into Rust command handlers; returns serialized JSON results
- Location: Auto-generated by Tauri; commands registered in `src-tauri/src/main.rs` via `generate_handler![]`
- Contains: All `#[tauri::command]` functions in `commands/` and `models/` modules
- Depends on: Tauri framework (`tauri`, `tauri-plugin-dialog`, `tauri-plugin-fs`, `tauri-plugin-shell`)
- Used by: Frontend via `invoke()` calls
- Purpose: ONNX model inference, image masking, compositing, thumbnail generation, file export
- Location: `src-tauri/src/commands/` and `src-tauri/src/models/`
- Contains: `process_image`, `generate_thumbnail`, `apply_mask`, `apply_bg_type`, `resize_to_target`, `export_image_dialog`, `save_data_url`
- Depends on: `ort` (ONNX Runtime), `image` (Rust image crate), `ndarray`, `base64`
- Used by: Invoked via Tauri IPC from frontend
- Purpose: Manage model lifecycle (init, infer, list, query sources, model file paths)
- Location: `src-tauri/src/models/registry.rs`
- Contains: Global `Lazy<Mutex<RegistryInner>>` singleton, `MattingModel` trait, model descriptor registration, `infer()` dispatch
- Depends on: `MattingModel` trait implementations (currently `BirefnetModel`)
- Used by: `commands/` modules, `models/mod.rs` commands
- Purpose: Download ONNX models from remote sources with resume support, progress events, cancellation
- Location: `src-tauri/src/commands/download.rs`
- Contains: Download via `reqwest` with chunked streaming, atomic file rename, `AtomicBool` cancellation flag, `AtomicU64` progress tracking, Tauri events (`model-download-progress`, `model-download-complete`)
- Depends on: `reqwest` with streaming, `tokio` async I/O, Tauri `Emitter`
- Used by: Frontend via `invoke("download_model")`
## Data Flow
- Single Zustand store at `src/store/useStore.ts`
- Persisted partial state (appSettings, currentSettings, activeModelId) to localStorage via `persist` middleware
- All component state flows through store selectors (`useStore((s) => s.xxx)`)
- Task lifecycle managed entirely in store: addTasks -> updateTask -> updateTaskResult
- `konvaExportFn` is a callback stored in Zustand, registered by PreviewCanvas when engine initializes, used by ControlPanel/TaskBar for export
## Key Abstractions
- Purpose: Plugin protocol for diverse matting models
- Location: `src-tauri/src/models/mod.rs`
- Methods: `id()`, `name()`, `description()`, `init()`, `is_loaded()`, `infer()`, `filename()`, `sources()`
- Implementations: `BirefnetModel` in `src-tauri/src/models/birefnet.rs`
- Registration: Descriptors registered in `RegistryInner.descriptors` vec; model factory in `create_model()` match arm
- Purpose: Abstract API for Konva canvas composition engine, decoupled from React
- Location: `src/components/konva/types.ts`
- Methods: `resizeStage()`, `getDocumentSize()`, `setDocumentSize()`, `setBackground*()`, `getExportPngDataUrl()`, `updateMask()`, zoom/pan controls
- Implementation: `createMattingEngine()` factory in `src/components/konva/mattingEngine.ts` returns `{ api, destroy }`
- Purpose: Single global state container for entire frontend
- Location: `src/store/useStore.ts`
- State slices: `tasks[]`, `currentSettings`, `appSettings`, `modelStatus`, processing flags, UI state
- Export pattern: Re-exported via `src/store/index.ts` barrel file as `useStore`
- Purpose: Predefined and custom output dimensions with aspect ratio locking
- Location: `src/types/index.ts` (types + `SIZE_TEMPLATES` array + `deriveTemplateId()`)
- Applied in Rust `resize_to_target()`: either exact resize or letterbox/pillarbox with transparent padding
- Synced to Konva engine via `engine.api.setDocumentSize()` which adjusts artboard clipRect and viewport
## Entry Points
- Location: `src/main.tsx`
- Triggers: Browser/WebView loads `index.html`
- Responsibilities: Mount React app to `#root` element with StrictMode
- Location: `src/App.tsx`
- Triggers: `main.tsx` renders `<App />`
- Responsibilities:
- Location: `src-tauri/src/main.rs`
- Triggers: Tauri runtime startup
- Responsibilities:
- Location: `src-tauri/src/models/registry.rs`
- Triggers: Static initialization of `REGISTRY` via `Lazy<Mutex<...>>`
- Responsibilities: Register model descriptors, track loaded model instance, dispatch `infer()`, provide model dir and filename lookups
## Error Handling
- Async invoke calls wrapped in try/catch -> on error set `task.status = "error"` with `String(error)` message
- Console.warn for non-critical errors (thumbnail failure, model init failure)
- UI error display: error badge on task thumbnail, toast banners on canvas
- All `#[tauri::command]` functions return `Result<T, String>` -- Tauri serializes errors as JSON strings
- `map_err(|e| format!("Human-readable error: {}", e))` wraps lower-level errors
- `spawn_blocking` for ONNX inference to avoid blocking the async IPC thread
- Mutex used for global registry state -- `poisoned` panic if lock fails
- File operations use `create_dir_all` to ensure parent paths exist
## Cross-Cutting Concerns
- Rust: `env_logger` with default filter "info" level; `log::info!`, `log::debug!`, `log::warn!` throughout
- Frontend: `console.log` / `console.warn` for debug tracing; no structured logging framework
- Image format validation: file extension filter on drop (`/\.(jpg|jpeg|png|webp|bmp|gif)$/i`)
- Size template validation: clamped to 1-10000px; `original` template uses undefined/null for passthrough
- Quality range: clamped to 10-100 in Rust `process_image`
- Settings serialized with `#[serde(rename_all = "camelCase")]` for consistent JS-Rust naming
- Not applicable (local desktop app, no external auth)
- ONNX inference runs on blocking thread pool (not blocking Tauri IPC)
- BiRefNet processes at 1024x1024 input -> bilinear upscale back to original resolution
- Preview thumbnails capped at 120px (backend) / 160px (frontend fallback)
- Canvas preview capped at 800px resolution
- Download chunked streaming with throttled progress events (every 512KB or 200ms)
- Download supports resume via `Range` header and `.tmp` file checkpoint
- Konva engine recreated on task change; mask composited once via offscreen canvas
- JPEG export flattens alpha onto white to avoid transparency incompatibility
<!-- GSD:architecture-end -->

<!-- GSD:skills-start source:skills/ -->
## Project Skills

No project skills found. Add skills to any of: `.claude/skills/`, `.agents/skills/`, `.cursor/skills/`, or `.github/skills/` with a `SKILL.md` index file.
<!-- GSD:skills-end -->

<!-- GSD:workflow-start source:GSD defaults -->
## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:
- `/gsd-quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd-debug` for investigation and bug fixing
- `/gsd-execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->

<!-- GSD:profile-start -->
## Developer Profile

> Profile not yet configured. Run `/gsd-profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->
