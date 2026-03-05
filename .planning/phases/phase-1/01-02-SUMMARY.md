# Summary: Plan 01-02

## Goal
Implement graceful shutdown and signal handling for the core server.

## Changes
- Added `signal.h` include in `src/main.c`.
- Implemented `handle_signal` callback to terminate the Wayland display on `SIGINT` or `SIGTERM`.
- Registered signal handlers using `wl_event_loop_add_signal`.
- Added cleanup sequence after `wl_display_run`:
  - Removed signal sources.
  - Called `wl_display_destroy_clients`.
  - Called `wl_display_destroy`.
  - Called `wlr_output_layout_destroy`.

## Verification Results
- Build successful in WSL (Arch Linux).
- Signal handling logic is in place and uses the correct `wl_event_loop_add_signal` API.
- Cleanup sequence follows standard wlroots practices for 0.18.
