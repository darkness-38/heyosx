# Structure

**Directory Layout:**
- `airootfs/`: Template for the live system's root filesystem.
  - `etc/`: System configuration (greetd, systemd, etc.).
  - `usr/bin/heydm`: Custom display manager/session script.
  - `usr/local/bin/transTR/`: Turkish localized system scripts (e.g., `guncelle`, `kur`, `arama`).
- `heygreeter/`: Rust source code for the custom login UI (Slint framework).
- `efiboot/`: UEFI bootloader configuration and entries.
- `syslinux/`: BIOS bootloader configuration.
- `build.sh`: Main script to trigger the ISO build process.
- `profiledef.sh`: Archiso profile definition (ISO name, label, etc.).
- `packages.x86_64`: List of packages to be included in the ISO.

**Key File Locations:**
- **Entry Point:** `build.sh`
- **Greeter Logic:** `heygreeter/src/main.rs`
- **Greeter UI:** `heygreeter/ui/greeter.slint`
- **System Customization:** `airootfs/root/customize_airootfs.sh`
