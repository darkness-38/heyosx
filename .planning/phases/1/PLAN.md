# Plan: Phase 1: Branding Consistency Update

## Goal
Unify the "heyOS" branding across bootloader, greeter, and desktop session.

## Tasks

### 1. Update `heygreeter/ui/greeter.slint`
- **Action:** Add "heyOS Team" to the UI branding.
- **Verification:** Read the file back and check for "heyOS Team".

### 2. Update `airootfs/etc/os-release`
- **Action:** Ensure `NAME`, `PRETTY_NAME`, and `ID` are set to "heyOS". Update `LOGO` if needed (e.g., set to `heyos-logo` or similar).
- **Verification:** Read the file back and check for "heyOS".

### 3. Update `airootfs/etc/issue`
- **Action:** Add a branded welcome message like "Welcome to heyOS \v (\l)".
- **Verification:** Read the file back and check for "heyOS".

### 4. Update `profiledef.sh`
- **Action:** Set `iso_publisher` to "heyOS Team".
- **Verification:** Read the file back and check for "heyOS Team".

### 5. Verify `airootfs/etc/hostname`
- **Action:** Ensure it is "heyOS".
- **Verification:** Read the file back and check for "heyOS".

### 6. Verification
- **Action:** Run a grep search across the `airootfs/etc` directory to confirm consistency.
- **Verification:** Check output for any "Arch Linux" or "Archlinux" strings.
