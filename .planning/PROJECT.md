# heyDE: The heyOS Wayland Compositor

**Vision:**  
A from-scratch Wayland compositor for **heyOS**, written in C11 using **wlroots**. It replicates the "end-4" (Illogical Impulse) aesthetic—characterized by fluid spring animations, soft shadows, rounded corners, and vibrant glassmorphism—without relying on Hyprland.

## Core Identity
- **Name:** heyDE (hey Desktop Environment)
- **Target OS:** heyOS (Arch Linux-based)
- **Engine:** wlroots (C11)
- **Aesthetic:** "end-4" / Illogical Impulse
- **Key Visuals:** Bezier-curve spring animations, 12px rounded corners, Kawase blur, Material You (Monet) dynamic theming.

## Strategic Objectives
1. **Performance:** Lightweight C11 implementation with efficient GLES2 rendering.
2. **Aesthetics:** Production-grade visual polish (shadows, blur, smooth motion).
3. **Modularity:** Clean separation of concerns (server, input, rendering, shells).
4. **heyOS Native:** Seamless integration with existing `heygreeter` and system scripts.

## Milestones
- [x] **Milestone 1: Core Foundation** — Basic wlroots server, XDG-Shell, and GLES2 rendering.
- [x] **Milestone 2: System Integration & Polish** — Fix permissions (heydm), stable session handoff, and VM compatibility.
- [x] **Milestone 3: Fix heyDE Rendering** — Fix black screen, implement background, and enable custom shaders.
- [ ] **Milestone 4: heyWM & Interaction** — Better window management (closing, focusing, floating/tiling).

## Tech Stack
- **Language:** C11
- **Libraries:** `wlroots`, `wayland-server`, `libinput`, `xkbcommon`, `pixman-1`, `OpenGL ES 2.0`.
- **Build System:** Makefile (pkg-config).
- **Rendering:** GLES2 with custom fragment shaders.
