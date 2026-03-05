# Roadmap: heyDE Development (Milestone 5)

## Phase 14: Monet Core (Palette Logic)
**Goal:** Implement dynamic color palettes.
- [ ] **Task 14.1:** Add `heyde_monet_set_palette` in `src/monet.c` to allow updating colors.
- [ ] **Task 14.2:** Define 3-4 "end-4" inspired palettes (Blue/Purple, Green/Nature, Red/Sunset, Monochrome).
- [ ] **Task 14.3:** Add `Logo+M` keybinding in `main.c` to cycle palettes.

## Phase 15: Theming Integration (Visuals)
**Goal:** Apply Monet colors to the scene graph.
- [ ] **Task 15.1:** Update the background rectangle's color when the palette changes.
- [ ] **Task 15.2:** Implement shader uniforms for `accent` and `surface` colors to be used by windows.
- [ ] **Task 15.3:** Basic color interpolation for smooth transitions using `heyde_animation`.

## Phase 16: System Polish
**Goal:** Finalize the "heyOS look".
- [ ] **Task 16.1:** Add "glass" border to focused windows using the accent color.
- [ ] **Task 16.2:** Ensure layer-shell surfaces (panels) use the `surface` color from Monet.
- [ ] **Task 16.3:** Visual verification on all workspaces.
