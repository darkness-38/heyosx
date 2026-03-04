# Technology Stack

**Languages:**
- **Rust (2021 Edition):** Primary language for `heygreeter` (`heygreeter/Cargo.toml`).
- **Bash:** Powering the build system (`build.sh`) and system utilities (`airootfs/usr/local/bin/transTR/*`).
- **Slint:** UI declaration language for the greeter (`heygreeter/ui/greeter.slint`).

**Runtime & OS:**
- **Arch Linux:** Base rolling release distribution (`packages.x86_64`, `pacman.conf`).
- **Linux Kernel:** Standard kernel with broad firmware support (`linux`, `linux-firmware-*`).

**Frameworks & Libraries:**
- **Slint (v1.9):** UI framework for the Rust greeter.
- **Tokio:** Async runtime used in the greeter.
- **greetd_ipc (v0.9):** Communication protocol for the login manager.
- **PAM:** System authentication via `pam-auth` (v0.2).

**Build & Deployment:**
- **mkarchiso:** Official tool for generating the live ISO (`profiledef.sh`).
- **Custom Build Script:** `build.sh` handles incremental Rust builds, package caching, and WSL-to-Native synchronization.
- **Systemd-boot / Syslinux:** Bootloaders supported via `profiledef.sh` and `efiboot/`.
