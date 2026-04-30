---
phase: B-plugin-descriptor-multi-model
plan: "03"
subsystem: model-registry
tags: [tauri, rust, plugin-discovery, descriptor-json, model-scanning]

# Dependency graph
requires:
  - phase: B-plugin-descriptor-multi-model
    plan: "01"
    provides: "DescriptorJson struct, ModelDescriptor with param_schema/capabilities fields, PluginCapabilities struct"
provides:
  - File-system model discovery via scan_models_directory()
  - scan_models Tauri command (IPC endpoint)
  - DESCRIPTORS populated from models/*/descriptor.json at runtime
  - TypeScript ModelInfo extended with paramSchema/capabilities
affects: [B-04, B-05, B-06, B-07]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "File-system plugin discovery: models/*/descriptor.json scanned at startup via scan_models Tauri command"
    - "pub(crate) Lazy<RwLock<Vec<ModelDescriptor>>> DESCRIPTORS pattern for late-init global registry"

key-files:
  created: []
  modified:
    - src-tauri/src/models/registry.rs
    - src-tauri/src/models/mod.rs
    - src-tauri/src/main.rs
    - src/App.tsx
    - src/types/index.ts

key-decisions:
  - "DESCRIPTORS initialized empty (RwLock::new(Vec::new())) — populated by scan_models_directory at startup"
  - "scan_models_directory silently skips invalid/missing descriptors (log::warn, no crash)"
  - "Path traversal prevention: validate filename contains no /, \\, or .. in scan_models_directory (T-B03-01)"

patterns-established:
  - "File-system plugin discovery: subdirectories under models/ each contain descriptor.json + model.onnx"

requirements-completed: ["MDL-06"]

# Metrics
duration: 18min
completed: 2026-04-30
---

# Phase B Plan 03: File-System Model Discovery Summary

**Replaced compile-time hardcoded BiRefNet descriptor with runtime file-system plugin discovery — models auto-registered from `models/*/descriptor.json` via `scan_models` Tauri command**

## Performance

- **Duration:** 18 min
- **Tasks:** 3
- **Files modified:** 5

## Accomplishments

- Replaced hardcoded DESCRIPTORS with file-system scanning via `scan_models_directory()`
- Added `scan_models` Tauri command that scans `models/` directory and populates the global registry
- Frontend now calls `invoke("scan_models")` on startup instead of `invoke("list_models")`
- `list_models()` already returned `param_schema` and `capabilities` in `ModelInfo` (confirmed working)
- TypeScript `ModelInfo` extended with optional `paramSchema` and `capabilities` fields
- New `PluginCapabilities` TypeScript interface (matting, backgroundReplace, edgeRefinement, uncertaintyMask)

## Task Commits

Each task was committed atomically:

1. **Task 1: TDD RED — Failing tests** — `3815a89` (test) — 5 tests for scan_models_directory and list_models
2. **Task 1: TDD GREEN — Implementation** — `6472409` (feat) — scan_models_directory(), empty DESCRIPTORS, path validation
3. **Task 2: scan_models command + frontend wiring** — `8efe54c` (feat) — new command in mod.rs, registered in main.rs, App.tsx startup call
4. **Task 3: TypeScript types** — `9892a6e` (feat) — PluginCapabilities interface, ModelInfo extension

## Files Modified

- `src-tauri/src/models/registry.rs` — Added `scan_models_directory()`, replaced hardcoded DESCRIPTORS with `pub(crate) RwLock::new(Vec::new())`, added `DescriptorJson`/`PluginCapabilities` imports, 5 passing unit tests
- `src-tauri/src/models/mod.rs` — Added `scan_models` `#[tauri::command]` (scans models dir, writes DESCRIPTORS, returns `list_models()`)
- `src-tauri/src/main.rs` — Registered `models::scan_models` in `generate_handler![]`
- `src/App.tsx` — Changed `invoke("list_models")` to `invoke("scan_models")` in startup `initModels`
- `src/types/index.ts` — Added `PluginCapabilities` interface, extended `ModelInfo` with `paramSchema?` and `capabilities?`

## Decisions Made

- Used `pub(crate)` visibility for DESCRIPTORS instead of `pub` — sufficient for same-crate access from mod.rs, avoids external exposure
- Path traversal prevention in `scan_models_directory()` validates filename immediately after JSON parse (before pushing to descriptors)
- Used `.clone()` instead of `.to_string()` for `String` fields in `model_filename_for()` — more idiomatic for owned strings
- Tests use `std::env::temp_dir()` with manual cleanup instead of adding `tempfile` dependency — keeps dependency tree minimal

## Deviations from Plan

None — plan executed exactly as written. All implementation steps matched the plan's action blocks. TDD RED/GREEN cycle followed for scan_models_directory. No auto-fixes needed. No blocking issues encountered.

## Issues Encountered

None.

## User Setup Required

None — no external service configuration required. The `models/` directory is created automatically by `registry::model_dir()` at `app_data_dir()/models/`. Users need `models/*/descriptor.json` files for each model plugin (BiRefNet's descriptor.json was created in Phase A/B-01).

## Next Phase Readiness

- DESCRIPTORS is populated at runtime — ready for multi-model registration in upcoming phases
- `scan_models` command returns `ModelInfo` with `param_schema` and `capabilities` — ready for frontend model selection UI
- `check_model` already resolves paths via `model_filename_for(&id)` — ready for multi-model path resolution
- TypeScript types (`PluginCapabilities`, extended `ModelInfo`) — ready for frontend capability-aware components

---
*Phase: B-plugin-descriptor-multi-model*
*Plan: 03*
*Completed: 2026-04-30*
