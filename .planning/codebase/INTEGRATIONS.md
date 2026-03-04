# External Integrations

**System Services:**
- **greetd:** The greeter integrates with `greetd` via a Unix socket (`GREETD_SOCK`) using the `greetd_ipc` protocol in `heygreeter/src/main.rs`.
- **Display Compositors:** `cage` is used to run the greeter as a single-app Wayland session (`airootfs/usr/local/bin/hey-greeter-launch`).
- **Desktop Environments:** Integration with Hyprland, GNOME, and KDE Plasma by parsing `.desktop` files in `/usr/share/wayland-sessions` and `/usr/share/xsessions`.

**Hardware & Virtualization:**
- **Mesa/Vulkan:** Graphics drivers for hardware acceleration (`packages.x86_64`).
- **Software Rendering:** Automatic fallback to `pixman` and `llvmpipe` for VM compatibility (VMware, VirtualBox) via environment variables in `hey-greeter-launch`.
- **Open-VM-Tools:** Built-in support for VMware guests.

**Environment Configuration:**
- **Wayland Environment:** Extensive use of `WLR_*`, `XDG_*`, and `QT_*` variables to ensure session compatibility across different compositors.
