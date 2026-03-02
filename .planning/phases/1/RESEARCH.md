# Phase 1: Branding Consistency Update - Research Summary

## Identified Inconsistencies
- `heygreeter/ui/greeter.slint`: Currently mentions "heyOS Greeter", but needs "heyOS Team" for consistent branding.
- `airootfs/etc/os-release`: NAME="heyOS", PRETTY_NAME="heyOS", ID=heyos, but `LOGO=archlinux-logo` (still refers to Arch Linux).
- `airootfs/etc/issue`: Currently `\S  (\m)`, which is generic and does not explicitly mention "heyOS".
- `profiledef.sh`: Uses `iso_publisher="heyOS Project"`, but needs to be changed to "heyOS Team".
- `airootfs/etc/hostname`: Currently `heyOS`.

## Recommendations
- Update `greeter.slint` to explicitly include "heyOS" and "heyOS Team".
- Fix `os-release` to align with the intended branding (consistent NAME, PRETTY_NAME, and consider the LOGO field).
- Update `issue` to a branded welcome message like "Welcome to heyOS \v (\l)".
- Ensure `profiledef.sh` publisher string matches "heyOS Team".
