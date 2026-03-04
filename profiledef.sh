#!/usr/bin/env bash
# shellcheck disable=SC2034
# =============================================================================
# heyOS — archiso profile definition
# =============================================================================

iso_name="heyOS"
iso_label="HEYOS_$(date --date="@${SOURCE_DATE_EPOCH:-$(date +%s)}" +%Y%m)"
iso_publisher="heyOS Team"
iso_application="heyOS Live Environment"
iso_version="$(date --date="@${SOURCE_DATE_EPOCH:-$(date +%s)}" +%Y.%m.%d)"
install_dir="arch"
buildmodes=('iso')
bootmodes=('bios.syslinux'
           'uefi.systemd-boot')
arch="x86_64"
pacman_conf="pacman.conf"
airootfs_image_type="squashfs"
airootfs_image_tool_options=('-comp' 'lz4')
file_permissions=(
  ["/usr/bin/hey-greeter"]="0:0:755"
  ["/usr/local/bin/hey-install"]="0:0:755"
  ["/usr/local/bin/hey-greeter-launch"]="0:0:755"
  ["/usr/local/bin/transTR/*"]="0:0:755"
  ["/etc/shadow"]="0:0:400"
  ["/etc/gshadow"]="0:0:400"
  ["/etc/greetd/config.toml"]="0:0:644"
  ["/etc/sudoers.d/00-heyos"]="0:0:440"
)
