---
phase: B-plugin-descriptor-multi-model
plan: "04"
subsystem: model-implementation
tags: [tauri, rust, rmbg, onnx, multi-model, threshold]

# Dependency graph
requires:
  - phase: B-plugin-descriptor-multi-model
    plan: "01"
    provides: "MattingModel trait with param_schema/capabilities/preprocess/postprocess, PluginCapabilities struct"
  - phase: B-plugin-descriptor-multi-model
    plan: "02"
    provides: "BirefnetModel with bilinear_resize_f32 upscaler"
  - phase: B-plugin-descriptor-multi-model
    plan: "03"
    provides: "File-system model discovery, DESCRIPTORS RwLock, scan_models_directory()"
provides:
  - RmbgModel struct implementing MattingModel trait for RMBG 1.4
  - Threshold parameter (0.0-1.0, default 0.5) applied in postprocess
  - pub(crate) bilinear_resize_f32 shared upscaler
  - descriptor.json files for rmbg-fp32 and rmbg-fp16
affects: [B-05, B-06, B-07]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "pub(crate) fn bilinear_resize_f32 shared between birefnet.rs and rmbg.rs"
    - "RmbgModel uses same take/restore session pattern as BirefnetModel for borrow-safe inference"
    - "Threshold postprocessing: clip values below t to 0, rescale [t, 1.0] to [0.0, 1.0]"

key-files:
  created:
    - src-tauri/src/models/rmbg.rs
    - src-tauri/resources/models/rmbg-fp32/descriptor.json
    - src-tauri/resources/models/rmbg-fp16/descriptor.json
  modified:
    - src-tauri/src/models/birefnet.rs
    - src-tauri/src/models/mod.rs
    - src-tauri/src/models/registry.rs

key-decisions:
  - "RMBG 1.4 FP32 and FP16 registered as separate model IDs (rmbg-fp32 / rmbg-fp16) per D-04"
  - "Hardcoded input_size/mean/std in RmbgModel::new() match descriptor.json values — D-06 compliance note"
  - "bilinear_resize_f32 made pub(crate) for rmbg.rs reuse rather than duplicating the upscaler"
  - "Same take/restore session pattern as BirefnetModel for consistent borrow-safe inference"

patterns-established:
  - "MattingModel trait implementation pattern: struct with model_id + threshold fields, trait methods delegate to shared upscaler"
  - "Paired FP32/FP16 model registration: separate descriptor.json files, single RmbgModel struct"

requirements-completed: ["MDL-02"]

# Metrics
duration: 12min
completed: 2026-04-30
---

# Phase B Plan 04: RMBG 1.4 Model Implementation Summary

**Implemented the RMBG 1.4 model as a second MattingModel trait implementation — supports FP32 and FP16 variants with threshold parameter for edge sharpness control**

## Performance

- **Duration:** 12 min
- **Tasks:** 2
- **Files created:** 3
- **Files modified:** 3

## Accomplishments

- Created `RmbgModel` struct in `src-tauri/src/models/rmbg.rs` with full `MattingModel` trait implementation
- Implemented threshold parameter (0.0-1.0, default 0.5) applied in postprocess: values below threshold clipped to 0, remaining rescaled to [0,1]
- Made `bilinear_resize_f32()` pub(crate) in birefnet.rs for rmbg.rs reuse
- Registered `rmbg-fp32` and `rmbg-fp16` model IDs in `create_model()`
- Created descriptor.json files for both FP32 and FP16 variants with download sources and threshold param_schema
- Added 8 unit tests covering: ID creation, param_schema, capabilities, preprocess shape, postprocess threshold clipping, postprocess shape, threshold sensitivity
- All 25 tests pass (8 new + 17 existing), cargo check clean

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement RmbgModel in rmbg.rs** — `027c915` (feat) — Created rmbg.rs with RmbgModel struct, MattingModel trait impl, threshold parameter, 8 unit tests; made bilinear_resize_f32 pub(crate)
2. **Task 2: Register RMBG in registry + descriptor.json files** — `baf1976` (feat) — Added pub mod rmbg, extended create_model() with rmbg-fp32/rmbg-fp16, created two descriptor.json files

## Files Modified/Created

- `src-tauri/src/models/rmbg.rs` — NEW: RmbgModel struct with MattingModel trait implementation, threshold parameter, preprocess/postprocess pipeline, 8 unit tests
- `src-tauri/src/models/birefnet.rs` — MODIFIED: `bilinear_resize_f32` changed from `fn` to `pub(crate) fn` (line 192)
- `src-tauri/src/models/mod.rs` — MODIFIED: Added `pub mod rmbg;` after existing modules
- `src-tauri/src/models/registry.rs` — MODIFIED: Extended `create_model()` with `"rmbg-fp32"` and `"rmbg-fp16"` match arms
- `src-tauri/resources/models/rmbg-fp32/descriptor.json` — NEW: FP32 variant with threshold param_schema, ModelScope/HF sources
- `src-tauri/resources/models/rmbg-fp16/descriptor.json` — NEW: FP16 variant with rmbg_fp16.onnx filename

## Decisions Made

- FP32 and FP16 registered as separate model IDs (not as a precision parameter) — matches plan D-04 and real-world usage where users want to switch between precision modes like they switch between models
- bilinear_resize_f32 made pub(crate) rather than duplicated — avoids 20-line code duplication, follows DRY principle
- Used same take/restore session pattern as BirefnetModel — consistent borrow-safe approach for ONNX session management
- Threshold rescaling formula clamped to [0,1] to handle floating-point edge cases at t=1.0

## Deviations from Plan

Minimal: Fixed one unused import warning (`Array3` in rmbg.rs) that the plan's code block included. Removed unused `Array3` from ndarray import.

## Issues Encountered

None. All 8 new tests passed on first run. No compilation errors.

## User Setup Required

None. RMBG 1.4 models will be auto-discovered by `scan_models_directory()` when descriptor.json files are present. Users need to download the ONNX model files via the existing download dialog. The descriptor.json files with `null` checksum will load with a warning (per T-B04-01 threat model mitigation).

## Next Phase Readiness

- RmbgModel fully implements MattingModel trait — ready for hot-swap (MDL-03 / B-05)
- Threshold param_schema exposes model-specific parameter — ready for auto-generated UI (MDL-04 / B-06)
- Two model IDs (rmbg-fp32, rmbg-fp16) registered alongside birefnet — ready for multi-model selection UI
- Descriptor.json files follow same format as birefnet — compatible with scan_models_directory()

---
*Phase: B-plugin-descriptor-multi-model*
*Plan: 04*
*Completed: 2026-04-30*
