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

# Enable core services
systemctl enable NetworkManager.service 2>/dev/null || true
systemctl enable vmtoolsd.service 2>/dev/null || true
systemctl enable seatd.service 2>/dev/null || true

# ---- Multi-DE Support ----
# Packages are installed, but we only enable greetd (heydm) as the primary entry point.
# Users can select GNOME/KDE/Hyprland from the hey-greeter session menu.
systemctl enable greetd.service 2>/dev/null || true

# Set greetd (heydm) as the default DM
systemctl set-default graphical.target

# ---- end-4 Hyprland Dotfiles Setup ----
echo "[heyOS] Preparing end-4 Hyprland dotfiles..."
DOTS_DIR="/tmp/dots-hyprland"
rm -rf "$DOTS_DIR"
git clone --depth 1 https://github.com/end-4/dots-hyprland "$DOTS_DIR"

# Copy dotfiles to /etc/skel so every new user (including 'hey') gets them
mkdir -p /etc/skel/.config
cp -r "$DOTS_DIR"/. /etc/skel/
cp -r "$DOTS_DIR"/.config/. /etc/skel/.config/

# Ensure 'hey' user gets them immediately for the live session
cp -r /etc/skel/. /home/hey/
chown -R hey:hey /home/hey/

# Clean up
rm -rf "$DOTS_DIR"

echo "[heyOS] First-boot setup complete."
