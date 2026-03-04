# Technology Stack: HeyOS wlroots Compositor

**Project:** HeyOS
**Researched:** 2024-05-22

## Recommended Stack

### Core Framework
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `wlroots` | 0.18+ | Compositor library | Modular, standard for non-KDE/GNOME Wayland. |
| `wayland-server` | 1.22+ | Protocol library | Core Wayland server communication. |
| `xkbcommon` | 1.5+ | Input handling | Industry standard for keyboard mapping. |

### Rendering
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `GLES2` | 2.0+ | GPU Rendering | Broad hardware compatibility, sufficient for 2D UI. |
| `wlr_scene` | N/A | Scene Graph | Automated damage tracking and buffer management. |

### Development & Build
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `C11` | Standard | Language | Native performance, memory control, standard for wlroots. |
| `Meson` | 1.0+ | Build System | Modern, fast, used by wlroots itself. |
| `Ninja` | 1.10+ | Build Tool | Required by Meson for high-speed compilation. |

### Supporting Libraries
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `pixman` | 0.40+ | CPU Regions | Region math and software fallback. |
| `libinput` | 1.20+ | Input devices | Hardware abstraction for keyboards, mice, trackpads. |

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| Framework | wlroots | Smithay (Rust) | Project specifies C11 for the core compositor. |
| Renderer | GLES2 | Vulkan | Vulkan is overkill for 2D shell; GLES2 has better legacy support. |
| Easing | Cubic Bezier | Linear | Linear motion feels robotic and non-premium. |

## Installation

### Core Dependencies (Arch Linux)
```bash
sudo pacman -S wlroots wayland wayland-protocols libinput libxkbcommon mesa pixman meson ninja
```

### Build Command Template
```bash
# Setup build directory
meson setup build

# Compile
ninja -C build

# Run (nested)
WAYLAND_DEBUG=1 ./build/heyos-compositor
```

## Sources
- [wlroots GitLab](https://gitlab.freedesktop.org/wlroots/wlroots)
- [TinyWL Example](https://gitlab.freedesktop.org/wlroots/wlroots/-/tree/master/tinywl)
- [wlr-protocols](https://gitlab.freedesktop.org/wlroots/wlr-protocols)
