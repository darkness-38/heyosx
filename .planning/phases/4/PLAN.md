# Plan: Phase 4: Greeter Polish and Packaging

## Goal
Enhance the visual appeal of `heygreeter` with high-quality animations and ensure all components are correctly packaged into the final heyOS ISO.

## Tasks

### 1. Update `heygreeter/ui/greeter.slint` with Animations
- **Action:** Implement `states` and `animate` for buttons (scaling on press, color transitions on hover).
- **Action:** Add an animated focus effect (e.g., changing border color or adding an underline) to the password input field.
- **Action:** Add hover animations for the power icons (Sleep, Restart, Shutdown).
- **Verification:** Run `heygreeter` (if possible to test in isolation) and manually verify the animations feel "fluid" and "premium".

### 2. Verify Packaging Dependencies
- **Action:** Check `packages.x86_64` for any missing system-level dependencies required by the upgraded `heydm` (e.g., `pixman`, `fontconfig`, `mesa`).
- **Action:** Review `build.sh` to ensure it correctly builds and installs the new binaries into `airootfs/usr/bin/` or `airootfs/usr/local/bin/`.
- **Verification:** Confirm all necessary files are tracked in `git` or listed in the build configuration.

### 3. Final ISO Build and Verification
- **Action:** Run the full `build.sh` script to generate the heyOS ISO.
- **Action:** Perform a final check of the `airootfs` to ensure all branding and visual updates from previous phases are present.
- **Verification:** Confirm the ISO builds successfully and the resulting environment boots into the fully customized heyOS experience.
