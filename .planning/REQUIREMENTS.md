# Requirements: heyDE Compositor

## Functional Requirements (FR)

### FR-1: Core Server & Output
- **FR-1.1:** Initialize `wlr_backend`, `wlr_renderer`, and `wlr_allocator`.
- **FR-1.2:** Handle multi-monitor layouts using `wlr_output_layout`.
- **FR-1.3:** Implement a clean shutdown sequence (SIGINT/SIGTERM) with proper resource cleanup.

### FR-2: Shell Support
- **FR-2.1: XDG-Shell:** Full support for standard application windows (toplevels) and popups.
- **FR-2.2: Layer-Shell:** Support for desktop UI elements (bars, docks, wallpapers) with exclusive zones.

### FR-3: Input Handling
- **FR-3.1:** Integrated `libinput` for keyboard and mouse events.
- **FR-3.2:** Custom keybinding system using `xkbcommon`.
- **FR-3.3:** Support for 3-finger swipe gestures for workspace switching.

### FR-4: Visual Effects (GLES2)
- **FR-4.1: Rounded Corners:** 12px antialiased corners using SDF shaders.
- **FR-4.2: Shadows:** 20px soft Gaussian drop-shadows.
- **FR-4.3: Blur:** Multi-pass Kawase blur for background and window layers.
- **FR-4.4: Dynamic Colors:** Implementation of Material You (Monet) dynamic color logic.

### FR-5: Animations
- **FR-5.1:** Physics-based spring animations using Bezier curves (`p1: [0.05, 0.9, 0.1, 1.05]`).
- **FR-5.2:** Targeted animations for `window_open`, `window_close`, and `workspace_swipe`.

## Non-Functional Requirements (NFR)

### NFR-1: Performance
- Maintain 60+ FPS on standard hardware; optimize blur passes for lower-end GPUs.
- Minimal memory footprint (aim for <50MB resident).

### NFR-2: Maintainability
- Modular C11 codebase (separated by responsibility).
- No hardcoded paths for assets/configs.

### NFR-3: Compatibility
- Run seamlessly on Arch Linux (heyOS).
- Support for VMware/VirtualBox via software rendering fallback (pixman/llvmpipe).
