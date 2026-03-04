# Roadmap: heyDE Development

## Phase 1: Scaffolding & Core Server (Current)
**Goal:** Initialize the wlroots environment and display an empty output.
**Plans:** 2 plans
- [ ] 01-01-PLAN.md — Core Composition (Scaffolding)
- [ ] 01-02-PLAN.md — Robustness (Signals & Cleanup)

## Phase 2: Output & XDG-Shell
**Goal:** Display application windows on the screen.
- [ ] **Task 2.1:** Implement output layout management.
- [ ] **Task 2.2:** Add `wlr_xdg_shell` integration for toplevel surfaces.
- [ ] **Task 2.3:** Basic window management (focus, stacking).

## Phase 3: Layer-Shell & Input
**Goal:** Support UI elements (bars) and interact with windows.
- [ ] **Task 3.1:** Add `wlr_layer_shell_v1` support.
- [ ] **Task 3.2:** Implement input handling (mouse/keyboard).
- [ ] **Task 3.3:** Basic keybindings (e.g., Terminal, Kill window).

## Phase 4: GLES2 Rendering & Shaders
**Goal:** Achieve the "end-4" aesthetic.
- [ ] **Task 4.1:** Implement custom GLES2 renderer with SDF rounded corners.
- [ ] **Task 4.2:** Add soft Gaussian shadows.
- [ ] **Task 4.3:** Implement multi-pass Kawase blur.
- [ ] **Task 4.4:** Dynamic theming (Monet) logic.

## Phase 5: Animations & Polish
**Goal:** Fluid motion and final refinements.
- [ ] **Task 5.1:** Physics-based spring animation system.
- [ ] **Task 5.2:** Workspace swipe gestures (3-finger).
- [ ] **Task 5.3:** Final stability and performance tuning.
