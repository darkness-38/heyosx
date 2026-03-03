# Plan: Phase 2: heydm Foundation for Advanced UI

## Goal
Upgrade the compositor (`heydm`) rendering pipeline to support advanced visual effects (blur, rounded corners, shadows).

## Tasks

### 1. Refactor `Renderer::draw_ui`
- **Action:** Modify `Renderer::draw_ui` signature to `pub fn draw_ui(state: &HeyDM) -> Pixmap`.
- **Action:** Update `state.rs` to pass `state` to `draw_ui`.
- **Verification:** Ensure the project compiles and the background/panel still render correctly.

### 2. Implement Drop Shadows for Windows
- **Action:** In `draw_ui`, iterate over the windows and draw a blurred, semi-transparent rectangle behind each window's geometry.
- **Verification:** Run `heydm` and check for shadows behind windows.

### 3. Implement Rounded Borders for Windows
- **Action:** In `draw_ui`, draw rounded rectangles for window borders instead of the sharp clear rectangles in `render_frame`.
- **Action:** Update `render_frame` to remove the old sharp border drawing logic.
- **Verification:** Run `heydm` and check for rounded window borders.

### 4. Investigate Surface Masking (Shader Integration)
- **Action:** Research how to apply a mask or use a custom shader in Smithay's `GlowRenderer` to round the actual Wayland surface corners.
- **Verification:** Identify if this is feasible in Phase 2 or should be moved to Phase 3.
