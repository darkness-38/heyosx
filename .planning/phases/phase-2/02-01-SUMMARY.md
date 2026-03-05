# Summary: Plan 02-01

## Goal
Implement output layout management and handling of new outputs.

## Changes
- Updated `src/server.h`:
  - Added `struct heyde_output` to manage output state.
  - Added `outputs` list and `new_output` listener to `heyde_server`.
- Updated `src/main.c`:
  - Implemented `server_new_output_handler` to handle new backend outputs.
  - Implemented `output_frame_handler`, `output_request_state_handler`, and `output_destroy_handler`.
  - Added logic to initialize outputs, set preferred modes, and add them to the scene graph.
- Updated `Makefile`:
  - Added support for `wayland-scanner` to generate protocol headers.
  - Added `xdg-shell-protocol.c/h` generation.
  - Added `-I.` to `CFLAGS`.

## Verification Results
- Build successful in WSL (Arch Linux).
- Output management logic follows wlroots 0.18 standards.
