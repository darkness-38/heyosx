# Concerns

## Technical Debt
- **Lack of Tests**: Zero automated test coverage for core components (`heydm`, `heygreeter`).
- **Script Duplication**: `transTR` scripts are mostly one-line wrappers; maintaining 50+ individual scripts might be cumbersome.

## Security
- **Sudoers Config**: `airootfs/etc/sudoers.d/00-heyos` should be reviewed for permissions.
- **PAM Integration**: `heygreeter` handles PAM authentication; security audits are recommended.

## Stability
- **Smithay Integration**: Using `smithay` from git (latest master/branch) in `heydm/Cargo.toml` may lead to breaking changes during builds.
- **ISO Build Dependencies**: Relies on host environment tools (mkarchiso) and internet access for package retrieval.

## Maintainability
- Hardcoded paths in scripts (e.g., `/usr/local/bin/transTR`).
- Turkish-only command names in `transTR` might limit non-Turkish speaking contributors, though this is by design for the OS.
