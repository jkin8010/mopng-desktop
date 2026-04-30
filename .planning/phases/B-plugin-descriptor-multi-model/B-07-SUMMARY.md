---
phase: B-plugin-descriptor-multi-model
plan: 07
type: fix
subsystem: "model-params-ui"
tags: [model-switch, revert, model-params, threshold, inference-pipeline]
depends_on: [B-07]
provides: [MDL-04]
affects: [ControlPanel, PreviewCanvas, Zustand store, Rust IPC, RmbgModel, BirefnetModel, registry]
tech-stack:
  added: []
  patterns:
    - "Model switch revert: restore old model on failure, prevent permanent model loss"
    - "Dynamic model params: thread threshold from Zustand through IPC to infer()"
    - "Temporary threshold override pattern in infer() without changing postprocess signature"
key-files:
  modified:
    - "src-tauri/src/models/registry.rs"
    - "src-tauri/src/models/mod.rs"
    - "src-tauri/src/models/rmbg.rs"
    - "src-tauri/src/models/birefnet.rs"
    - "src-tauri/src/commands/mod.rs"
    - "src/store/useStore.ts"
    - "src/App.tsx"
    - "src/components/PreviewCanvas.tsx"
    - "src/components/ControlPanel.tsx"
metrics:
  duration: "15 min"
  completed_date: "2026-04-30"
decisions: []
---

# Phase B Plugin Descriptor Multi-Model: Verification Gap Fixes

## One-Liner

Fix two verification gaps in B-07: (1) model switch failure now reverts to old model with error card UI instead of permanent loss, (2) threshold slider values now actually affect RMBG 1.4 inference output.

## Tasks Executed

### Task 1: Gap 1 — Add revert logic on switch failure + error card UI

**Part A — registry.rs:** `switch_model_with_dir` now captures the old model before `active.take()` and moves it into the spawned thread closure. On both `Err` and panic branches, the old model is restored to `ACTIVE_MODEL` and its state set back to `Loaded`.

**Part B — useStore.ts:** Added `modelSwitchingError: string | null` field and `setModelSwitchingError` action.

**Part C — App.tsx:** In the polling useEffect, when switch fails (state === "error" and modelSwitching is true), calls `setModelSwitchingError` with the extracted error message.

**Part D — PreviewCanvas.tsx:** Added error card UI rendered after the modelSwitching overlay. Shows XCircle icon, error title/message, and dismiss button. Uses existing lucide XCircle and X icons.

**Part E — ControlPanel.tsx:** `handleModelChange` clears `modelSwitchingError` before initiating a new switch, auto-dismissing any prior error.

- Commit: `f05a7ba`

### Task 2: Gap 2 — Thread model_params through inference pipeline

**Part A — mod.rs:** Changed `MattingModel::infer` trait signature from `infer(&mut self, image: DynamicImage)` to `infer(&mut self, image: DynamicImage, params: serde_json::Value)`.

**Part B — registry.rs:** Updated `infer()` to accept and forward `params` to the active model's `infer()`.

**Part C — commands/mod.rs:** Clones `params.model_params` outside the spawning closure and passes it to `registry::infer()`.

**Part D — rmbg.rs:** `RmbgModel::infer()` reads `params.get("threshold")`, falls back to `self.threshold` (0.5). Uses temporary threshold override pattern: saves `self.threshold`, sets it to the dynamic value, calls `self.postprocess()`, restores the original.

**Part E — birefnet.rs:** Updated `BirefnetModel::infer()` signature to accept `_params: serde_json::Value` (unused since BiRefNet has no params).

**Part F — Tests:** All `infer()` calls in test code updated to pass `serde_json::json!({})`.

- Commit: `f3197c0`

## Verification Results

- `npx tsc --noEmit` — PASSED (exit code 0)
- `cargo check` — PASSED (exit code 0, 3 pre-existing warnings)
- `cargo test -- --test-threads=1` — PASSED (29/29 tests passed)

## Deviations from Plan

None — plan executed exactly as written.

## Threat Assessment

No new threat flags. The `model_params` serde_json::Value flows through existing IPC trust boundaries. Threshold is safely extracted with `.and_then(|v| v.as_f64())` — invalid/missing values fall back to default (0.5).

## Known Stubs

None.

## Self-Check: PASSED

Both commits exist in git log. All 9 modified files verified.
