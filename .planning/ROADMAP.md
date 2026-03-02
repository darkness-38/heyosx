# Project Roadmap

## Phase 1: Branding Consistency Update
- **Goal:** Unify the "heyOS" branding across bootloader, greeter, and desktop session.
- **Tasks:**
  - Update greeter UI (`heygreeter/ui/greeter.slint`) to explicitly state "heyOS" and "heyOS Team".
  - Verify and update `airootfs` (like `/etc/issue`, `/etc/os-release`) to have the correct branding strings.
  - Test the build to confirm visual consistency.
- **Success Criteria:** System clearly displays the correct branding everywhere.

## Phase 2: heydm Foundation for Advanced UI
- **Goal:** Upgrade the compositor (`heydm`) rendering pipeline to support advanced visual effects (blur, rounded corners).
- **Tasks:**
  - Investigate Smithay `glow` backend capabilities for shader-based blur and rounded corners.
  - Implement basic rounded corners for XDG and layer shell windows.
  - Add drop shadows to active windows.
- **Success Criteria:** Windows have rounded corners and shadows without significant performance drop.

## Phase 3: Fluid Animations
- **Goal:** Introduce smooth, high-refresh-rate window transitions similar to end-4 hyprland.
- **Tasks:**
  - Implement an animation system/loop inside `heydm` for tracking window states.
  - Add fade and scale-in animations for new windows.
  - Add fade and scale-out animations for closing windows.
  - Implement workspace transition animations.
- **Success Criteria:** Window operations feel fluid and responsive at 60+ FPS.

## Phase 4: Greeter Polish and Packaging
- **Goal:** Enhance `heygreeter` visuals and ensure everything is correctly packaged into the ISO.
- **Tasks:**
  - Add hardware-accelerated entry transitions to `heygreeter`.
  - Update `packages.x86_64` and `build.sh` if new UI dependencies were added.
  - Generate the final ISO and perform end-to-end testing.
- **Success Criteria:** System builds fully and the live ISO boots directly into the fully customized environment.
