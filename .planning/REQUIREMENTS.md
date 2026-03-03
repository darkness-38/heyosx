# Requirements

## 1. Core Branding
- Ensure all instances of branding explicitly state "heyOS" and "heyOS Team".
- Consistent visual identity across the bootloader, greeter, and desktop session.

## 2. Advanced Desktop UI/UX (heydm)
- **Fluid Animations:** Implement smooth, high-refresh-rate window transitions (open, close, minimize, focus change) similar to end-4 hyprland.
- **Visual Effects:** Integrate blur, rounded window corners, dynamic shadows, and detailed borders within the Smithay compositor.
- **Dynamic Workspaces:** Animated workspace switching and intuitive visual feedback for tiling/layout actions.
- **Integrated Status Bar:** A high-fidelity bar rendered via tiny-skia showing active workspaces, current time, and system status (network/battery).
- **Background Rendering:** Support for high-resolution wallpaper images or procedurally generated animated backgrounds.
- **Themed Cursor:** Replace the "white dot" with a proper themed cursor sprite that tracks accurately.

## 3. Greeter Enhancements (heygreeter)
- Polish the Slint-based UI to match the high-end animated aesthetic of the desktop session.
- Hardware-accelerated transitions into the user session.

## 4. Packaging & Distribution
- Automatically include any new UI assets and dependencies into the `mkarchiso` build process (`packages.x86_64`, `airootfs`).
- Ensure the live ISO seamlessly boots into the fully animated environment without manual configuration.

## 5. Compositor Stability & Rendering
- **Fix Black Screen:** Ensure the rendering pipeline correctly draws the desktop background, status bar, and all window elements.
- **Smithay Alignment:** Maintain compatibility with the latest Smithay API (0.3x+).
- **Damage Tracking:** Implement efficient damage tracking to ensure high performance during animations.

## 6. Layout & Interaction
- **Dynamic Tiling:** Refine the Master/Stack layout with smooth transitions between layouts.
- **Magnetic Focus:** The animated focus indicator must follow windows with a high-fidelity "glow" effect.
- **Accurate Hit-testing:** Ensure pointer interactions are pixel-perfect even during complex scale/slide animations.
