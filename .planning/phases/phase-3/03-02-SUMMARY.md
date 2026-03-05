# Summary: Plan 03-02

## Goal
Initialize input handling infrastructure (seat, cursor, keyboard).

## Changes
- Updated `src/server.h`:
  - Added `wlr_cursor`, `wlr_xcursor_manager`, `wlr_seat`, and input-related listeners.
  - Defined `struct heyde_keyboard` for tracking per-device keyboard state.
  - Added list of keyboards to `heyde_server`.
- Updated `src/main.c`:
  - Initialized cursor, seat, and xcursor manager in `main`.
  - Implemented `server_new_input_handler` to handle new keyboards and pointers.
  - Implemented base handlers for cursor events (motion, button, axis, frame) and keyboard events (modifiers, key, destroy).
  - Attached cursor to output layout for seamless movement across outputs.

## Verification Results
- Build successful in WSL (Arch Linux).
- Input infrastructure correctly initialized according to wlroots 0.18 standards.
