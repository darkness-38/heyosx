# heyOS

heyOS is a custom, production-ready Arch Linux-based distribution featuring a proprietary Rust Wayland compositor (`heyDM`) and a custom graphical login manager (`hey-greeter`). It is designed to be fully automated and built from scratch using `archiso`.

## Project Architecture

heyOS is composed of several key components:

### 1. heyDM (Wayland Compositor)
Located in the `heydm/` directory, `heyDM` is a custom Wayland compositor built from scratch using the [Smithay](https://github.com/Smithay/smithay) framework. 
- **Language**: Rust
- **Features**: 
  - Dynamic application launcher (toggled with Super+D)
  - Status panel (Clock, Battery, Network)
  - Window management (Tiling, Fullscreen, Floating)
  - Support for direct hardware rendering (udev/DRM) and nested rendering (Winit)

### 2. hey-greeter (Login Manager)
Located in the `hey-greeter/` directory, this is a graphical login manager for heyOS.
- **Language**: Rust
- **UI Framework**: `egui` (via `eframe`)
- **Authentication**: PAM (Pluggable Authentication Modules)
- **Operation**: The greeter daemon runs on tty1 as root, launches the `cage` kiosk compositor with the `hey-greeter-ui` client, handles user authentication via PAM, drops root privileges, and then executes the `heyDM` wayland session for the authenticated user.

### 3. ISO Build System
The root directory contains the configuration for `mkarchiso` to build the customized live/installable OS image.
- **`build.sh`**: The master build script. It handles:
  1. Installing necessary build dependencies on the host system.
  2. Compiling both `heyDM` and `hey-greeter` Rust projects.
  3. Deploying the binaries into the `airootfs` overlay.
  4. Setting the correct file permissions.
  5. Invoking `mkarchiso` to generate the final bootable `.iso`.
- **`airootfs/`**: The overlay filesystem that will be copied directly to the live OS root. It contains custom configurations, service files, and scripts.
- **`packages.x86_64`**: The definitive list of Arch Linux packages to be installed onto the ISO.
- **`profiledef.sh`**: The `archiso` profile definition file.


## Building heyOS

To build the heyOS ISO, you need an **Arch Linux host** (or WSL2 running Arch Linux) with internet access.

### Requirements:
- Arch Linux
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

### Build Process details:
1. If launched from a Windows environment (like WSL), the script intelligently copies the source to a native Linux filesystem (`/var/lib/heyos-build`) for significantly faster compilation and `mkarchiso` speeds.
2. It uses `cargo` to build the Rust components incrementally.
3. The final ISO will contain a pre-configured `hey-greeter` service that launches on boot, allowing you to log into the custom `heyDM` desktop environment.

## Customization

- **Packages**: Add or remove software by editing `packages.x86_64`.
- **Compositor Theme**: Edit the rendering colors in `heydm/src/render.rs` (`colors` module).
- **Greeter UI**: Modify `hey-greeter/src/ui.rs`.
- **System Overlay**: Add files to `airootfs/` to have them appear on the live system.

## License
[GPL-3.0]
