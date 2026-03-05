# Summary: Plan 01-01

## Goal
Initialize the base wlroots environment by defining the core server state, implementing the main event loop, and setting up the build system.

## Changes
- Created `src/server.h` with the `heyde_server` structure.
- Created `src/main.c` with wlroots initialization logic, including display, backend, renderer, allocator, scene, and output layout.
- Created `Makefile` using `pkg-config` for dependencies and `WLR_USE_UNSTABLE` flag.
- Fixed wlroots 0.18 specific initialization calls (`wlr_backend_autocreate` and `wlr_output_layout_create`).
- Suppressed unused parameter warnings in `main.c`.

## Verification Results
- Build successful in WSL (Arch Linux) using `make`.
- Binary `heyde` produced.
- Initialization flow follows wlroots 0.18 standards.
