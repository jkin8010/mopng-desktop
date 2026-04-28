---
phase: A-foundation-stabilization
plan: "02"
subsystem: download-pipeline
tags:
  - sha256-verification
  - download-fallback
  - url-consolidation
  - error-tracking
requires:
  - A-01 (registry ModelDescriptor.sources API)
provides:
  - sha256 streaming verification for downloaded models
  - automatic download source fallback chain
  - per-source error tracking with structured error response
  - single source of truth for model download URLs
affects:
  - src-tauri/Cargo.toml (new dependencies)
  - src-tauri/src/commands/download.rs (full rewrite)
  - src-tauri/src/models/registry.rs (SHA256 delegation)
  - src-tauri/.env.example (URL cleanup)
  - src-tauri/src/main.rs (handler cleanup)
tech-stack:
  added:
    - sha2 = "0.11" (SHA256 hashing)
    - hex = "0.4" (hex encoding)
key-files:
  created: []
  modified:
    - src-tauri/Cargo.toml
    - src-tauri/src/commands/download.rs
    - src-tauri/src/models/registry.rs
    - src-tauri/.env.example
    - src-tauri/src/main.rs
  deleted: []
decisions:
  - Use hex::encode(hasher.finalize()) instead of format!("{:x}", ...) for sha2 GenericArray type compatibility
  - .env MODEL_URL override is prepended as first-priority source rather than replacing all sources
  - compute_file_sha256 lives in download.rs (sync, pub(crate)), registry.rs delegates to it
metrics:
  duration: "~15 min"
  completed_date: "2026-04-28"
  commits: 3
  files_changed: 5
---

# Phase A Plan 02: SHA256 Download Pipeline + Fallback Chain

Rewrite the download pipeline with SHA256 streaming verification, automatic fallback chain, per-source error tracking, and URL configuration consolidation (remove all compile-time constants, read from ModelDescriptor.sources).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] sha2 GenericArray type not compatible with format!("{:x}", ...)**
- **Found during:** Task 1
- **Issue:** `hasher.finalize()` returns `GenericArray<u8, U64>` which doesn't implement `LowerHex` trait in the presence of ndarray's `Array` type in scope. `format!("{:x}", hasher.finalize())` fails to compile with LowerHex trait resolution error.
- **Fix:** Replaced `format!("{:x}", hasher.finalize())` with `hex::encode(hasher.finalize())` in both `compute_file_sha256` (Task 1) and `download_from_source` (Task 2).
- **Files modified:** `src-tauri/src/commands/download.rs` (2 occurrences)
- **Commit:** 4d6521b, 041bc9d

## Threat Flags

None — all new threat surface (SHA256 checksum comparison, .env override URL, download fallback error responses) is covered by the existing threat model (T-A-04, T-A-05, T-A-06).

## Known Stubs

None identified.

## Execution Summary

### Task 1: Add sha2/hex dependencies + SHA256 compute tool function

**Commit:** 4d6521b

Added `sha2 = "0.11"` and `hex = "0.4"` to Cargo.toml. Added `compute_file_sha256()` function to download.rs with streaming read (8KB buffer). Updated registry.rs to delegate to download.rs's implementation instead of the placeholder stub.

### Task 2: Rewrite download pipeline — SHA256 streaming verification + auto fallback + per-source errors

**Commit:** 041bc9d

Complete rewrite of `src-tauri/src/commands/download.rs`:

**Deleted (7 items):**
- `const DEFAULT_MODEL_URL` (compile-time constant)
- `const DEFAULT_MODEL_FILENAME` (compile-time constant)
- `fn model_url()` (runtime URL builder)
- `fn model_filename()` (runtime filename builder)
- `fn model_download_url()` (URL construction)
- `const HF_RAW_URL` / `const HF_MIRROR_URL` (hardcoded URL constants)
- `fn get_model_download_url()` + `fn download_model_inner()` (replaced functions)

**Added:**
- `SourceError` struct — per-source error tracking (source_id, source_name, error_type, detail)
- `DownloadErrorResponse` struct — structured error when all sources fail
- `download_model_with_fallback()` — iterates sources, returns structured error on total failure
- `download_from_source()` — single-source download with SHA256 streaming verification per chunk, atomic .tmp rename, and resume support
- `verify_download_checksum()` — async helper to validate downloaded chunks
- `verify_checksum_against_descriptor()` — checks computed hash against ModelDescriptor.checksum
- `classify_download_error()` — classifies errors into "http_404", "http_5xx", "checksum_mismatch", "timeout", "network"

**Behavioral changes:**
- `download_model` signature: `(app, source_url: Option, model_id: Option)` -> `(app, model_id: Option)` — source_url removed
- Download sources now come exclusively from `ModelDescriptor.sources` (per D-04)
- .env `MODEL_URL` override prepended as first-priority source (still SHA256 verified per T-A-05)
- `get_model_sources` simplified — no more fallback to hardcoded URLs

### Task 3: Clean up .env.example + main.rs handler list

**Commit:** 3975087

- Rewrote `.env.example` to remove all hardcoded HuggingFace/ModelScope/CDN URLs. Now contains only commented-out documentation of MODEL_URL and MODEL_FILENAME override variables.
- Removed `commands::get_model_download_url` from main.rs `generate_handler![]` macro (function was deleted in Task 2).

## Verification Results

| Check | Result |
|-------|--------|
| `cargo check -p mopng-desktop` | Passed (only pre-existing dead_code warnings) |
| No DEFAULT_MODEL_URL, HF_RAW_URL, HF_MIRROR_URL, model_url(), model_download_url() | 0 matches |
| `sha2` in Cargo.toml | 1 match |
| `hasher` usage count | 7 matches (>= 6) |
| `download_model` signature without source_url | Confirmed |
| No hardcoded URLs in .env.example | 0 matches |

## Plan Commits

| Commit | Message |
|--------|---------|
| 4d6521b | feat(A-foundation-stabilization-02): add sha2/hex dependencies and SHA256 compute tool function |
| 041bc9d | feat(A-foundation-stabilization-02): rewrite download pipeline with SHA256 streaming verification and auto fallback |
| 3975087 | chore(A-foundation-stabilization-02): clean up .env.example and main.rs handler list |

## Self-Check: PASSED

All modified files exist and all commits are present in git history.
