#!/usr/bin/env bash
# =============================================================================
# heyOS — Master Build Script
#
# This script runs on an Arch Linux host to:
#   1. Install build dependencies
#   2. Compile the heyDM compositor (Rust → release binary)
#   3. Compile the hey-greeter login manager (Rust → release binary)
#   4. Deploy binaries into the airootfs overlay
#   5. Set correct permissions
#   6. Invoke mkarchiso to produce the final heyOS ISO
#
# Usage:  sudo bash build.sh [--clean]
#   --clean    Force a full rebuild (wipe work dir and cargo cache)
#
# Requirements: Arch Linux host with internet access
# =============================================================================

set -euo pipefail

# ---- Configuration ----
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [[ -n "${WINDOWS_SRC:-}" ]]; then
    BUILD_LOG="${WINDOWS_SRC}/build_log.txt"
else
    BUILD_LOG="${SCRIPT_DIR}/build_log.txt"
fi
OUTPUT_DIR="${SCRIPT_DIR}/out"
AIROOTFS="${SCRIPT_DIR}/airootfs"
BUILD_TMP="/tmp/heyos-cargo-build"
JOBS=$(nproc 2>/dev/null || echo 4)

# ---- Parse flags ----
CLEAN=false
for arg in "$@"; do
    case "$arg" in
        --clean) CLEAN=true ;;
    esac
done

# ---- Colors ----
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

log()      { echo -e "${BLUE}[BUILD]${NC} $*" | tee -a "$BUILD_LOG"; }
log_ok()   { echo -e "${GREEN}[OK]${NC}    $*" | tee -a "$BUILD_LOG"; }
log_warn() { echo -e "${YELLOW}[SKIP]${NC}  $*" | tee -a "$BUILD_LOG"; }
log_err()  { echo -e "${RED}[ERROR]${NC} $*" | tee -a "$BUILD_LOG"; }
log_step() { echo -e "\n${CYAN}${BOLD}══════ $* ══════${NC}\n" | tee -a "$BUILD_LOG"; }

# Track total build time
BUILD_START=$SECONDS

# =============================================================================
# Initialize
# =============================================================================

if [[ -z "${WINDOWS_SRC:-}" ]]; then
    echo "" > "$BUILD_LOG"
    echo "heyOS Build Log — $(date)" >> "$BUILD_LOG"
    echo "========================================" >> "$BUILD_LOG"
else
    echo "--- Native build relaunched: $(date) ---" >> "$BUILD_LOG"
fi

echo -e "${CYAN}${BOLD}"
cat << 'EOF'

    ██╗  ██╗███████╗██╗   ██╗ ██████╗ ███████╗
    ██║  ██║██╔════╝╚██╗ ██╔╝██╔═══██╗██╔════╝
    ███████║█████╗   ╚████╔╝ ██║   ██║███████╗
    ██╔══██║██╔══╝    ╚██╔╝  ██║   ██║╚════██║
    ██║  ██║███████╗   ██║   ╚██████╔╝███████║
    ╚═╝  ╚═╝╚══════╝   ╚═╝    ╚═════╝ ╚══════╝

           ═══ ISO Build System ═══
EOF
echo -e "${NC}"

# Must be root for mkarchiso
if [[ $EUID -ne 0 ]]; then
    log_err "This script must be run as root."
    exit 1
fi

# =============================================================================
# Auto-relocate to native Linux filesystem for speed
# =============================================================================
NATIVE_BUILD_DIR="/var/lib/heyos-build"
# Track original Windows path so we can copy the ISO back at the end
WINDOWS_SRC="${WINDOWS_SRC:-}"

if [[ "$SCRIPT_DIR" == /mnt/* ]]; then
    # Ensure rsync is available for the copy
    pacman -S --needed --noconfirm rsync &>/dev/null || true

    log "Detected Windows mount ($SCRIPT_DIR) — copying to native Linux filesystem..."
    mkdir -p "$NATIVE_BUILD_DIR"
    # rsync the project (exclude work dir and output to save time)
    rsync -a --delete \
        --exclude='work/' \
        --exclude='out/' \
        --exclude='pkg-cache/' \
        --exclude='.git/' \
        --exclude='heydm/target/' \
        --exclude='hey-greeter/target/' \
        "$SCRIPT_DIR/" "$NATIVE_BUILD_DIR/"

    log_ok "Project synced to $NATIVE_BUILD_DIR"
    log "Re-launching build from native filesystem..."
    cd "$NATIVE_BUILD_DIR"
    export WINDOWS_SRC="$SCRIPT_DIR"
    exec bash "$NATIVE_BUILD_DIR/build.sh" "$@"
fi

if $CLEAN; then
    log "Clean build requested — wiping caches..."
    rm -rf "$BUILD_TMP" "${SCRIPT_DIR}/work"
fi

# =============================================================================
# Step 1: Install Build Dependencies (single pacman call)
# =============================================================================

log_step "Step 1: Installing build dependencies"

# Single pacman call: sync DBs + install everything (--needed skips what's current)
pacman -Sy --needed --noconfirm \
    archlinux-keyring \
    archiso \
    rustup \
    git \
    base-devel \
    wayland \
    wayland-protocols \
    libxkbcommon \
    libinput \
    seatd \
    mesa \
    pam \
    pkg-config \
    syslinux \
    dos2unix \
    lz4 \
    2>&1 | tee -a "$BUILD_LOG"

# Ensure Rust toolchain is properly configured
log "Ensuring Rust stable toolchain is configured..."
rustup default stable 2>&1 | tee -a "$BUILD_LOG"

RUSTC_VERSION=$(rustc --version 2>/dev/null || echo "unknown")
log_ok "Rust: $RUSTC_VERSION"
log_ok "Build dependencies installed"

# =============================================================================
# Step 2 & 3: Compile Rust Projects (with incremental build cache)
# =============================================================================

# Helper: build a Rust project, skip if binary is newer than all source files
# Usage: build_rust <name> <source_dir>
build_rust() {
    local name="$1"
    local src_dir="${SCRIPT_DIR}/${2}"
    local build_dir="${BUILD_TMP}/${2}"
    local bin_name="$2"
    local bin_path="${build_dir}/target/release/${bin_name}"

    log_step "Compiling ${name}"

    mkdir -p "$BUILD_TMP"

    # Check if source has changed since last build
    local needs_build=false
    if [[ ! -f "$bin_path" ]]; then
        needs_build=true
    else
        # Rebuild only if any src/ file or Cargo.toml is newer than the binary
        local newer
        newer=$(find "$src_dir/src" -newer "$bin_path" -print -quit 2>/dev/null || true)
        if [[ -n "$newer" ]]; then
            needs_build=true
        elif [[ "$src_dir/Cargo.toml" -nt "$bin_path" ]]; then
            needs_build=true
        fi
    fi

    if $needs_build; then
        # Sync sources while preserving the cargo target/ cache
        log "Syncing ${name} source to ${build_dir}..."
        mkdir -p "$build_dir"
        # Copy source files (exclude target/ to keep build cache)
        find "$src_dir" -maxdepth 1 -mindepth 1 ! -name target -exec cp -a {} "$build_dir/" \;

        cd "$build_dir"
        log "Running cargo build --release for ${name}..."
        cargo build --release 2>&1 | tee -a "$BUILD_LOG"

        if [[ ! -f "$bin_path" ]]; then
            log_err "${name} build failed — binary not found"
            exit 1
        fi

        local size
        size=$(du -h "$bin_path" | cut -f1)
        log_ok "${name} compiled successfully (${size})"
    else
        local size
        size=$(du -h "$bin_path" | cut -f1)
        log_warn "${name} unchanged — reusing cached binary (${size})"
    fi
}

build_rust "heyDM (Wayland compositor)" "heydm"
build_rust "hey-greeter (login manager)" "hey-greeter"

# =============================================================================
# Step 4: Deploy Binaries into airootfs
# =============================================================================

log_step "Step 4: Deploying binaries into airootfs"

mkdir -p "${AIROOTFS}/usr/bin"
mkdir -p "${AIROOTFS}/usr/local/bin"

cp "${BUILD_TMP}/heydm/target/release/heydm" "${AIROOTFS}/usr/bin/heydm"
cp "${BUILD_TMP}/hey-greeter/target/release/hey-greeter" "${AIROOTFS}/usr/bin/hey-greeter"
cp "${BUILD_TMP}/hey-greeter/target/release/hey-greeter-ui" "${AIROOTFS}/usr/bin/hey-greeter-ui"

log_ok "Binaries deployed to airootfs/usr/bin/"

# =============================================================================
# Step 5: Set Permissions
# =============================================================================

log_step "Step 5: Setting file permissions"

chmod 755 "${AIROOTFS}/usr/bin/heydm"
chmod 755 "${AIROOTFS}/usr/bin/hey-greeter"
chmod 755 "${AIROOTFS}/usr/bin/hey-greeter-ui"
chmod 755 "${AIROOTFS}/usr/local/bin/hey-install"
chmod 755 "${AIROOTFS}/root/customize_airootfs.sh"

# Ensure sudoers has correct permissions
chmod 440 "${AIROOTFS}/etc/sudoers.d/00-heyos" 2>/dev/null || true

log_ok "Permissions set"

# =============================================================================
# Step 6: Build ISO with mkarchiso
# =============================================================================

log_step "Step 6: Building ISO with mkarchiso"

cd "$SCRIPT_DIR"

# Use a disk-based work directory
WORK_DIR="${SCRIPT_DIR}/work"

# Reuse work dir for incremental builds (mkarchiso skips installed packages)
# Auto-wipe if the package list has changed since last build
PACKAGES_FILE="${SCRIPT_DIR}/packages.x86_64"
PACKAGES_STAMP="${WORK_DIR}/.packages_stamp"
if [[ -d "$WORK_DIR" ]]; then
    if [[ ! -f "$PACKAGES_STAMP" ]] || ! diff -q "$PACKAGES_FILE" "$PACKAGES_STAMP" &>/dev/null; then
        log "Package list changed — wiping work directory for fresh install..."
        rm -rf "$WORK_DIR"
    fi
fi
mkdir -p "$WORK_DIR"
cp "$PACKAGES_FILE" "$PACKAGES_STAMP"
mkdir -p "$OUTPUT_DIR"

# Remove old ISOs to avoid confusion
rm -f "${OUTPUT_DIR}"/heyOS-*.iso

# Fix Windows CRLF → Unix LF line endings using dos2unix (much faster than sed find)
log "Ensuring Unix line endings for profile files..."
find "$SCRIPT_DIR" -maxdepth 1 \( -name '*.sh' -o -name '*.cfg' -o -name 'packages.*' \) -exec dos2unix -q {} +
find "$SCRIPT_DIR/syslinux" "$SCRIPT_DIR/efiboot" -type f -exec dos2unix -q {} + 2>/dev/null || true
find "$SCRIPT_DIR/airootfs" -type f \( -name '*.conf' -o -name '*.sh' -o -name '*.service' \
    -o -name '*.gen' -o -name 'shadow' -o -name 'gshadow' -o -name 'hostname' \) \
    -exec dos2unix -q {} + 2>/dev/null || true

log "Running mkarchiso..."
mkarchiso -v -w "$WORK_DIR" -o "$OUTPUT_DIR" "$SCRIPT_DIR" \
    2>&1 | tee -a "$BUILD_LOG"

# Find the generated ISO
ISO_FILE=$(find "$OUTPUT_DIR" -name "heyOS-*.iso" -type f | head -1)

if [[ -z "$ISO_FILE" ]]; then
    log_err "ISO build failed — no .iso file found in $OUTPUT_DIR"
    exit 1
fi

ISO_SIZE=$(du -h "$ISO_FILE" | cut -f1)

# =============================================================================
# Done!
# =============================================================================

ELAPSED=$(( SECONDS - BUILD_START ))
MINS=$(( ELAPSED / 60 ))
SECS=$(( ELAPSED % 60 ))

log_step "Build Complete!"

echo -e "${GREEN}${BOLD}"
cat << EOF
    ╔═══════════════════════════════════════════════╗
    ║                                               ║
    ║    heyOS ISO built successfully!              ║
    ║                                               ║
    ║    Output: ${ISO_FILE}
    ║    Size:   ${ISO_SIZE}
    ║    Time:   ${MINS}m ${SECS}s
    ║                                               ║
    ╚═══════════════════════════════════════════════╝
EOF
echo -e "${NC}"

echo "" >> "$BUILD_LOG"
echo "Build completed: $(date) (${MINS}m ${SECS}s)" >> "$BUILD_LOG"
echo "ISO: ${ISO_FILE} (${ISO_SIZE})" >> "$BUILD_LOG"

# Copy ISO back to Windows workspace if we were launched from there
if [[ -n "$WINDOWS_SRC" ]]; then
    WIN_OUT="${WINDOWS_SRC}/out"
    mkdir -p "$WIN_OUT"
    log "Moving ISO to Windows workspace: $WIN_OUT"
    mv "$ISO_FILE" "$WIN_OUT/"
    log_ok "ISO moved to: ${WIN_OUT}/$(basename "$ISO_FILE")"
fi
