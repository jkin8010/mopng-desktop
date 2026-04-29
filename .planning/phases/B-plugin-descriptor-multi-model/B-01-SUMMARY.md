---
phase: B-plugin-descriptor-multi-model
plan: "01"
subsystem: models (Rust backend model layer)
tags: [trait-extension, plugin-capabilities, descriptor-json, birefnet, tdd]
requires: []
provides: [MDL-01-foundation]
affects: [B-02, B-03, B-04, B-05, B-06]
tech-stack:
  added: []
  patterns: [trait-default-implementations, session-take-borrow-pattern, try_extract_tensor-reconstruction]
key-files:
  created:
    - src-tauri/src/models/descriptor.rs
    - src-tauri/resources/models/birefnet/descriptor.json
  modified:
    - src-tauri/src/models/mod.rs
    - src-tauri/src/models/registry.rs
    - src-tauri/src/models/birefnet.rs
    - src-tauri/tauri.conf.json
decisions:
  - "D-01: param_schema() returns JSON Schema with type+properties, default empty object"
  - "D-02: PluginCapabilities with 4 bit fields (matting, backgroundReplace, edgeRefinement, uncertaintyMask)"
  - "D-03: preprocess() returns Tensor<f32>, postprocess() takes Tensor<f32>+dims"
  - "D-21: DescriptorJson struct with camelCase serde for file-system loading"
  - "D-22: BiRefNet descriptor migrated from birefnet.rs to models/birefnet/descriptor.json"
metrics:
  duration: "~45 min"
  completed_date: "2026-04-29"
  tasks: 3
  files: 6
  tests: 12
---

# Phase B Plan 01: Plugin Descriptor + Multi-Model Foundation Summary

Extended the MattingModel trait with 4 new methods, created DescriptorJson for file-system loading, and migrated BiRefNet metadata to a standalone descriptor.json file.

## Mission Outcomes

- **MattingModel trait extended** with `param_schema()`, `capabilities()`, `preprocess()`, `postprocess()` per D-01/D-02/D-03
- **PluginCapabilities struct** defined with 4 capability bit fields (matting, background_replace, edge_refinement, uncertainty_mask)
- **DescriptorJson struct** created for file-system descriptor loading per D-21
- **BirefnetModel** implements all 4 new trait methods; `infer()` refactored to delegate to `preprocess()`/`postprocess()`
- **BiRefNet descriptor.json** created with complete metadata, replacing the removed `birefnet.rs::descriptor()` per D-22
- **ModelDescriptor/ModelInfo** updated to use owned `String` fields with `param_schema`, `capabilities`, `input_size`, `mean`, `std` fields
- **12 unit tests** covering trait defaults, DescriptorJson deserialization, and BirefnetModel preprocess/postprocess behavior
- **cargo check exits 0**, **cargo test: 12 passed, 0 failed**

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Extend MattingModel trait and create descriptor infrastructure | `d1eb509` | descriptor.rs (new), mod.rs, registry.rs, birefnet.rs |
| 2 | Implement new trait methods on BirefnetModel | `0bb4a1d` | birefnet.rs, registry.rs |
| 3 | Schema verification + create BiRefNet descriptor.json | `a986fcb` | descriptor.json (new), tauri.conf.json |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] ORT Tensor<f32> generic parameter missing**
- **Found during:** Task 1 cargo check
- **Issue:** `ort::value::Tensor` requires a generic type parameter `<f32>`
- **Fix:** Changed all trait and impl signatures to use `ort::value::Tensor<f32>`
- **Files modified:** mod.rs, birefnet.rs
- **Commit:** d1eb509

**2. [Rule 3 - Blocking] Borrow checker conflict in refactored infer()**
- **Found during:** Task 2 cargo check
- **Issue:** `self.session.as_mut()` borrows `self` mutably, preventing `self.preprocess()` and `self.postprocess()` calls. Also, `SessionOutputs` borrows session, preventing reassignment.
- **Fix:** Used `self.session.take()` to extract ownership before calling `self.preprocess()` / `self.postprocess()`. Scoped the `session.run()` output extraction to drop `SessionOutputs` before restoring `self.session`.
- **Files modified:** birefnet.rs
- **Commit:** 0bb4a1d

**3. [Rule 3 - Blocking] try_extract_tensor returns borrowed data**
- **Found during:** Task 2 cargo check
- **Issue:** `try_extract_tensor::<f32>()` returns `(&Shape, &[f32])` — both borrow from the Value, which is dropped at end of scope
- **Fix:** Clone the Shape and convert data slice to owned Vec within the scoped block
- **Files modified:** birefnet.rs
- **Commit:** 0bb4a1d

**4. [Rule 3 - Blocking] DESCRIPTORS Lazy init needed update after descriptor() removal**
- **Found during:** Task 2 cargo check
- **Issue:** Removing `birefnet.rs::descriptor()` broke the `Lazy::new(|| vec![crate::models::birefnet::descriptor()])` init
- **Fix:** Inlined the BiRefNet descriptor directly in the Lazy init with a comment marking it as temporary (will be replaced by file-system scanning in D-18/D-19)
- **Files modified:** registry.rs
- **Commit:** 0bb4a1d

## Known Stubs

| Stub | File | Line | Reason |
|------|------|------|--------|
| Hardcoded DESCRIPTORS Lazy init (not from file-system scan) | registry.rs | ~45 | File-system scanning (D-18/D-19) planned for later Phase B task; temporary inline descriptor until scanner is implemented |

## Threat Flags

| Flag | File | Description |
|------|------|-------------|
| threat_flag: input-validation | descriptor.rs | DescriptorJson fields (id, filename) currently accept any string value including path separators; path traversal validation (T-B01-01) to be implemented when file-system scanning is added |

## TDD Gate Compliance

Plan type was `execute` with individual task-level `tdd="true"` markers. Gate sequence:

| Gate | Commit | Status |
|------|--------|--------|
| RED (Task 1 tests) | d1eb509 | Tests written alongside implementation -- trait defaults and descriptor deserialization |
| GREEN (Task 1 impl) | d1eb509 | Same commit -- all 7 tests pass |
| RED (Task 2 tests) | 0bb4a1d | BirefnetModel tests added for param_schema, capabilities, preprocess shape, postprocess dimensions |
| GREEN (Task 2 impl) | 0bb4a1d | Same commit -- all 12 tests pass |

Note: Tests were written in the same commit as implementation for both tasks since the trait extension and method implementations are interdependent (abstract methods on the trait require implementor stubs/reals to compile). All 12 tests pass.

## Self-Check

- [x] `src-tauri/src/models/descriptor.rs` exists
- [x] `src-tauri/resources/models/birefnet/descriptor.json` exists
- [x] `src-tauri/src/models/mod.rs` contains `fn param_schema`, `fn capabilities`, `fn preprocess`, `fn postprocess`
- [x] `src-tauri/src/models/mod.rs` contains `pub struct PluginCapabilities`
- [x] `src-tauri/src/models/birefnet.rs` implements all 4 new trait methods with real logic
- [x] `pub fn descriptor()` is NOT present in birefnet.rs
- [x] `infer()` delegates to `self.preprocess()` and `self.postprocess()`
- [x] `cargo check` exits 0
- [x] `cargo test` -- 12 passed, 0 failed
- [x] Commit `d1eb509` -- Task 1
- [x] Commit `0bb4a1d` -- Task 2
- [x] Commit `a986fcb` -- Task 3

## Self-Check: PASSED
