# Coding Conventions

**Analysis Date:** 2026-04-28

## Naming Patterns

**Files:**
- PascalCase for React components: `ControlPanel.tsx`, `PreviewCanvas.tsx`, `ThumbnailList.tsx`, `ModelDialog.tsx`
- camelCase for utility/hook files: `useStore.ts`, `useTauri.ts`, `utils.ts`, `id.ts`
- Lowercase for Konva engine files: `mattingEngine.ts`, `types.ts`
- Lowercase for UI primitive files: `button.tsx`, `dialog.tsx`, `select.tsx`, `slider.tsx`

**Functions:**
- camelCase for all function names: `handleFiles`, `handleDrop`, `applyBackground`, `generateId`, `createMattingEngine`, `cn`
- React components use PascalCase and are declared as `export function Foo()` (not arrow functions, not default exports, except `App.tsx`)
- Event handlers prefixed with `handle` (e.g., `handleModeChange`, `handleBgTypeChange`, `handleProcess`, `handleExport`)
- Pointer events use `on` prefix naming within callbacks (e.g., `onPointerDown`, `onPointerMove`)
- Helper functions defined as module-level named functions (e.g., `formatFileSize()`, `formatBytes()`, `formatSpeed()`, `angleToCoords()`, `coordsToAngle()`)

**Variables:**
- camelCase for all variables: `engineRef`, `selectedTask`, `currentSettings`
- `ref` postfix for refs created via `useRef()`: `containerRef`, `konvaHostRef`, `engineRef`, `svgRef`
- Boolean state variables often prefixed with descriptive verbs: `isProcessing`, `engineReady`, `showOriginal`, `aspectLocked`, `dragging`
- Short but descriptive names preferred: `w`, `h`, `s` for widths/height/settings in local scope

**Types:**
- PascalCase for interfaces and type aliases: `MattingTask`, `MattingSettings`, `MattingResult`, `MattingEngineApi`, `AppSettings`, `ModelStatus`, `ModelInfo`
- Type unions with PascalCase: `MattingMode`, `OutputFormat`, `BgType`, `GradientType`, `SizeTemplateId`
- Props interfaces named `{ComponentName}Props`: `PreviewCanvasProps`, `ControlPanelProps`, `TaskBarProps`, `ThumbnailListProps`
- Event payload interfaces prefixed descriptively: `DownloadProgressEvent`, `ModelCompleteEvent`

**Constants:**
- UPPER_SNAKE_CASE for module-level constants: `MIN_ZOOM`, `MAX_ZOOM`, `CHECKER_CELL`, `SNAP_ANGLE`, `RADIUS`, `VIEWBOX`, `MODEL_SIZE_MB`
- Default settings constants: `DEFAULT_SETTINGS`, `DEFAULT_APP_SETTINGS`

## Code Style

**Formatting:**
- No Prettier config detected; inconsistent formatting across files:
  - `mattingEngine.ts` uses 2-space indentation
  - Most UI component files use 2-space indentation
  - Some files use single quotes (`useTauri.ts`, `button.tsx`, `switch.tsx`, `input.tsx`) while most use double quotes
  - Semicolons are present in most files
  - Trailing commas are inconsistently applied (some files use them, others do not)

**Linting:**
- No ESLint configuration detected
- TypeScript strict mode enabled in `tsconfig.json` (`"strict": true`)
- `noUnusedLocals` and `noUnusedParameters` are set to `false` (lenient)

## Import Organization

**Order:**
1. React core imports (`useEffect`, `useCallback`, `useState`, `useRef`)
2. Third-party library imports (`@tauri-apps/*`, `lucide-react`, `konva`, Radix UI primitives, `zustand`, `clsx`)
3. Internal project imports via `@/` alias (`@/store`, `@/lib/utils`, `@/components/*`, `@/types`)
4. Relative imports for co-located files (`./konva/mattingEngine`, `./konva/types`)

**Pattern:**
```typescript
import { useEffect, useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Play, FolderOpen, Download } from "lucide-react";
import { useStore } from "@/store";
import { Button } from "@/components/ui/button";
import type { MattingTask } from "@/types";
```

**Path Aliases:**
- `@` maps to `./src/` (configured in `vite.config.ts` and `tsconfig.json`)

## Error Handling

**Patterns:**
- All Tauri `invoke()` calls are wrapped in `try/catch` blocks
- Non-critical errors use `console.warn()`:
  ```typescript
  console.warn("缩略图生成失败:", task.filePath, err);
  console.warn("模型检查失败:", err);
  ```
- Critical errors use `console.error()`:
  ```typescript
  console.error("读取文件失败:", path, e);
  ```
- Error state propagates to UI via task status: `task.status === "error"` with `task.error` string
- Some errors are coerced to string: `error: String(error)`, `error: String(e)`
- Other errors use optional chaining: `err?.message || ""`
- Tauri event errors have catch-all handlers: `.catch(() => { /* ignore */ })`
- Canvas context operations often use null-check with early return:
  ```typescript
  const ctx = c.getContext("2d");
  if (!ctx) return c;
  ```
- Global composite operations in `mattingEngine.ts` return early on null context

**Error State Management:**
- Model download errors set `modelStatus.error` and `modelStatus.downloading: false`
- Processing errors update task status to `"error"` with error message
- Image loading errors are handled via `onerror` callbacks
- `finally` blocks used to reset processing state

## Logging

**Framework:** `console` only (no logging library)

**Patterns:**
- `console.warn()` for non-critical failures (model init failure, thumbnail generation failure, directory selection failure)
- `console.error()` for file read failures and task-level failures
- `console.log()` for export debugging with `[prefix]` convention: `[export]`, `[TaskBar export]`, `[PreviewCanvas]`
- Some `.catch(() => { /* ignore */ })` patterns for expected/recoverable failures

## Comments

**Language:**
- Chinese for UI-related comments (labels, tooltips, user-facing text explanations)
- English for technical implementation comments (algorithm, logic, edge cases)
- Mixing occurs freely based on developer preference

**When to Comment:**
- Complex algorithms and rendering logic: matting engine, canvas compositing, gradient calculations
- Component initialization and lifecycle intent
- Platform-specific workarounds (macOS window padding for traffic light buttons)
- Tauri event setup and teardown

**JSDoc/TSDoc:**
- Not detected. No JSDoc or TSDoc annotations anywhere in the codebase.
- Inline comments used instead of formal documentation annotations.

**Comment examples:**
```typescript
// macOS: 左侧留出 ~80px 安全区（红绿灯按钮）
// Windows/Linux: 右侧留出 ~140px 安全区（最小化/最大化/关闭按钮）
```

## Function Design

**Size:** Functions range from small single-purpose handlers (5-15 lines) to large orchestration functions (50+ lines in `ControlPanel.handleExport`, `useTauri.startProcessing`). The Konva engine `createMattingEngine` is a large factory function (~620 lines) that builds the entire rendering context.

**Parameters:**
- React components accept a single `Props` interface parameter
- Event handlers accept typed event parameters (e.g., `React.DragEvent`, `React.PointerEvent`, `React.ChangeEvent`)
- Helper functions use simple typed parameters (`(bytes: number): string`)
- Engine factory accepts positional parameters with an options bag as the last parameter

**Return Values:**
- React components return JSX (conditional rendering for empty/loading/error states)
- Helper utilities return primitives (string, number, boolean)
- The matting engine factory returns `{ api: KonvaEngineApi; destroy: () => void }`
- Custom hooks return an object of methods: `return { selectFiles, selectOutputFolder, startProcessing, ... }`

## Module Design

**Exports:**
- **Named exports for all components:** `export function ControlPanel(...)`, `export function PreviewCanvas(...)`
- **Default export only for `App.tsx`** (top-level app component)
- **Named exports for UI primitives:** `export { Button, buttonVariants }`, `export { Slider }`, `export { Input }`
- **Named exports for helper functions:** `export function cn(...)`, `export function generateId()`, `export function angleToCoords(...)`

```typescript
// Preferred component export pattern
export function ControlPanel({ onOpenSettings }: ControlPanelProps) { ... }

// Exception
export default App;
```

**Barrel Files:**
- Simple re-exports via index files: `src/store/index.ts` exports `useStore` from `./useStore`
- No comprehensive barrel file for all components (imports use direct paths from `@/components/ComponentName`)
- Types are imported directly from `@/types` (single barrel file)

## React Patterns

**State Management:**
- Global state via Zustand store (`src/store/useStore.ts`), with `persist` middleware for localStorage
- Component-local state via `useState()` for UI concerns (dialogs, tabs, zoom)
- Selector pattern: `const currentSettings = useStore((s) => s.currentSettings)`

**Effect Management:**
- `useEffect` for initialization/async data loading (model checks, event listeners)
- `useEffect` cleanup returns disposer functions (event unlisteners, engine destroy)
- `useCallback` for all event handlers and functions passed as prop dependencies
- `useRef` for DOM references and mutable state that shouldn't trigger re-renders (drag state, engine references)

**Conditional Rendering:**
- Loading state: `if (!initialized) return <Loader />`
- Empty state: `if (!task) return <EmptyState />`
- Error overlay: `{task.status === "error" && <div>...</div>}`
- Conditional sections within JSX: `{selectedTemplateId === "custom" && (...)}`

## CSS/Styling Conventions

**Framework:** Tailwind CSS utility classes exclusively (no CSS modules or styled-components)

**Patterns:**
- `cn()` utility from `@/lib/utils` is used for conditional class merging (uses `clsx` + `tailwind-merge`)
- HSL CSS variables for theming (light/dark mode via `.dark` class)
- Radix UI animations via Tailwind CSS data-state selectors (e.g., `data-[state=open]:animate-in`)
- Custom CSS keyframes in `src/index.css` for specialized animations (drop pulse)
- Custom scrollbar styles in `index.css`
- Inline `style` for dynamic values (e.g., `style={{ width: \`${pct}%\` }}`)

## UI Component Conventions

**Radix UI Primitives:**
- Wrapped in custom component files in `src/components/ui/`
- All use `React.forwardRef` pattern
- All have `.displayName` set
- All use `cn()` for className merging
- Pattern: import primitive, create forwarded component, apply Tailwind styling via `cn()`, set displayName, export

**Icon Usage:** `lucide-react` is the icon library. Icons imported individually (not as default): `import { ZoomIn, ZoomOut, RotateCcw } from "lucide-react"`

---

*Convention analysis: 2026-04-28*
