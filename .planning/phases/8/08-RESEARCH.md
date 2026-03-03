# Phase 8: Debug & Fix Rendering Pipeline - Research

**Researched:** 2024-03-03
**Domain:** Smithay 0.3x Rendering & Input
**Confidence:** MEDIUM

## Summary

This research addresses the rendering and input issues in `heydm`. The "black screen" is likely caused by coordinate system mismatches in `render_texture_from_to` or incorrect texture importing. The "inverted cursor" is a common issue when mapping absolute backend events to compositor space.

**Primary recommendation:** Use `Rectangle::from_loc_and_size((0.0, 0.0), size)` for the source rectangle in `render_texture_from_to` and verify winit's coordinate transformation for absolute pointer events.

<user_constraints>
## User Constraints (from CONTEXT.md)

*Note: No CONTEXT.md was found, following ROADMAP.md and PROJECT.md requirements.*

### Locked Decisions
- Use Smithay 0.3x.
- Use tiny-skia for UI rendering.
- Keep the "white dot" cursor design.

### Claude's Discretion
- Implementation details of the rendering pipeline.
- Input mapping logic.

### Deferred Ideas (OUT OF SCOPE)
- Damage tracking optimization (deferred to Phase 10).
</user_constraints>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Smithay | 0.3.x | Wayland Compositor Framework | Industry standard for Rust compositors. |
| tiny-skia | 0.11 | CPU Rendering | High-quality 2D graphics with a simple API. |

## Architecture Patterns

### render_texture_from_to Coordinate Systems
- **Source Rectangle (`src`):** Expressed in **buffer coordinates**. If you import a Pixmap of size 1920x1080, the source rectangle should be `(0, 0, 1920, 1080)`.
- **Destination Rectangle (`dst`):** Expressed in **physical output coordinates**.
- **Transform:** Smithay applies the transform to the source texture before rendering. Usually `Transform::Normal` is desired for pre-rendered UI.

### Inverted Cursor & Absolute Input
- **Winit Backend:** `PointerMotionAbsoluteEvent::x_transformed` and `y_transformed` return coordinates scaled to the provided width/height.
- **Inversion Cause:** If the cursor moves up when the mouse moves down, the `y` coordinate is being interpreted incorrectly. Some backends might provide Y-down (standard) or Y-up (OpenGL style). Smithay/Wayland expects Y-down (0 at top).
- **Mapping:** `state.window_manager.set_cursor_position(x, y)` should ensure `y` is not being subtracted from height accidentally.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Coordinate Transform | Custom mapping logic | `event.x_transformed()` | Handles backend-specific quirks. |
| Memory Import | Manual GL texture creation | `ImportMem::import_memory` | Handles alignment and format conversion. |

## Common Pitfalls

### Pitfall 1: Incorrect Source Rectangle
**What goes wrong:** Black screen or distorted UI.
**Why it happens:** Passing logical coordinates to a function expecting buffer-relative coordinates.
**How to avoid:** Always use `(0, 0)` to `(width, height)` of the actual texture for the source.

### Pitfall 2: Texture Format Mismatch
**What goes wrong:** Red/Blue swap or transparency issues.
**Why it happens:** tiny-skia (RGBA) vs Smithay expectations (often BGRA or vice versa).
**How to avoid:** Verify `Fourcc` format during import.

## Code Examples

### Correct `render_texture_from_to` Usage
```rust
let size = ui_texture.size(); // Physical size of the imported texture
frame.render_texture_from_to(
    ui_texture,
    Rectangle::from_loc_and_size((0.0, 0.0), size.to_f64()), // Source: Full texture
    Rectangle::new((0, 0).into(), output_size),             // Dest: Full output
    &[], // damage
    &[], // opaque region
    Transform::Normal,
    1.0, // alpha
)?;
```

## Sources

### Primary (HIGH confidence)
- Smithay Book / API Docs - Rendering section.
- `smithay::backend::renderer::Frame` trait documentation.

### Tertiary (LOW confidence)
- Interrupted investigation due to time limits.

## Metadata
**Research date:** 2024-03-03
**Valid until:** 2024-04-03
