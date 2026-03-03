# Plan: Phase 3: Fluid Animations

## Goal
Introduce smooth, high-refresh-rate window transitions (fade/scale/easing) similar to modern Wayland compositors (like Hyprland).

## Tasks

### 1. Update `WindowElement` for Animation State
- **Action:** In `heydm/src/window.rs`, add `current_position`, `current_size`, `opacity`, and `scale` fields to `WindowElement`.
- **Action:** Initialize these in `WindowElement::new`.
- **Verification:** Ensure the project still compiles.

### 2. Implement Animation Update Logic
- **Action:** In `heydm/src/window.rs`, add `update_animations(&mut self, dt: std::time::Duration)` to `WindowManager`.
- **Action:** Iterate through windows and interpolate current values towards target values using a simple easing formula (e.g., `current += (target - current) * factor * dt`).
- **Verification:** Log animation steps during testing to confirm values are changing.

### 3. Integrate with Main Event Loop
- **Action:** In `heydm/src/state.rs`, modify the `run_winit` loop to calculate delta time (dt) between frames.
- **Action:** Call `state.window_manager.update_animations(dt)` before each render pass.
- **Verification:** Confirm the loop remains stable and responsive.

### 4. Update Rendering to use Animated Values
- **Action:** In `heydm/src/render.rs`, modify `gather_elements` and `draw_ui` to use `current_position`, `current_size`, `scale`, and `opacity` from `WindowElement`.
- **Action:** Ensure Wayland surfaces are positioned according to their *animated* coordinates.
- **Verification:** Run `heydm` and move/resize windows to see the smooth transitions.

### 5. Update Input Handling for Animated Windows
- **Action:** In `heydm/src/window.rs`, update `contains_point`, `surface_under`, and `handle_pointer_motion` to use the animated `current_position` and `current_size` for hit-testing and relative motion.
- **Verification:** Verify that windows can be clicked and dragged smoothly even during animations.
