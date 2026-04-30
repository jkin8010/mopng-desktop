---
phase: B-plugin-descriptor-multi-model
plan: 07
type: execute
subsystem: "model-params-ui"
tags: [json-schema, auto-ui, radix-ui, accordion, model-params]
depends_on: [B-06]
provides: [MDL-04]
affects: [ControlPanel, Zustand store, Rust IPC]
tech-stack:
  added: [@radix-ui/react-accordion, shadcn accordion component]
  patterns:
    - "Pure mapping function: JSON Schema properties → ControlDescriptor[]"
    - "Auto-generated parameter UI without per-model hardcoding"
    - "Collapsible accordion wrapper for model parameter section"
key-files:
  created:
    - "src/components/json-schema/schemaToControls.ts"
    - "src/components/ModelParamControl.tsx"
    - "src/components/ModelParamSection.tsx"
    - "src/components/ui/accordion.tsx"
  modified:
    - "src/components/ControlPanel.tsx"
    - "src-tauri/src/models/mod.rs"
    - "package.json"
    - "tailwind.config.js"
metrics:
  duration: "5 min 31 sec"
  completed_date: "2026-04-30"
decisions: []
---

# Phase B Plugin Descriptor Multi-Model Plan 07: Model Parameter Auto-UI

## One-Liner

Auto-generated model parameter UI: a pure function maps JSON Schema param_schema to ControlDescriptor objects, rendered as Radix UI controls (Slider, Select, Switch, Number, Text) in a collapsible shadcn Accordion section, with values flowing through Zustand store to Rust process_image.

## Tasks Executed

### Task 1: Install shadcn accordion and create schemaToControls mapper

- Installed `@radix-ui/react-accordion` via `npx shadcn add accordion`
- Created `src/components/ui/accordion.tsx`
- Created `src/components/json-schema/schemaToControls.ts` — pure function mapping JSON Schema types to ControlDescriptors per D-13:
  - `{type: "number"}` → `"slider"` with min/max/step
  - `{type: "integer"}` → `"number"` input
  - `{type: "boolean"}` → `"switch"`
  - `{type: "string", enum: [...]}` → `"select"` with options
  - `{type: "string"}` (no enum) → `"text"` input
  - Unknown types default to `"text"` (safe fallback per T-B07-01)
  - Supports `"x-order"` for explicit property ordering (Pitfall 4)
  - Returns `[]` for null/empty schema
- Commits: `3b41122` (main), `82a04af` (deps/config side effects)

### Task 2: Create ModelParamControl and ModelParamSection

- Created `src/components/ModelParamControl.tsx` — renders single control using existing Radix UI primitives (Slider, Select, Switch, Input) with Label and value display
- Created `src/components/ModelParamSection.tsx` — collapsible shadcn Accordion wrapper:
  - Derives controls via `useMemo(() => schemaToControls(paramSchema, modelParams), [paramSchema, modelParams])`
  - Returns `null` when `controls.length === 0`
  - Default expanded state with ChevronDown icon rotation per UI-SPEC
  - Hidden default AccordionTrigger chevron via `[&>svg]:hidden`
- Commit: `0890088`

### Task 3: Integrate into ControlPanel and wire to Rust

- Imported `ModelParamSection` in `src/components/ControlPanel.tsx`
- Rendered `<ModelParamSection />` after model selector and before mode selector (per UI-SPEC Layout Contract)
- Updated `handleProcess` to send `modelParams: store.modelParams ?? {}` in invoke payload
- Added `model_params: serde_json::Value` field with `#[serde(default)]` to Rust `ProcessParams` struct in `src-tauri/src/models/mod.rs`
- Commit: `9dccf35`

## Verification Results

- `npx tsc --noEmit` — **PASSED** (no errors)
- `cargo check` — **PASSED** (exit code 0, 3 pre-existing warnings)
- All 4 created files exist at specified paths

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Duplicate accordion keyframes in tailwind.config.js**
- **Found during:** Post-install diff review
- **Issue:** `npx shadcn@latest add accordion` duplicated the `accordion-down` and `accordion-up` keyframes and animation entries (tabs indentation + duplicate values)
- **Fix:** Removed duplicate entries, restored unique keyframes/animation objects
- **Files modified:** `tailwind.config.js`
- **Commit:** `82a04af`

**2. [Rule 1 - Bug] shadcn install modified indentation to tabs**
- **Found during:** Post-install diff review
- **Issue:** The shadcn@4 tool converted `tailwind.config.js` from 2-space indentation to tabs, generating a noisier diff
- **Fix:** Restored original 2-space indentation style while keeping necessary changes
- **Files modified:** `tailwind.config.js`
- **Commit:** `82a04af`

## Threat Assessment

No new threat flags introduced. The `model_params` data flows through existing IPC trust boundaries (Zustand → Tauri invoke → Rust deserialization). Rust receives JSON values, and the existing threat model already covers this with:
- T-B07-03: Rust-side deserialization with `#[serde(default)]` ensures graceful handling of missing/invalid fields
- T-B07-01: Unrecognized JSON Schema types default to `"text"` input render, safe by design

## Known Stubs

None. The implementation is complete: all 5 control types are fully wired, empty schemas render nothing, and all values flow to Rust.

## Self-Check: PASSED

All 4 created files exist. All 4 commits exist in git log.
