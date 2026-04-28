# Codebase Structure

**Analysis Date:** 2026-04-28

## Directory Layout

```
mopng-desktop/
├── src/                               # Frontend source (React + TypeScript)
│   ├── main.tsx                       # React entry point
│   ├── App.tsx                        # Root component: layout orchestration, model init, file drop handler
│   ├── index.css                      # Tailwind directives + CSS variables + custom scrollbar + checkerboard
│   ├── types/
│   │   └── index.ts                   # All TypeScript types/enums/constants (MattingTask, MattingSettings, etc.)
│   ├── store/
│   │   ├── index.ts                   # Barrel export
│   │   └── useStore.ts                # Zustand single store with persist middleware
│   ├── hooks/
│   │   └── useTauri.ts                # Hook wrapping Tauri invoke calls (file pick, process, save, etc.)
│   ├── lib/
│   │   ├── utils.ts                   # cn() utility (clsx + tailwind-merge)
│   │   └── id.ts                      # generateId() using timestamp+random
│   └── components/
│       ├── TitleBar.tsx               # Draggable title bar with platform-aware safety padding
│       ├── DropZone.tsx               # Drag-over overlay with pulse animation
│       ├── ThumbnailList.tsx          # Left sidebar: image thumbnails, file adder, selection, delete
│       ├── PreviewCanvas.tsx          # Center: Konva canvas, zoom controls, compare mode, resize observer
│       ├── TaskBar.tsx                # Bottom bar: task counts, open/export, clear actions
│       ├── ControlPanel.tsx           # Right sidebar: model select, matting mode, format, bg, size template, actions
│       ├── ModelDialog.tsx            # Model download dialog with source picker, progress, cancel
│       ├── SettingsDialog.tsx         # App settings dialog (output dir, auto-export, defaults, model mgmt)
│       ├── BatchProgress.tsx          # Batch processing progress bar + error list
│       ├── GradientAnglePicker.tsx    # UI component for gradient angle input
│       ├── konva/
│       │   ├── types.ts               # KonvaEngineApi interface definition
│       │   └── mattingEngine.ts       # Konva engine factory: stage, layers, compositing, zoom/pan, export
│       └── ui/                        # shadcn-style Radix UI primitives
│           ├── button.tsx
│           ├── dialog.tsx
│           ├── input.tsx
│           ├── label.tsx
│           ├── progress.tsx
│           ├── scroll-area.tsx
│           ├── scrub-input.tsx
│           ├── select.tsx
│           ├── slider.tsx
│           ├── switch.tsx
│           └── tooltip.tsx
├── src-tauri/                         # Rust backend (Tauri v2)
│   ├── Cargo.toml                     # Rust dependencies
│   ├── tauri.conf.json                # Tauri app/bundle/build config
│   ├── entitlements.plist             # macOS sandbox entitlements for notarization
│   ├── build.rs                       # Tauri build script
│   ├── .env.example                   # MODEL_URL / MODEL_FILENAME template
│   ├── capabilities/
│   │   └── default.json               # Tauri v2 capability permissions
│   ├── gen/schemas/                   # Auto-generated Tauri schemas
│   ├── icons/                         # App icons (PNG, ICO, ICNS, iOS, Android)
│   └── src/
│       ├── main.rs                    # Tauri entry: plugin init, command registration, .env loading
│       ├── commands/
│       │   ├── mod.rs                 # Tauri commands: process_image, generate_thumbnail, open_in_folder,
│       │   │                          #   export_image_dialog, save_data_url + helper functions
│       │   │                          #   (apply_mask, apply_bg_type, composite_on_solid, resize_to_target, etc.)
│       │   ├── download.rs            # Download commands: check_model, download_model, cancel_download,
│       │   │                          #   get_model_sources, get_model_dir + streaming download logic
│       │   └── file.rs                # File commands: read_image_file, read_file_as_data_url,
│       │                              #   pick_files, select_output_dir
│       └── models/
│           ├── mod.rs                 # MattingModel trait, MattingSettings/ProcessParams/ProcessResult structs,
│           │                          #   init_model / is_model_loaded / list_models commands
│           ├── birefnet.rs            # BirefnetModel: MattingModel impl, ONNX inference, bilinear upscale, tests
│           └── registry.rs            # Global model registry: Lazy<Mutex<RegistryInner>>, model lifecycle
├── public/
│   └── logo.png                       # App logo used in title bar and favicon
├── index.html                         # HTML entry point
├── package.json                       # Node.js dependencies and scripts
├── vite.config.ts                     # Vite config: React plugin, @ alias, port 1420
├── tsconfig.json                      # TypeScript config: @ path alias, strict mode
├── tsconfig.node.json                 # TypeScript config for Node/Vite context
├── tailwind.config.js                 # Tailwind CSS with shadcn color tokens and animations
├── postcss.config.js                  # PostCSS with Tailwind + Autoprefixer
├── components.json                    # shadcn/ui configuration
├── .github/workflows/
│   ├── build.yml                      # PR verification build workflow
│   └── release.yml                    # macOS signing + notarization release workflow
├── .gitignore
├── CLAUDE.md
└── README.md
```

## Directory Purposes

**`src/`:**
- Purpose: All frontend source code
- Contains: React components, Zustand store, TypeScript types, hooks, utility libs, Konva engine
- Key files: `main.tsx` (entry), `App.tsx` (root orchestrator), `store/useStore.ts` (state)

**`src/types/`:**
- Purpose: Single source of truth for all TypeScript type definitions and constants
- Contains: `MattingTask`, `MattingSettings`, `MattingResult`, `AppSettings`, `ModelInfo`, `ModelStatus`, `SizeTemplate`, `SIZE_TEMPLATES`, `DEFAULT_SETTINGS`, `DEFAULT_APP_SETTINGS`, enums (`MattingMode`, `OutputFormat`, `BgType`, `SizeTemplateId`)
- Key files: `index.ts`

**`src/store/`:**
- Purpose: Global state management
- Contains: Zustand store with `persist` middleware
- Key files: `useStore.ts` (all state + actions), `index.ts` (re-export)

**`src/hooks/`:**
- Purpose: Reusable React hooks wrapping Tauri IPC
- Contains: `useTauri.ts` (file pick, process, save, open folder wrapper)

**`src/lib/`:**
- Purpose: Pure utility functions
- Contains: `cn()` (class merging), `generateId()` (unique IDs)

**`src/components/`:**
- Purpose: All React UI components
- Contains: App-level components (TitleBar, DropZone, ThumbnailList, PreviewCanvas, TaskBar, ControlPanel, ModelDialog, SettingsDialog, BatchProgress, GradientAnglePicker)

**`src/components/konva/`:**
- Purpose: Konva canvas rendering engine, fully self-contained
- Contains: `mattingEngine.ts` (factory), `types.ts` (API interface)
- Structure: Factory pattern -- `createMattingEngine(container, w, h, img, options)` returns `{ api, destroy }`

**`src/components/ui/`:**
- Purpose: Low-level UI primitives based on Radix UI + CVA + Tailwind
- Contains: 11 components (button, dialog, input, label, progress, scroll-area, scrub-input, select, slider, switch, tooltip)
- Pattern: Each file exports one component using Radix primitives with Tailwind styling

**`src-tauri/src/commands/`:**
- Purpose: All Tauri command handlers for image processing, file operations, and model download
- Contains: `mod.rs` (image processing core), `download.rs` (model download with progress), `file.rs` (file I/O commands)
- Pattern: Public functions re-exported via `pub use self::*` in mod.rs

**`src-tauri/src/models/`:**
- Purpose: ONNX model abstraction layer
- Contains: `mod.rs` (trait + data structs + model lifecycle commands), `birefnet.rs` (BiRefNet concrete impl), `registry.rs` (global registry singleton)

**`.github/workflows/`:**
- Purpose: CI/CD automation
- Contains: `build.yml` (PR verification), `release.yml` (macOS production signing + notarization)

## Key File Locations

**Entry Points:**
- `src/main.tsx`: React mount point, renders `<App />`
- `src/App.tsx`: Root component, layout and initialization orchestration
- `src-tauri/src/main.rs`: Tauri entry, command registration, plugin init
- `index.html`: HTML shell with `#root` div

**Configuration:**
- `package.json`: Node dependencies, scripts (dev, build, tauri)
- `vite.config.ts`: Vite build config, @ path alias, React plugin, port 1420
- `tsconfig.json`: TypeScript config with `@/*` path mapping
- `tailwind.config.js`: Tailwind theme with shadcn color tokens
- `src-tauri/tauri.conf.json`: Tauri window/app/bundle/security config (identifier `cn.mopng.desktop`)
- `src-tauri/Cargo.toml`: Rust dependencies (ort, image, ndarray, tauri, etc.)
- `src-tauri/capabilities/default.json`: Tauri v2 capability permissions
- `components.json`: shadcn/ui configuration

**Core Logic:**
- `src/App.tsx`: Root layout, file drop handling, model init, task creation
- `src/store/useStore.ts`: Application state and actions
- `src/types/index.ts`: All type definitions and constants
- `src/components/PreviewCanvas.tsx`: Konva canvas lifecycle, engine init, background/zoom sync
- `src/components/ControlPanel.tsx`: Settings UI, process/export triggers
- `src/components/konva/mattingEngine.ts`: Full Konva composition engine (mask, bg types, gradient, zoom/pan, export)
- `src-tauri/src/commands/mod.rs`: Image processing pipeline (load -> infer -> mask -> resize -> save)
- `src-tauri/src/models/birefnet.rs`: ONNX model init and inference
- `src-tauri/src/models/registry.rs`: Model registry singleton
- `src-tauri/src/commands/download.rs`: Model download with resume and progress events

**Testing:**
- `src-tauri/src/models/birefnet.rs`: Embedded unit tests (test_model_loading) -- skipped if model file not present

## Naming Conventions

**Files:**
- React components: PascalCase (`PreviewCanvas.tsx`, `ControlPanel.tsx`, `ModelDialog.tsx`)
- Rust modules: snake_case (`birefnet.rs`, `mod.rs`, `commands/`, `models/`)
- Utilities: camelCase (`utils.ts`, `id.ts`, `useStore.ts`, `mattingEngine.ts`)
- Config files: kebab-case (`vite.config.ts`, `tailwind.config.js`, `tauri.conf.json`)

**Directories:**
- All lowercase (src, components, hooks, lib, store, types, konva, ui, commands, models)
- Single-responsibility directories (e.g., `store/` = state, `hooks/` = hooks, `types/` = types)

## Where to Add New Code

**New Feature (Frontend):**
- Primary code: Add component file in `src/components/` (PascalCase.tsx)
- If feature involves new state: Add state + actions in `src/store/useStore.ts`
- If feature involves new types: Add types in `src/types/index.ts`
- If feature needs new IPC command: Add Rust command in `src-tauri/src/commands/mod.rs` (or new file under `commands/`) + register in `src-tauri/src/main.rs`
- Tests: Add embedded `#[cfg(test)] mod tests` in the relevant Rust source file

**New Component/Module:**
- App-level component: `src/components/MyComponent.tsx`
- UI primitive: `src/components/ui/my-component.tsx` using Radix + CVA + Tailwind pattern
- Konva-related engine logic: Add to `src/components/konva/` using the factory module pattern

**New Rust Command:**
- Implementation: `src-tauri/src/commands/mod.rs` (or new file `src-tauri/src/commands/my_command.rs` with `pub mod my_command;` in `mod.rs`)
- Registration: Add function to `generate_handler![]` in `src-tauri/src/main.rs`

**New Model Implementation:**
- Model struct: `src-tauri/src/models/my_model.rs` implementing `MattingModel` trait
- Descriptor: Add `pub fn descriptor() -> ModelDescriptor` and register in `registry.rs`
- Factory: Add `"my_model" => Ok(Box::new(MyModel::new()))` in `create_model()`

**Utilities:**
- Shared helpers: `src/lib/` for pure functions (no React or Tauri dependency)

**Hooks:**
- Reusable Tauri-wrapping hooks: `src/hooks/` following the `useTauri.ts` pattern

## Special Directories

**`src/components/ui/`:**
- Purpose: Low-level UI primitives built on Radix UI
- Generated: Not auto-generated (hand-crafted shadcn-style)
- Committed: Yes

**`src-tauri/gen/`:**
- Purpose: Auto-generated Tauri schemas by `tauri build` / `tauri dev`
- Generated: Yes (Tauri tooling)
- Committed: Yes (checked into repo)

**`public/`:**
- Purpose: Static assets served by Vite dev server
- Contains: `logo.png` (app icon for favicon and title bar)
- Committed: Yes

---

*Structure analysis: 2026-04-28*
