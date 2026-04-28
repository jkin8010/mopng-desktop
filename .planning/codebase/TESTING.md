# Testing Patterns

**Analysis Date:** 2026-04-28

## Test Framework

**Runner:**
- Not configured. No test runner (Jest, Vitest, Mocha, or any other) is installed as a dependency in `package.json`.

**Assertion Library:**
- Not configured. No assertion library detected.

**Run Commands:**
- No test commands exist in `package.json` scripts. The only available commands are:
  ```bash
  pnpm dev              # Vite development server
  pnpm tauri dev        # Tauri development mode
  pnpm tauri build      # Production build
  pnpm build            # TypeScript check + Vite build
  ```

## Test File Organization

**Status: No test files exist anywhere in the codebase.**

- No files matching `*.test.*`, `*.spec.*` patterns were found
- No `__tests__/` directories exist
- No test configuration files (`jest.config.*`, `vitest.config.*`) present
- The `.gitignore` includes a `coverage/` entry (standard template entry), but no coverage tool is configured

## Test Structure

**Not applicable.** No test files exist to infer suite organization patterns.

## Mocking

**Framework:** Not applicable.

**Patterns:** Not applicable.

## Fixtures and Factories

**Test Data:** Not applicable.

## Coverage

**Requirements:** None enforced. `coverage/` is in `.gitignore`, but no coverage tool is installed or configured.

**View Coverage:** No command available.

## Test Types

**Unit Tests:**
- No unit testing infrastructure exists.

**Integration Tests:**
- No integration testing infrastructure exists.

**E2E Tests:**
- Not used. No Playwright, Cypress, or any E2E framework detected.

## Common Patterns

**Not applicable.** No testing patterns to document.

## Recommendations for Adding Tests

Based on the codebase structure and technology stack:

**Recommended Test Stack:**
- **Runner:** Vitest (natively integrates with Vite, already in project)
- **Component Testing:** React Testing Library (@testing-library/react)
- **Mocking:** Vitest built-in mocking or `vi.mock()`
- **Coverage:** Vitest has built-in coverage via c8/istanbul

**Key Areas to Test (by priority):**
1. Zustand store logic in `src/store/useStore.ts` (pure state transitions)
2. Utility functions in `src/lib/utils.ts` and `src/lib/id.ts`
3. Matting engine render logic in `src/components/konva/mattingEngine.ts`
4. Tauri hook interactions in `src/hooks/useTauri.ts`
5. Type validation and defaults in `src/types/index.ts`

**Co-location Pattern to Follow:**
Place test files next to source files following this convention:
```
src/store/useStore.test.ts
src/lib/utils.test.ts
src/components/konva/mattingEngine.test.ts
src/hooks/useTauri.test.ts
```

**Store Testing Pattern (Zustand):**
1. Create a fresh store instance per test
2. Test each action directly via `useStore.getState().actionName()`
3. Assert state changes with `useStore.getState().stateKey`

**Engine Testing Pattern (Konva):**
1. Test pure functions (`compositeWithMask`, `createCheckerCanvas`, `clamp`, `parseAlpha`)
2. Test factory output contract (engine API methods exist and are callable)
3. Mock Image and Canvas contexts as needed

**Tauri Testing Pattern:**
1. Mock `@tauri-apps/api/core` invoke calls using `vi.mock()`
2. Test that `useTauri` hook returns correct methods
3. Test that side effects (state updates) fire correctly on invoke success/failure

---

*Testing analysis: 2026-04-28*
