# Architecture

**Pattern Overview:**
HeyOS is an Arch Linux-based live distribution built using the `archiso` framework. It utilizes a custom greeter and localized system management scripts.

**Layers:**
- **Build Layer:** Managed by `build.sh` and `profiledef.sh`. It defines how the ISO is packaged and what filesystems are created.
- **RootFS Layer (`airootfs/`):** A template for the live system's root directory, containing pre-configured system files, custom binaries, and scripts.
- **Boot Layer:** Dual bootloader support via `efiboot/` (UEFI) and `syslinux/` (BIOS).

**Display & Greeter Flow:**
1. The system boots and `systemd` starts the display service.
2. `greetd` (configured in `airootfs/etc/greetd/config.toml`) is likely the primary login manager.
3. `airootfs/usr/local/bin/hey-greeter-launch` is invoked to start the custom Rust-based greeter (`heygreeter`).
4. `airootfs/usr/bin/heydm` serves as a wrapper or coordinator for the display session.
