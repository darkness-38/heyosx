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

## Phase 6: Advanced Tiling Layouts
- **Goal:** Implement dynamic master/stack tiling logic with configurable gaps.
- **Plans:** 3 plans
- **Requirements:** FEAT-TILING-01, FEAT-TILING-02, FEAT-TILING-03
- **Tasks:**
  - [ ] 06-01-PLAN.md — Refactor WindowManager and add layout state infrastructure.
  - [ ] 06-02-PLAN.md — Implement Master/Stack logic and gap geometry.
  - [ ] 06-03-PLAN.md — Add input bindings and tiling controls.
- **Success Criteria:** Windows automatically tile according to the master/stack strategy with gaps.

## Phase 7: Animation and Interaction Polish
- **Goal:** Refine existing animations and ensure input handling is flawless during transitions.
- **Tasks:**
  - Implement workspace switching animations.
  - Fix hit-testing for windows currently in motion (animated state).
  - Add focus animation for window transitions.
- **Success Criteria:** System interaction feels "premium" and animations are perfectly smooth at high frame rates.
