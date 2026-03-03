# Project: heyOS
**Producer:** heyOS Team

## Overview
heyOS is a custom Arch Linux distribution with a native Rust Wayland compositor (heydm based on Smithay) and a custom greeter (heygreeter based on Slint). 

## Aesthetic & UI/UX Goals
The desktop manager (`heydm`) must feature highly detailed and animated UI/UX elements, heavily inspired by "end-4 hyprland" dotfiles, providing a fluid and modern aesthetic out-of-the-box. Branding across the OS should be "heyOS" by the "heyOS Team".

## Technical Stack
- **Rust**: Core system components
- **Smithay**: Wayland compositor framework
- **Slint**: UI definition
- **Arch Linux**: Base OS distribution

## Milestone: Fixing heydm (Current)
- **Goal:** Resolve the "black screen" state and implement a fully-featured, high-fidelity desktop UI.
- **Key Focus:** Real-time background rendering, integrated status bar, polished window decorations, and a "magnetic" focus system inspired by the end-4 hyprland aesthetic.
