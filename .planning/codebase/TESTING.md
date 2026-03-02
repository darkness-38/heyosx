# Testing

## Current State
- **Automated Testing**: No automated tests found in the codebase (no `#[test]` in Rust files or dedicated test suites).
- **Manual Verification**: Testing is primarily performed by building the ISO via `build.sh` and booting it (likely in a VM like QEMU or VMware).
- **CI/CD**: No CI configuration found in the repository.

## Future Needs
- Unit tests for `heydm` state management and input handling.
- Integration tests for `heygreeter` IPC with `greetd`.
- Script validation (shellcheck) for `transTR` and build scripts.
