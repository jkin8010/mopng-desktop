# Codebase Concerns

**Analysis Date:** 2026-04-28

## Tech Debt

### Dead code: `useTauri.ts` hook with non-existent Tauri commands

- Issue: The `src/hooks/useTauri.ts` file (198 lines) contains a `startProcessing` function that invokes Tauri commands `process_image` and `get_file_info` with an old API signature (returns a string path, not the `ProcessResult` struct). It also calls `save_image` which does not exist in the Rust backend at all. This appears to be legacy code from an earlier architecture that was replaced by the `process_image` command in `src-tauri/src/commands/mod.rs`.
- Files: `src/hooks/useTauri.ts` (lines 93-152, 154-168, 169-182)
- Impact: Maintainers may mistakenly use these dead functions. The `useTauri` hook is exported but never imported anywhere in the app -- `App.tsx` and `ControlPanel.tsx` call Tauri commands directly.
- Fix approach: Remove `src/hooks/useTauri.ts` entirely, or rewrite it to use the current command signatures if it still serves a purpose.

### Duplicate export logic in ControlPanel and TaskBar

- Issue: The `handleExport` function in both `src/components/ControlPanel.tsx` (lines 254-295) and `src/components/TaskBar.tsx` (lines 22-57) is virtually identical. Both implement the same Konva export vs fallback-to-dialog logic.
- Files: `src/components/ControlPanel.tsx`, `src/components/TaskBar.tsx`
- Impact: Any fix or enhancement to export behavior must be applied in two places, leading to drift.
- Fix approach: Extract shared export logic into a standalone function or a dedicated hook.

### CI uses `npm ci` but project uses `pnpm`

- Issue: Both `build.yml` and `release.yml` GitHub Actions workflows use `npm ci`, `cache: npm`, and `npm run build`. However, the project uses `pnpm` as its package manager (`package.json` never has a `package-lock.json` committed; the lockfile should be `pnpm-lock.yaml`).
- Files: `.github/workflows/build.yml` (lines 22-23), `.github/workflows/release.yml` (lines 100-101)
- Impact: CI installs may use different dependency resolutions than local development, potentially introducing CI-only bugs.
- Fix approach: Switch CI to `pnpm install --frozen-lockfile` and update cache to use `pnpm`.

### WebP and PNG quality settings ignored in Rust backend

- Issue: In `src-tauri/src/commands/mod.rs`, the quality slider only affects JPG output. WebP output always uses `WebPEncoder::new_lossless` (line 101), ignoring the user's quality setting. PNG output always uses maximum compression (lines 108-117), also ignoring quality. The frontend quality slider implies control over output for all formats.
- Files: `src-tauri/src/commands/mod.rs` (lines 100-117)
- Impact: Users cannot control compression level for WebP or PNG outputs. WebP saves to unnecessarily large lossless files when lossy would be acceptable.
- Fix approach: Pass the quality parameter to WebPEncoder as a lossy config, and offer a configurable compression level for PNG.

### Model URL configuration drift across codebase

- Issue: The model download URL is configured in three places with different defaults:
  1. `src-tauri/src/commands/download.rs` line 12: defaults to ModelScope URL
  2. `src-tauri/.env.example`: defaults to HuggingFace URL
  3. `.github/workflows/release.yml` line 154: defaults to `mocdn.mopng.cn` CDN URL
  Also, `src-tauri/src/models/birefnet.rs` (lines 121-143) hardcodes ModelScope, HuggingFace, and HF Mirror URLs independently of the download command defaults.
- Files: `src-tauri/src/commands/download.rs`, `src-tauri/src/models/birefnet.rs`, `src-tauri/.env.example`, `.github/workflows/release.yml`
- Impact: Inconsistent URLs mean the model download experience differs between development and production builds, and between the model sources dialog vs the compile-time fallback URL.
- Fix approach: Consolidate all model URLs into a single source of truth. Remove the compile-time environment variable overrides in favor of a runtime config.

### ONNX GraphOptimizationLevel::Disable

- Issue: In `src-tauri/src/models/birefnet.rs` line 43, the ONNX session is created with `GraphOptimizationLevel::Disable`. This means no graph-level optimizations (constant folding, node elimination, layout optimization) are applied, resulting in slower inference.
- Files: `src-tauri/src/models/birefnet.rs` (line 43)
- Impact: Reduced inference performance. For a BiRefNet model, this could mean seconds of extra processing time per image.
- Fix approach: Enable at least `Basic` optimization level, and benchmark to determine if `Extended` or `All` is safe for the model.

### Hardcoded model size in ModelDialog

- Issue: `src/components/ModelDialog.tsx` line 17 declares `const MODEL_SIZE_MB = 900` as a hardcoded constant. The actual model file may change size (different ONNX export, different model version) and this value would become inaccurate.
- Files: `src/components/ModelDialog.tsx` (line 17)
- Impact: Users see incorrect file size information when downloading models.
- Fix approach: Fetch model size from the server at dialog open time, or retrieve it from the Rust backend (e.g., from the model sources descriptor).

## Known Bugs

### Checkerboard tile size mismatch between preview and export

- Symptoms: The Konva preview engine (`src/components/konva/mattingEngine.ts` line 7) uses `CHECKER_CELL = 10` for its checkerboard background pattern, but the Rust backend (`src-tauri/src/commands/mod.rs` line 428) uses `tile_size = 16` when rendering the checkerboard background to the output file. The preview and the exported result use different checkerboard patterns.
- Files: `src/components/konva/mattingEngine.ts` (line 7), `src-tauri/src/commands/mod.rs` (line 428)
- Trigger: Set background type to "checkerboard" and export. The preview shows 10px cells, the exported image uses 16px cells.
- Workaround: Cosmetic only, does not affect matting quality.

### Gradient background composites on white instead of gradient in Rust backend

- Symptoms: The `apply_bg_type` function in `commands/mod.rs` handles gradient background type by falling through to the default arm `_ => composite_on_solid(img, 255, 255, 255)` (line 394). Gradient background is not rendered in the output file; it composites on solid white instead.
- Files: `src-tauri/src/commands/mod.rs` (lines 381-396)
- Trigger: Select background type "gradient" and process an image via the main "开始抠图" button. The exported output has a solid white background instead of the gradient shown in the canvas preview.
- Workaround: Use the Konva-based export path ("另存为..." button) which renders gradients correctly by exporting from the canvas.

### Batch processing sequentially mutates shared task list

- Symptoms: In `src/components/ControlPanel.tsx` lines 206-247, batch processing iterates over `pendingTasks` captured at the start. Each iteration calls `selectTask(task.id)` which updates `selectedTaskId` in the zustand store, triggering re-renders of connected components (PreviewCanvas). This can cause the canvas to reload images for each task in the batch.
- Files: `src/components/ControlPanel.tsx` (lines 206-247)
- Trigger: Start batch processing with multiple images. During processing, each task's selection change may trigger unnecessary canvas reinitialization.
- Workaround: Avoid visual glitches by not interacting with the app during batch processing.

## Security Considerations

### Tauri shell plugin exposed without clear usage

- Risk: `tauri_plugin_shell` is initialized in `src-tauri/src/main.rs` (line 17) and listed in `package.json` as a dependency, but no shell commands are registered in the capabilities or invoked in the codebase. If not scoped properly via Tauri capability permissions, this could be a vector for arbitrary shell execution.
- Files: `src-tauri/src/main.rs` (line 17), `package.json` (line 29)
- Current mitigation: No Tauri commands expose shell execution, but the plugin's capability permissions should be verified.
- Recommendations: Verify `src-tauri/capabilities/` config restricts shell plugin permissions to only what is needed, or remove the plugin if unused.

### File path validation in Tauri commands

- Risk: Several Tauri commands accept raw file path strings from the frontend without path traversal validation (e.g., `read_file_as_data_url`, `read_image_file`, `open_in_folder`, `export_image_dialog`). While Tauri's IPC isolation prevents arbitrary access, the commands could be invoked with unexpected paths.
- Files: `src-tauri/src/commands/file.rs` (lines 6, 13), `src-tauri/src/commands/mod.rs` (lines 190, 235)
- Current mitigation: Tauri v2's capability-based security model restricts which IPC commands the webview can invoke.
- Recommendations: Add path normalization and allow-list checks for file operations.

## Performance Bottlenecks

### Large images converted to base64 for thumbnails and previews

- Problem: The `generate_thumbnail` command (`commands/mod.rs` lines 156-186) reads the entire image file, decodes it, resizes it, re-encodes to PNG, then base64-encodes the result to return as a data URL string. High-resolution images (4000x3000+) cause three full decode/encode cycles per image during thumbnail generation and preview loading.
- Files: `src-tauri/src/commands/mod.rs` (lines 156-186), `src-tauri/src/commands/file.rs` (lines 13-28)
- Cause: Multiple full decode-encode cycles and base64 serialization for IPC transfer.
- Improvement path: Use Tauri's `convertFileSrc` / `asset://` protocol instead of base64 data URLs for image display where possible. Resize downsampled data directly without full decode.

### Pixel-by-pixel mask composition in Rust backend

- Problem: The `apply_mask` function (lines 351-371 in `commands/mod.rs`) iterates over every pixel individually using nested loops. For a 20MP image, this means 20 million loop iterations with per-pixel function calls, each involving array indexing and arithmetic. Similarly, `composite_on_solid` and `composite_on_checkerboard` use the same pattern.
- Files: `src-tauri/src/commands/mod.rs` (lines 351-453)
- Cause: Naive per-pixel processing without using image crate's optimized bulk operations.
- Improvement path: Use `image::imageops` bulk operations or process pixels in scanline batches.

### Single-threaded ONNX inference

- Problem: Each `process_image` call runs inference on a `tokio::task::spawn_blocking` thread, but only one model instance can be active at a time (global registry mutex). Batch processing iterates sequentially. The ORT session uses `with_intra_threads(1)` (birefnet.rs line 44), meaning each inference uses a single CPU thread.
- Files: `src-tauri/src/models/birefnet.rs` (line 44), `src-tauri/src/models/registry.rs` (line 37)
- Cause: Global mutex locks the model during inference which can take seconds.
- Improvement path: Use a model pool (multiple session instances) for concurrent inference. Increase intra-thread count for multi-core CPUs. Consider GPU inference via ONNX Runtime CUDA provider.

## Fragile Areas

### `mattingEngine.ts` -- monolithic Konva rendering engine

- Files: `src/components/konva/mattingEngine.ts` (618 lines)
- Why fragile: Single file handles viewport zoom/pan (both mouse and touch), background rendering (6 types), mask compositing, export, resize, and compare mode. Any change to one subsystem risks breaking others due to tight coupling via shared mutable state (`worldScale`, `worldX`, `worldY`, `bgMode`, etc.).
- Safe modification: When modifying, always update the `KonvaEngineApi` type definition in `src/components/konva/types.ts` first to define the contract. Avoid adding new state to the closure -- extend the API instead.
- Test coverage: None.

### `ControlPanel.tsx` -- coordinates too many responsibilities

- Files: `src/components/ControlPanel.tsx` (641 lines)
- Why fragile: Contains background configuration, size template selection, model switching, single/batch processing, output management, and export. Changes to any one of these features risk affecting others through shared `currentSettings` mutations.
- Safe modification: Extract background config (bgType, bgColor, bgGradient, bgImageUrl, opacity) into a dedicated sub-component. Same for size template controls.
- Test coverage: None.

### Rust registry module uses global mutex

- Files: `src-tauri/src/models/registry.rs` (128 lines)
- Why fragile: The `REGISTRY` is a global `Mutex<RegistryInner>` (line 37). Every call to `list_models()`, `init_model()`, `is_model_loaded()`, `infer()`, `model_filename_for()`, and `model_sources_for()` acquires and holds this mutex. Long-running inference (potentially multiple seconds) blocks all other registry access.
- Safe modification: Keep `infer()` lock time minimal. Consider splitting the model instance from the descriptor registry so inference does not block descriptor reads.
- Test coverage: Only the `infer()` path is tested (via the BirefNet unit test); registry management functions are untested.

## Dependencies at Risk

### `ort = "2.0.0-rc.12"` (release candidate)

- Risk: The ONNX Runtime Rust bindings are pinned to a pre-release release candidate version (`rc.12`). API-breaking changes may occur between RC versions, and the final 2.0.0 release may not be backward compatible. The `download-binaries` feature auto-downloads native libraries that may not match the production release.
- Impact: CI builds depend on auto-downloaded ORT binaries. If the download URL changes or the RC is removed, CI builds fail. macOS Intel cross-compilation requires manual dylib management (see `release.yml` lines 109-189).
- Files: `src-tauri/Cargo.toml` (line 21), `.github/workflows/release.yml` (lines 109-189)
- Migration plan: Pin to a specific ORT 1.x stable version that is compatible with the current model, or upgrade to ORT 2.0.0 stable once released.

## Missing Critical Features

### No graceful fallback when model fails to load

- Problem: If `init_model` fails (invalid model file, incompatible ONNX opset), the error is only logged to console via `console.warn` in `src/App.tsx` (line 49). The user sees no error indicator and the app continues to an unusable state.
- Files: `src/App.tsx` (lines 46-50)
- Blocks: Users who download a corrupted model file get no feedback about what went wrong.
- Priority: High

## Test Coverage Gaps

### Zero frontend tests

- What's not tested: Entire React frontend -- all components (ControlPanel, PreviewCanvas, ModelDialog, SettingsDialog, TaskBar, ThumbnailList, BatchProgress, DropZone, GradientAnglePicker), the Konva matting engine, the Zustand store, utility functions, and type definitions have zero test coverage.
- Files: All files in `src/` and `src/components/`
- Risk: Any refactoring or enhancement risks breaking existing functionality without detection. The complex state interactions (settings sync between store, engine, and controls) are particularly fragile.
- Priority: High

### Only one Rust test, which auto-skips

- What's not tested: The single test in `src-tauri/src/models/birefnet.rs` (lines 214-259) auto-skips if the model file is not present on the local machine. No tests exist for commands (process_image, download_model, file operations), the registry, or error handling paths.
- Files: `src-tauri/src/models/birefnet.rs` (lines 214-259)
- Risk: Rust backend changes (especially to image processing pipelines and download/resume logic) cannot be validated in CI.
- Priority: High

---

*Concerns audit: 2026-04-28*
