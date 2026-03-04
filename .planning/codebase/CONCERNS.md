# Codebase Concerns

**Analysis Date:** 2024-03-04

## Tech Debt

**[transTR Translation Layer]:**
- Issue: Over 40 tiny bash scripts in `airootfs/usr/local/bin/transTR/` that simply wrap single commands (e.g., `pacman -Syu`). This creates significant maintenance overhead.
- Files: `airootfs/usr/local/bin/transTR/*`
- Impact: Updating package manager flags or adding features requires modifying dozens of files.
- Fix approach: Consolidate into a single script with subcommands or use aliases/functions in a shell profile.

**[Hardcoded Session Logic]:**
- Issue: The greeter explicitly filters for "hyprland", "plasma", and "gnome".
- Files: `heygreeter/src/main.rs`
- Impact: Adding support for a new desktop environment (like XFCE or Sway) requires a code change and recompile.
- Fix approach: Remove hardcoded filters and instead parse `.desktop` files for valid session types.

## Security Considerations

**[Hardcoded Live Credentials]:**
- Risk: Default credentials `hey:hey` and `root:heyos` are set during build.
- Files: `airootfs/root/customize_airootfs.sh`
- Current mitigation: Typical for live ISOs, but poses a risk if users install the system without changing them.
- Recommendations: Ensure the installer forces a password change (it currently asks, but does not validate against these defaults).

**[Plaintext Password Handling]:**
- Risk: Passwords are handled as standard Rust `String` objects and Slint `SharedString`.
- Files: `heygreeter/src/main.rs`
- Current mitigation: None.
- Recommendations: Use a specialized crate like `secrecy` to wrap sensitive data and ensure it is zeroed out of memory.

## Potential Failure Points

**[Installer Disk Operations]:**
- Problem: The installer immediately zaps the target disk without a confirmation of the specific disk details (beyond the path).
- Files: `airootfs/usr/local/bin/hey-install`
- Cause: Reliance on `sgdisk --zap-all` and `wipefs` with minimal safety checks.
- Improvement path: Add a final summary of the disk (Model, Serial, Size) and a mandatory "YES" confirmation.

**[Incremental Build Desync]:**
- Problem: `build.sh` uses MD5 hashes of `packages.target` to cache packages. If the upstream repository updates a package but the name/list hasn't changed, the ISO will use stale packages.
- Files: `build.sh`
- Cause: Hash only tracks the package *names*, not the versions or repo state.
- Improvement path: Incorporate the `pacman -Sy` database state into the cache stamp.

## Fragile Areas

**[Desktop Entry Parsing]:**
- Files: `heygreeter/src/main.rs`
- Why fragile: Uses `line.starts_with("Exec=")` which is a naive implementation of the Desktop Entry Specification. It might fail on files with multiple groups or complex exec lines.
- Safe modification: Use a dedicated `.desktop` file parsing library.
