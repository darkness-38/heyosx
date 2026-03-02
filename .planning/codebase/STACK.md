# Tech Stack

## Languages
- **Rust**: Core system components (`heydm`, `heygreeter`).
- **Bash/Shell**: System integration, Arch ISO build scripts (`build.sh`, `transTR`).
- **Slint**: UI definition for the login greeter.
- **TOML**: Configuration for Cargo, `greetd`, and system setup.

## Core Components
- **heydm**: Custom Wayland compositor/window manager built using the **Smithay** framework.
- **heygreeter**: Custom login greeter using **Slint** for UI and `greetd-ipc` for session management.
- **transTR**: A suite of Turkish-localized shell wrappers for standard Linux/Arch commands.

## Frameworks & Libraries
- **Smithay**: Wayland compositor library for Rust.
- **Slint**: Declarative GUI framework for Rust.
- **greetd**: Minimal login daemon.
- **Tokio**: Async runtime for the greeter.
- **Wayland**: Core display protocol.
- **PipeWire**: Audio and video streaming.

## Infrastructure
- **Arch Linux**: Base operating system distribution.
- **mkarchiso**: Tool used to build the live ISO.
- **Systemd**: Init system and service manager.
- **NetworkManager**: Network configuration.
- **PipeWire**: Audio stack.
