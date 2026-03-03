# Phase 3: Fluid Animations - Research

**Researched:** 2025-03-24
**Domain:** Animation systems, easing functions, and event loop integration in Smithay/Calloop.
**Confidence:** HIGH

## Summary

This research identifies the necessary hooks in `heydm` to implement a fluid animation system. The current architecture uses `calloop` for the event loop and `winit` for the nested backend. Animations can be integrated by extending `WindowElement` with interpolated state and adding a per-frame update call in the main loop.

**Primary recommendation:** Implement a centralized `AnimationManager` that handles easing calculations and updates `WindowElement` properties (position, size, opacity, scale) before each render pass.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `calloop` | 0.14 | Event loop & Timers | Standard for Smithay-based compositors. |
| `smithay` | git | Compositor framework | Provides the foundation for window management and rendering. |
| `tiny-skia` | 0.11 | UI Rendering | Used for high-fidelity window decorations (shadows/borders). |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `interpolation` | - | Easing functions | Optional; can be hand-rolled for simplicity. |

## Architecture Patterns

### Recommended Project Structure
```
heydm/src/
├── animation.rs     # NEW: Animation manager and easing functions
├── state.rs         # MODIFIED: Main loop update hook
├── window.rs        # MODIFIED: Animated window properties
└── render.rs        # MODIFIED: Use animated properties for drawing
```

### Pattern 1: Interpolated State
Instead of updating window geometry immediately, store a "target" geometry and a "current" geometry. The renderer uses the "current" geometry, which is updated towards the "target" every frame.

**When to use:** Window movement, resizing, and opening/closing transitions.

### Anti-Patterns to Avoid
- **Blocking the Event Loop:** Don't perform heavy calculations in the animation update.
- **Fixed Timestep issues:** Ensure animations are frame-rate independent by using the elapsed time (Delta Time).
- **Z-order flickering:** Ensure animations don't accidentally change the stack order of windows mid-transition.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Spring Physics | Complex solvers | `spring` crate or simple formula | Edge cases in spring physics (overshoot, damping) are tricky. |
| Event Loop | Custom `poll()` | `calloop` | Smithay is built around `calloop`; manual polling breaks integration. |

## Common Pitfalls

### Pitfall 1: Frame Snapping
**What goes wrong:** Animations look "jittery" or snap to integers.
**Why it happens:** Using `i32` for intermediate animation steps.
**How to avoid:** Store animation state in `f32` or `f64` and only convert to `i32` at the final rendering step if required by the API.
**Warning signs:** Visible stuttering during slow transitions.

### Pitfall 2: Input/Output Mismatch
**What goes wrong:** A window is visually at one position, but input (clicks) is processed at its target position.
**Why it happens:** Only updating the visual state and not the "input geometry" used for hit-testing.
**How to avoid:** Ensure `WindowManager::surface_under` uses the *current* animated position for hit-testing.

## Code Examples

### 1. Proposed `WindowElement` Extension
```rust
// heydm/src/window.rs

pub struct WindowElement {
    pub toplevel: ToplevelSurface,
    // Target state (what the WM wants)
    pub target_position: Point<i32, Logical>,
    pub target_size: Size<i32, Logical>,
    
    // Animation state
    pub current_position: Point<f32, Logical>,
    pub current_size: Size<f32, Logical>,
    pub opacity: f32,
    pub scale: f32,
}
```

### 2. Main Loop Hook
```rust
// heydm/src/state.rs (run_winit)

let mut last_frame = Instant::now();
while running {
    let now = Instant::now();
    let dt = now.duration_since(last_frame);
    last_frame = now;

    // Update animations
    state.window_manager.update_animations(dt);

    // Render frame...
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Snap-to-grid | Spring/Easing | Recent (Hyprland style) | Modern, "fluid" feel that heyOS aims for. |
| Global redraw | Damage tracking | Smithay default | Higher performance, though fluid animations often damage large areas. |

## Open Questions

1. **Subpixel Surface Positioning:** Does `WaylandSurfaceRenderElement` support `f64` offsets in the current Smithay version?
   - What we know: `render_elements_from_surface_tree` takes `Point<i32, Logical>`.
   - What's unclear: If there's a higher-level API or transform that allows `f64` positioning of the Wayland surface itself.
   - Recommendation: Start with `i32` snapping for the surface but use `f32` for the `tiny-skia` decorations to hide the snapping.

2. **Animation Manager Ownership:** Should the `AnimationManager` own the windows, or should `WindowManager` own the animations?
   - Recommendation: Keep animations inside `WindowElement` for easier state management, but use a helper trait or module for the math.

## Sources

### Primary (HIGH confidence)
- `heydm/src/state.rs` - Main event loop location verified.
- `heydm/src/window.rs` - Window state storage location verified.
- `smithay` Source/Docs - Patterns for rendering and element gathering.

### Secondary (MEDIUM confidence)
- `calloop` 0.14 Docs - Timer and EventSource integration patterns.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Libraries are already in use.
- Architecture: HIGH - Integration points are clear.
- Pitfalls: MEDIUM - Input/Output mismatch is a common risk in custom WMs.

**Research date:** 2025-03-24
**Valid until:** 2025-04-23
