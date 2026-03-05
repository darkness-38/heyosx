# Summary: Plan 03-01

## Goal
Implement support for the Layer-Shell protocol to allow UI elements like bars.

## Changes
- Updated `Makefile` to generate `wlr-layer-shell-unstable-v1-protocol.h/c` from `/usr/share/wlr-protocols/unstable/wlr-layer-shell-unstable-v1.xml`.
- Updated `src/server.h`:
  - Added `struct heyde_layer_surface` to manage layer surface state.
  - Added `layer_shell`, `layer_surfaces`, and `new_layer_shell_surface` listener to `heyde_server`.
  - Added `layers` array to `heyde_server` to store scene trees for each shell layer.
- Updated `src/main.c`:
  - Initialized `wlr_layer_shell_v1` in `main`.
  - Created scene trees for each layer (background, bottom, top, overlay).
  - Implemented `server_new_layer_shell_surface_handler` and associated listeners (`map`, `unmap`, `destroy`, `surface_commit`).
  - Correctly placed layer surfaces in their respective scene trees.

## Verification Results
- Build successful in WSL (Arch Linux).
- Layer-shell protocol is integrated into the compositor.
