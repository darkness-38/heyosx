# Requirements: Milestone 5 (Monet & Theming)

## Goal
Implement a dynamic, Material You-inspired (Monet) theming system that sets the visual tone for the entire heyOS desktop environment.

## Functional Requirements
1. **Dynamic Palette Generation:** The compositor should be able to update its color palette (accent, background, surface) dynamically.
2. **Glassmorphism:** Implementing "glass" surfaces with soft transparency (surface color with 0.8 alpha) and subtle borders.
3. **Animated Transitions:** Colors should transition smoothly when changed (e.g., using the animation manager).
4. **System Integration:** Provide a way for external tools (like a control center) to trigger color updates.

## Technical Requirements
- Extend `monet.c` to support "palette" updates.
- Integrate the `monet_colors` into the rendering pipeline (using them in shaders).
- Create a `Logo+M` keybinding to "cycle" through 3-4 preset "end-4" palettes for testing.
- Use the `heyde_animation_manager` to interpolate color changes over 500ms.
