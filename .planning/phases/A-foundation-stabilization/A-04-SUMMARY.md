---
phase: A-foundation-stabilization
plan: "04"
subsystem: frontend
tags: [typescript, zustand, react, model-management, async-polling]
requires:
  - phase: A-foundation-stabilization
    plan: "01"
    provides: Async init_model and state-aware list_models() response
  - phase: A-foundation-stabilization
    plan: "02"
    provides: SHA256 checksum verification, auto fallback chain, DownloadErrorResponse
provides:
  - Frontend type definitions for ModelInfo.state, DownloadErrorResponse, SourceError
  - Zustand store ModelStatus.state field for lifecycle tracking
  - App.tsx polling loop monitoring model async load state
  - ModelDialog per-source error display with retry and manual URL
affects: [B-01, B-02]

tech-stack:
  added: []
  patterns:
    - Async model init with polling instead of blocking await
    - Download error deserialization from JSON string to typed interface
    - Per-source error detail rendering in dialog

key-files:
  created: []
  modified:
    - src/types/index.ts
    - src/store/useStore.ts
    - src/App.tsx
    - src/components/ControlPanel.tsx
    - src/components/ModelDialog.tsx

key-decisions:
  - "Use fire-and-forget invoke for init_model with polling instead of blocking await"
  - "Use useStore.getState() in polling interval to avoid React closure stale reference"
  - "Keep sources list as informational display (no user source selection) since backend auto-fallback handles retry chain"
  - "Manual URL input shows instruction to set MODEL_URL env var instead of direct download (backend SHA256 verification safety)"

patterns-established:
  - "Polling pattern: setInterval 500ms calling list_models() with cleanup on unmount"
  - "Error display pattern: parse DownloadErrorResponse JSON in catch block, display per-source errors with retry UI"

requirements-completed:
  - REG-08
  - REG-09

duration: 18min
completed: 2026-04-28
---

# Phase A Foundation Stabilization - Plan 04 Summary

**Frontend type, store, and UI updates for async model initialization and per-source download error display**

## Performance

- **Duration:** 18 min
- **Started:** 2026-04-28T08:07:40Z
- **Completed:** 2026-04-28T08:25:40Z
- **Tasks:** 3
- **Files modified:** 5

## Accomplishments

- Extended `ModelInfo` with state enum (`notDownloaded`/`loading`/`loaded`/`error`) and optional `checksum` field
- Added `SourceError` and `DownloadErrorResponse` TypeScript interfaces for typed error handling
- Updated `ModelStatus` with `state` field for lifecycle tracking in the Zustand store
- Replaced blocking `await invoke("init_model")` with fire-and-forget async call
- Added `setInterval` 500ms polling in App.tsx to monitor model async load via `list_models()`
- Auto-closes ModelDialog when model state transitions to `loaded`
- Refactored ModelDialog `handleDownload` to invoke `download_model` with `modelId` only (removed `sourceUrl` param)
- Added JSON parsing of `DownloadErrorResponse` in catch block for per-source error detail display
- Added retry button and manual URL input UI for failed downloads
- Replaced source selection radio buttons with informational source list (backend auto-fallback)

## Task Commits

Each task was committed atomically:

1. **Task 1: Update TypeScript types + Zustand store** - `affa7c5` (feat)
2. **Task 2: Update App.tsx — async init_model polling** - `8b1a169` (feat)
3. **Task 3: Update ModelDialog — per-source error + retry/manual URL** - `cf6cef1` (feat)

## Files Created/Modified

- `src/types/index.ts` - Extended ModelInfo (state, checksum), added SourceError, DownloadErrorResponse, updated ModelStatus with state field
- `src/store/useStore.ts` - Added `state: 'notDownloaded'` to modelStatus initial value
- `src/App.tsx` - Replaced blocking init_model with fire-and-forget, added setInterval 500ms polling, removed inline type annotation (uses ModelInfo)
- `src/components/ControlPanel.tsx` - Updated `m.loaded` reference to `m.state !== "loaded"`
- `src/components/ModelDialog.tsx` - Refactored download flow, added per-source error display, retry button, manual URL input

## Decisions Made

- **Fire-and-forget with polling**: Instead of blocking the app on `init_model()`, the init call is fire-and-forget with `.catch()` and a polling interval monitors state via `list_models()` — this unblocks the UI and supports the async model initialization from A-01.
- **useStore.getState() in polling**: Polling code uses `useStore.getState().modelDialogOpen` instead of a closure variable to avoid React stale reference bugs.
- **Informational source list**: Removed source selection radio buttons since A-02's backend implements automatic retry chain across all sources. Sources still shown for user transparency.
- **Manual URL via env var**: Manual URL input doesn't directly download; it tells the user to set `MODEL_URL` in the `.env` file. This ensures backend SHA256 verification (T-A-09 mitigation) is always enforced.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- **TypeScript errors from loaded -> state migration**: `App.tsx` had an inline type annotation still using `loaded: boolean`, and `ControlPanel.tsx` referenced `m.loaded` instead of `m.state`. Fixed as part of Task 1 by updating all references.

## Stub Tracking

No stubs introduced. All state transitions and error displays are fully wired.

## Threat Surface

No new threat surface introduced beyond what is documented in the plan's threat model (T-A-09 manual URL mitigated by backend SHA256 checksum, T-A-10 polling accepted as low-impact).

## Next Phase Readiness

- Frontend fully adapted to async model initialization (A-01 backend)
- Frontend ready for per-source download errors (A-02 backend)
- Phase B can build on the state-aware ModelInfo and polling infrastructure

---
*Phase: A-foundation-stabilization*
*Completed: 2026-04-28*

## Self-Check: PASSED

All 6 files confirmed present, all 3 commits confirmed in git history.
