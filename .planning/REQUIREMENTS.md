# Requirements: Milestone 2 (System Integration & Polish)

## Goal
Ensure the heyOS live environment boots reliably into `hey-greeter` and correctly transitions to `heyDE` or other selected sessions with proper permissions and environment variables.

## Functional Requirements
1. **Binary Persistence:** All critical binaries (`heydm`, `hey-greeter`, `heyde`) must be built and copied to the correct paths in `airootfs` by the `build.sh` script.
2. **Execute Permissions:** Ensure `chmod 755` is applied to all binaries and scripts in the ISO.
3. **Session Handoff:** `hey-greeter` must be able to launch `heyde` or other Wayland sessions without permission errors.
4. **VM Stability:** Software rendering defaults must be robustly set for the greeter and the compositor in VM environments.

## Technical Requirements
- Update `build.sh` to explicitly handle `heydm` (as a copy of `hey-greeter`).
- Verify `greetd` configuration points to the correct launcher.
- Standardize environment variables for software rendering across `/etc/environment` and local wrappers.
