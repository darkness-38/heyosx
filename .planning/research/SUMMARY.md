# Research Summary: HeyOS wlroots Compositor

**Domain:** Wayland Compositor Development (wlroots)
**Researched:** 2024-05-22
**Overall confidence:** HIGH

## Executive Summary

This research explores the implementation of a modern Wayland compositor using **wlroots (v0.18+)** and **C11**. The focus is on leveraging the **Scene Graph API (`wlr_scene`)** for efficient window management and rendering, as well as implementing advanced UI features like **rounded corners (12px)**, **Kawase blur**, and **spring animations** using GLES2 shaders and physics-based logic.

Key findings indicate that `wlr_scene` significantly simplifies the development process by handling damage tracking and buffer management automatically. For UI aesthetics, GLES2 shaders using **Signed Distance Fields (SDF)** provide high-quality antialiased rounded corners, while **Kawase blur** offers a performance-optimized alternative to Gaussian blur. Animations are best handled via a combination of **Cubic Bezier easing** for fixed-duration transitions and **damped harmonic oscillator** physics for interactive elements.

## Key Findings

**Stack:** wlroots (C11), GLES2, Meson/Ninja, XDG & Layer Shell protocols.
**Architecture:** Scene-graph based management with custom rendering nodes for shaders.
**Critical pitfall:** wlroots is "unstable" and frequently introduces breaking API changes; version locking is essential.

## Implications for Roadmap

Based on research, suggested phase structure:

1. **Phase 1: Core Boilerplate** - Establish the server, backend, and basic rendering using `wlr_scene`.
   - Addresses: Basic compositor loop, output management.
   - Avoids: Manual rendering complexity by using Scene Graph.

2. **Phase 2: Shell Integration** - Implement XDG-Shell and Layer-Shell.
   - Addresses: Window management and status bar/UI support.
   - Rationale: Necessary for a functional desktop environment.

3. **Phase 3: Visual Effects** - Integrate GLES2 shaders for rounded corners and blur.
   - Addresses: Aesthetics (12px corners, Kawase blur).
   - Needs deeper research: Custom `wlr_scene` nodes for shader injection.

4. **Phase 4: Motion Design** - Implement Bezier and Spring animation logic.
   - Addresses: Smooth transitions and interactive feedback.

**Phase ordering rationale:**
- Basic functionality (Phase 1-2) must precede visual polish (Phase 3-4) to ensure a stable testing environment.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | wlroots 0.18 is the current standard. |
| Features | HIGH | Protocols and shaders are well-documented in community. |
| Architecture | HIGH | `wlr_scene` is the recommended modern approach. |
| Pitfalls | MEDIUM | Community wisdom on performance for blur varies by hardware. |

## Gaps to Address

- Specific implementation of "blur regions" (blurring only what's behind a window) in `wlr_scene` requires careful handling of subsurfaces and damage.
- Integration of `heygreeter` (Rust/Slint) with the C11 compositor needs a defined IPC or socket protocol.
