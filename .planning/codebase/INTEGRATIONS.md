# Integrations

## Session Lifecycle
1. **Boot**: System boots into Arch Linux environment.
2. **greetd**: The `greetd` service starts automatically.
3. **heygreeter**: `greetd` is configured to launch `heygreeter`.
4. **Authentication**: `heygreeter` interacts with PAM for user validation and `greetd-ipc` to start the user session.
5. **heydm**: Upon successful login, `heydm` is launched as the session compositor/window manager.

## System Management
- **transTR Scripts**: Integrated via `/etc/profile.d/transTR.sh`, which adds `/usr/local/bin/transTR` to the system `PATH`. These scripts act as wrappers for `pacman`, `systemctl`, and other core utilities.
- **Desktop Entries**: `heydm.desktop` defines the Wayland session for display managers.

## Build Process
- **build.sh**: Orchestrates the ISO creation.
- **profiledef.sh**: Configures `mkarchiso` settings (ISO name, label, bootloaders).
- **packages.x86_64**: Lists all packages to be included in the ISO, including core system utilities, Wayland stack, and desktop applications.
- **airootfs**: Contains the overlay filesystem that will be applied to the ISO.
