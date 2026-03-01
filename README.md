# 🌸 heyOS

heyOS is a custom, high-end Arch Linux-based distribution featuring a proprietary Rust-based graphics stack. It is designed for users who want a modern, "End-4" inspired aesthetic out of the box, with a focus on speed, automation, and a deep pink visual identity.

## ✨ Key Features

- **Custom Wayland Stack**: Built with Rust and Smithay, bypassing traditional desktop environments for a lightweight, performant experience.
- **"End-4" Aesthetic**: Island-style UI modules, floating panels, and a grid-based application launcher.
- **Pink Branding**: A consistent visual identity across terminal scripts, the installer, and the desktop UI.
- **Production-Ready Installer**: A robust CLI installer that handles everything from partitioning to deploying the custom desktop.
- **Diagnostic-Ready**: Built-in Memtest86+ integration in both live and installed boot menus.

---

## 🏗️ Project Architecture

### 1. heyDM (The Compositor)
Located in `heydm/`, this is the heart of the desktop. Built using the [Smithay](https://github.com/Smithay/smithay) framework.
- **Island UI**: Features a floating top bar with distinct modules for system status and workspaces.
- **Grid Launcher**: A modern, searchable application grid (Super+D) that dynamically detects installed `.desktop` files.
- **Robust Rendering**: Supports both hardware-accelerated OpenGL (Glow) and a Pixman-based **software rendering fallback** for VMs (like VMware) without 3D support.
- **Tiling Logic**: Efficient window management with thick, vibrant crimson/pink borders for focused windows.

### 2. hey-greeter (The Login Manager)
Located in `heygreeter/`, this provides a high-end login experience.
- **UI Engine**: Built with [Slint](https://slint.dev/) for smooth animations and modern "glassmorphism" effects.
- **Live Diagnostics**: Features a real-time digital clock and date display updated via a backend Rust timer.
- **User & Session Management**: Dynamically scans `/etc/passwd` for real users and `/usr/share/wayland-sessions` for available environments.
- **Secure Auth**: Uses `greetd` IPC and `shlex` for safe, robust session spawning.

### 3. hey-install (The OS Installer)
Located in `airootfs/usr/local/bin/hey-install`, this is the pink-themed CLI gateway to heyOS.
- **Safety First**: Explicitly validates hardware requirements (UEFI) and binary integrity before starting.
- **Flexible Partitioning**: Supports both standard `ext4` and modern `btrfs` (with subvolume support).
- **Offline-Optimized**: Can utilize a local package cache on the ISO to speed up installations in low-bandwidth environments.

---

## 🛠️ Build System

The root directory contains an optimized `build.sh` script designed for rapid development.

### Specialized Build Flags
For faster testing, you can build specialized ISOs that isolate specific components:
- `sudo bash build.sh --greeter-only`: Builds an ISO that launches directly into the login screen for UI testing.
- `sudo bash build.sh --heydm-only`: Builds an ISO that bypasses the greeter and launches the desktop directly.
- `sudo bash build.sh --clean`: Wipes all caches for a fresh production release.

### Optimization Details
- **Rsync Checksums**: The build system uses checksum-based synchronization to prevent false-positive Rust recompiles when building from Windows/WSL mounts.
- **Lockfile Preservation**: Intelligently preserves `Cargo.lock` in the native Linux build environment to maintain dependency stability.
- **LZ4 Compression**: The ISO uses high-speed LZ4 compression for the SquashFS image, ensuring fast boot times.

---

## 🚀 Getting Started

### Building the ISO
1. Boot into an **Arch Linux** host (or WSL2 with Arch).
2. Ensure `rsync` and `archiso` are installed.
3. Run the master script:
   ```bash
   sudo bash build.sh
   ```
4. The final image will be available in `out/heyOS-xxxx.iso`.

### Installation
1. Boot the ISO.
2. Connect to the internet (`nmcli` or `iwd`).
3. Launch the installer:
   ```bash
   sudo hey-install
   ```
4. Follow the pink prompts, reboot, and enjoy heyOS.

---

## 🎨 Customization

- **Colors**: Modify `heydm/src/render.rs` (compositor) or `heygreeter/ui/greeter.slint` (login screen).
- **Packages**: Add your favorite software to `packages.x86_64`.
- **System Tweaks**: Add files to `airootfs/` to have them persist in every build.

## ⚖️ License
GPL-3.0 - heyOS Project
