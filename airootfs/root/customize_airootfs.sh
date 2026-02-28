#!/usr/bin/env bash
# =============================================================================
# heyOS — Live environment setup script (runs inside the chroot during build)
# =============================================================================

# Don't use set -e — some commands may fail in chroot and that's OK
set +e

# Generate locales
locale-gen

# Set timezone
ln -sf /usr/share/zoneinfo/UTC /etc/localtime

# Create a default live user (skip if already exists)
if ! id hey &>/dev/null; then
    useradd -m -G wheel,video,audio,input,seat -s /bin/bash hey
fi

# Set passwords safely using standard chpasswd
echo 'hey:hey' | chpasswd
echo 'root:heyos' | chpasswd

# Enable services
systemctl enable NetworkManager.service 2>/dev/null || true
systemctl enable vmtoolsd.service 2>/dev/null || true
systemctl enable seatd.service 2>/dev/null || true
# Set multi-user target as default (CLI boot)
systemctl set-default multi-user.target

echo "[heyOS] First-boot setup complete."
