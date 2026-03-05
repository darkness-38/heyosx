# Roadmap: heyDE Development (Milestone 2)

## Phase 6: System Integration
**Goal:** Fix build-to-live pipeline and permissions.
- [ ] **Task 6.1:** Update `build.sh` to sync `heydm` and `heyde` to `airootfs`.
- [ ] **Task 6.2:** Audit `customize_airootfs.sh` for all script permissions.
- [ ] **Task 6.3:** Verify `greetd` session launch logic and environment propagation.

## Phase 7: VM & Hardware Optimization
**Goal:** Ensure broad compatibility.
- [ ] **Task 7.1:** Refine software rendering flags in `hey-greeter-launch`.
- [ ] **Task 7.2:** Add fallback logic for `heyde` when GLES2 hardware is missing.
- [ ] **Task 7.3:** Final ISO build and verification.
