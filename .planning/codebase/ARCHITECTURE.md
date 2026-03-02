# Architecture

## Overview
HeyOS is a custom Arch Linux distribution focused on a localized (Turkish) user experience and a custom Rust-based desktop environment.

## Design Patterns
- **Localized Shell Layer**: Instead of standard aliases, HeyOS uses a set of physical scripts (`transTR`) to provide Turkish commands for system management.
- **Rust-Centric UI/DM**: Core desktop components are built with Rust for safety and performance, avoiding heavy dependencies like GNOME or KDE.
- **Smithay-based Compositor**: `heydm` follows the Smithay architectural pattern, where the compositor is a library that manages Wayland clients, inputs, and outputs.
- **Slint for UI**: Decouples UI design from business logic in the greeter.

## Subsystems
- **Display Management**: `greetd` + `heygreeter`.
- **Window Management**: `heydm`.
- **System Localization**: `transTR` suite.
- **ISO Distribution**: `mkarchiso` configuration.
