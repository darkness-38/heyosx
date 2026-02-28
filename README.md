# heyOS

heyOS is a custom, production-ready Arch Linux-based distribution featuring a proprietary Rust Wayland compositor (`heyDM`) and a user-friendly CLI installer (`hey-install`). It is designed to be fully automated and built from scratch using `archiso`.

## Project Architecture

heyOS is composed of three key components:

### 1. heyDM (Wayland Compositor)
Located in the `heydm/` directory, `heyDM` is a lightweight, custom Wayland compositor built from scratch using the [Smithay](https://github.com/Smithay/smithay) framework.
- **Language**: Rust
- **Features**:
  - Dynamic application launcher (toggled with Super+D)
  - Status panel (Clock, Battery, Network)
  - Window management (Tiling, Fullscreen, Floating)
  - Support for direct hardware rendering (udev/DRM) and nested rendering (Winit)

### 2. hey-install (CLI OS Installer)
Located at `airootfs/usr/local/bin/hey-install`, this is the interactive command-line installer for deploying heyOS to a target disk.
- **Features**:
  - Validates network connectivity and UEFI/BIOS boot modes.
  - Interactive target disk and filesystem (ext4/btrfs) selection.
  - Hardened automated partitioning (sgdisk/fdisk) and formatting.
  - Parallel downloads for fast `pacstrap` package installation.
  - Automated system configuration (fstab, time, locale, sudoers, initramfs).
  - Automated bootloader installation and GRUB setup.
  - Interactive user creation with safe password prompts.

### 3. ISO Build System
The root directory contains the configuration for `mkarchiso` to build the customized live/installable OS image.
- **`build.sh`**: The master build script. It handles:
  1. Installing necessary build dependencies on the host system.
  2. Compiling the `heyDM` Rust project.
  3. Deploying the compiled binaries into the `airootfs` overlay.
  4. Setting correct permissions.
  5. Invoking `mkarchiso` to generate the final bootable `.iso`.
- **`airootfs/`**: The overlay filesystem copied directly to the live OS root. It contains custom configurations, service files, and the installer (`hey-install`).
- **`packages.x86_64`**: The definitive list of Arch Linux packages to be installed onto the ISO during the build.
- **`profiledef.sh`**: The `archiso` profile definition file.


## Building heyOS

To build the heyOS ISO, you need an **Arch Linux host** (or WSL2 running Arch Linux) with internet access. The build process has been heavily optimized for speed (incremental caching, LZ4 compression, native filesystem copying in WSL).

### Requirements:
- Arch Linux (or WSL2 Arch Linux)
- `root` privileges (required by `mkarchiso`)

### Build Command:
Run the master build script as root:
```bash
sudo bash build.sh
```

To force a clean rebuild (wiping the work directory and cargo cache):
```bash
sudo bash build.sh --clean
```

The script will produce a bootable ISO file in the `out/` directory.

### Build Process Details:
1. If launched from a Windows environment (like WSL), the script intelligently copies the source to a native Linux filesystem (`/var/lib/heyos-build`) for significantly faster compilation and `mkarchiso` speeds.
2. It uses `cargo` to build the Rust components incrementally.
3. The final ISO will launch directly into the custom live `heyDM` desktop environment.

## Installation and Usage

Once the ISO is built, flash it to a USB drive or boot it in a Virtual Machine.
1. Boot into the live heyOS environment.
2. Ensure you have an active internet connection (e.g., `nmcli device wifi connect <SSID> password <PASSWORD>`).
3. Run the installer as root:
   ```bash
   sudo hey-install
   ```
4. Follow the interactive prompts to select your disk, set up partitions, install the base system, and create your credentials.
5. Reboot into your new custom heyOS installation!

## Customization

- **Packages**: Add or remove software by editing `packages.x86_64`.
- **Install Script**: Edit the installer behavior in `airootfs/usr/local/bin/hey-install`.
- **Compositor Theme**: Edit the rendering colors in `heydm/src/render.rs` (`colors` module).
- **System Overlay**: Add files to `airootfs/` to have them appear on the live system.

## License
[GPL-3.0]
