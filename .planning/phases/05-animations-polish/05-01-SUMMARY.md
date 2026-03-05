# Summary: Plan 05-01

## Goal
Implement a core physics-based spring animation system and apply it to basic window transitions (mapping).

## Changes
- Created `src/animation.h` and `src/animation.c` with a spring-damper physics solver (Semi-implicit Euler).
- Updated `src/server.h`:
  - Added `heyde_animation_manager` to `heyde_server`.
  - Added `last_frame` timestamp to `heyde_output` for `dt` calculation.
- Updated `src/main.c`:
  - Initialized animation manager in `main`.
  - Updated `output_frame_handler` to calculate time delta and tick animations.
  - Implemented `toplevel_map_handler` to trigger a slide-up animation when a window is mapped.
- Updated `Makefile` to include the `animation.o` module.

## Verification Results
- Build successful in WSL (Arch Linux).
- Physics engine converges correctly (using `fabs` and `EPSILON`).
- Window mapping animation is integrated into the frame loop.
