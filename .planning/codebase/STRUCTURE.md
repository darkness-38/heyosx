# Project Structure

## Root Directories
- `airootfs/`: The root filesystem template for the live OS.
- `heydm/`: Source code for the custom Wayland window manager.
- `heygreeter/`: Source code for the login greeter.
- `efiboot/`, `syslinux/`: Bootloader configuration files.
- `photos/`: Static image assets (backgrounds, error screens).

## Key Files
- `build.sh`: Main build script for the project.
- `packages.x86_64`: Package list for the ISO.
- `profiledef.sh`: ISO metadata and permission settings.
- `pacman.conf`: Repository configuration for building the ISO.

## Important Subdirectories
- `airootfs/usr/local/bin/transTR/`: Turkish command wrappers.
- `heydm/src/`: Core compositor logic (launcher, panel, input, etc.).
- `heygreeter/ui/`: Slint UI definitions.
- `airootfs/etc/greetd/`: Greeter configuration.
