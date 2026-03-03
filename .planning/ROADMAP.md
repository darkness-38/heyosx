# Project Roadmap

## Phase 1: Branding Consistency Update (Completed)
- **Goal:** Unify the "heyOS" branding across bootloader, greeter, and desktop session.

## Phase 2: heydm Foundation for Advanced UI (Completed)
- **Goal:** Upgrade the compositor (`heydm`) rendering pipeline to support advanced visual effects (blur, rounded corners).

## Phase 3: Fluid Animations (Completed)
- **Goal:** Introduce smooth, high-refresh-rate window transitions similar to end-4 hyprland.

## Phase 4: Greeter Polish and Packaging (Completed)
- **Goal:** Enhance `heygreeter` visuals and ensure everything is correctly packaged into the ISO.

## Phase 5: Fix Build and Smithay Alignment (Completed)
- **Goal:** Address all compilation errors and align `heydm` with the latest Smithay version.

## Phase 6: Advanced Tiling Layouts (Completed)
- **Goal:** Implement dynamic master/stack tiling logic with configurable gaps.

## Phase 7: Animation and Interaction Polish (Completed)
- **Goal:** Implement workspaces, workspace switching animations, and refined interaction feedback.

## Phase 8: Debug & Fix Rendering Pipeline (Completed)
- **Goal:** Resolve the "black screen" issue and ensure the background and UI are correctly rendered.
- **Tasks:**
  - Investigate and fix `render_texture_from_to` issues for the UI texture.
  - Fix cursor tracking/inversion (ensure the "white dot" follows the actual pointer).
  - Add debug logging for the rendering loop.
- **Success Criteria:** The desktop background and status bar shells are visible; the cursor dot follows the pointer accurately.

## Phase 9: High-Fidelity UI & Components (Completed)
- **Goal:** Implement the final "end-4 hyprland" inspired UI.
- **Plans:** 3 plans
- **Requirements:** FEAT-UI-01 through FEAT-UI-06
- **Tasks:**
  - [ ] 09-01-PLAN.md — Status Bar & Text Rendering (Clock, Workspaces).
  - [ ] 09-02-PLAN.md — Living Background & Glassmorphism effects.
  - [ ] 09-03-PLAN.md — Premium Magnetic Focus & Status Icons.
- **Success Criteria:** The desktop looks professional and modern, matching the aesthetic goals.

## Phase 10: Integration & Performance
- **Goal:** Finalize the OS for ISO distribution.
- **Tasks:**
  - Optimize rendering performance (damage tracking).
  - Final ISO build and end-to-end verification.
  - Documentation and cleanup.
- **Success Criteria:** heyOS builds into a high-performance ISO that boots into a beautiful, functional desktop.
