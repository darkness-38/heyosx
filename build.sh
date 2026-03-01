#!/usr/bin/env bash
# =============================================================================
# heyOS — Master Build Script
#
# Optimized for speed, incremental builds, and WSL/Windows relocation.
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
BUILD_TMP="/var/lib/heyos-cargo-build"
# Calculate jobs for parallel compilation
TOTAL_JOBS=$(nproc 2>/dev/null || echo 4)
PARALLEL_JOBS=$(( TOTAL_JOBS / 2 ))
[[ $PARALLEL_JOBS -lt 1 ]] && PARALLEL_JOBS=1

# ---- Parse flags ----
CLEAN=false
GREETER_ONLY=false
HEYDM_ONLY=false
for arg in "$@"; do
    case "$arg" in
        --clean) CLEAN=true ;;
        --greeter-only) GREETER_ONLY=true ;;
        --heydm-only) HEYDM_ONLY=true ;;
    esac
done

if $GREETER_ONLY && $HEYDM_ONLY; then
    echo "Error: Cannot use both --greeter-only and --heydm-only"
    exit 1
fi

# ---- Colors ----
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
PINK='\033[1;35m'
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

echo -e "${PINK}${BOLD}"
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
WINDOWS_SRC="${WINDOWS_SRC:-}"

if [[ "$SCRIPT_DIR" == /mnt/* ]]; then
    pacman -S --needed --noconfirm rsync &>/dev/null || true

    log "Detected Windows mount ($SCRIPT_DIR) — syncing to native Linux filesystem..."
    mkdir -p "$NATIVE_BUILD_DIR"
    
    # RELOCATION OPTIMIZATION: Archive mode includes timestamps (-t). 
    # This is much faster than checksums for initial sync.
    rsync -a --delete \
        --exclude='work/' \
        --exclude='out/' \
        --exclude='pkg-cache/' \
        --exclude='.git/' \
        --exclude='heydm/target/' \
        --exclude='heygreeter/target/' \
        "$SCRIPT_DIR/" "$NATIVE_BUILD_DIR/"

    log_ok "Project synced to $NATIVE_BUILD_DIR"
    log "Re-launching build from native filesystem..."
    cd "$NATIVE_BUILD_DIR"
    export WINDOWS_SRC="$SCRIPT_DIR"
    exec bash "$NATIVE_BUILD_DIR/build.sh" "$@"
fi

if $CLEAN; then
    log "Clean build requested — wiping caches..."
    rm -rf "$BUILD_TMP" "${SCRIPT_DIR}/work" "${SCRIPT_DIR}/pkg-cache" "${AIROOTFS}/opt/heyos-packages"
fi

# PRE-BUILD CLEANUP: Delete any existing ISOs in the local Linux output directory
# This ensures we don't accidentally move an old version.
log "Cleaning local output directory..."
rm -f "${OUTPUT_DIR}"/*.iso

# =============================================================================
# Step 1: Install Build Dependencies
# =============================================================================

log_step "Step 1: Verifying build dependencies"

# Only run pacman if dependencies are missing to save time
MISSING_PKGS=""
for pkg in archlinux-keyring archiso rustup git base-devel wayland wayland-protocols \
           libxkbcommon libinput seatd mesa pam pkg-config syslinux dos2unix lz4 \
           fontconfig pixman libdrm noto-fonts; do
    if ! pacman -Qi "$pkg" &>/dev/null; then
        MISSING_PKGS+="$pkg "
    fi
done

if [[ -n "$MISSING_PKGS" ]]; then
    log "Installing missing dependencies: $MISSING_PKGS"
    pacman -Sy --needed --noconfirm $MISSING_PKGS 2>&1 | tee -a "$BUILD_LOG"
else
    log_ok "All build dependencies are already installed."
fi

rustup default stable 2>&1 | tee -a "$BUILD_LOG"
log_ok "Build environment ready"

# =============================================================================
# Step 2 & 3: Compile Rust Projects (PARALLELIZED)
# =============================================================================

# Helper: build a Rust project
build_rust() {
    local name="$1"
    local dir_name="$2"
    local bin_name="${3:-$2}"
    local parallel="${4:-false}"
    local src_dir="${SCRIPT_DIR}/${dir_name}"
    local build_dir="${BUILD_TMP}/${dir_name}"
    local bin_path="${build_dir}/target/release/${bin_name}"

    log "Checking ${name}..."
    mkdir -p "$build_dir"

    # Sync source using timestamps (-a) instead of checksums for speed
    rsync -a --delete --exclude='target/' "$src_dir/" "$build_dir/"

    cd "$build_dir"
    # Assign unique TMPDIR to prevent parallel build race conditions
    export TMPDIR="${build_dir}/tmp"
    mkdir -p "$TMPDIR"
    
    # Run cargo
    # If parallel, limit jobs to avoid system freezing
    local jobs_flag=""
    if $parallel; then
        jobs_flag="-j ${PARALLEL_JOBS}"
    fi

    # Use a subshell to capture output and prefix it for clarity if parallel
    if ! cargo build --release ${jobs_flag} 2>&1 | sed "s/^/[${name}] /" | tee -a "$BUILD_LOG"; then
        log_err "${name} build failed."
        return 1
    fi

    if [[ ! -f "$bin_path" ]]; then
        log_err "${name} binary not found at $bin_path."
        return 1
    fi

    log_ok "${name} ready."
    return 0
}

log_step "Step 2 & 3: Compiling Rust components (Parallel: ${PARALLEL_JOBS} jobs/task)"

# Launch builds in background if both are needed
if ! $GREETER_ONLY && ! $HEYDM_ONLY; then
    build_rust "heyDM" "heydm" "heydm" true &
    PID_HEYDM=$!
    build_rust "hey-greeter" "heygreeter" "hey-greeter" true &
    PID_GREETER=$!
    
    wait "$PID_HEYDM" || { log_err "heyDM build failed"; exit 1; }
    wait "$PID_GREETER" || { log_err "hey-greeter build failed"; exit 1; }
else
    # Sequential build if only one requested (use all cores)
    if ! $GREETER_ONLY; then
        build_rust "heyDM" "heydm" "heydm" false
    fi
    if ! $HEYDM_ONLY; then
        build_rust "hey-greeter" "heygreeter" "hey-greeter" false
    fi
fi

log_ok "All Rust components compiled."

# =============================================================================
# Step 4: Deploy Binaries
# =============================================================================

log_step "Step 4: Deploying binaries and configuring boot"

mkdir -p "${AIROOTFS}/usr/bin"
mkdir -p "${AIROOTFS}/usr/local/bin"
mkdir -p "${AIROOTFS}/etc/greetd"

# Helper for greetd config
write_greetd_config() {
    local cmd="$1"
    cat << EOF > "${AIROOTFS}/etc/greetd/config.toml"
[terminal]
vt = 1
[default_session]
command = "env WLR_RENDERER=pixman WLR_NO_HARDWARE_CURSORS=1 $cmd"
user = "hey"
EOF
}

if $GREETER_ONLY; then
    cp "${BUILD_TMP}/heygreeter/target/release/hey-greeter" "${AIROOTFS}/usr/bin/hey-greeter"
    write_greetd_config "cage -s -- /usr/bin/hey-greeter"
elif $HEYDM_ONLY; then
    cp "${BUILD_TMP}/heydm/target/release/heydm" "${AIROOTFS}/usr/bin/heydm"
    write_greetd_config "cage -s -- /usr/bin/heydm"
else
    cp "${BUILD_TMP}/heydm/target/release/heydm" "${AIROOTFS}/usr/bin/heydm"
    cp "${BUILD_TMP}/heygreeter/target/release/hey-greeter" "${AIROOTFS}/usr/bin/hey-greeter"
    write_greetd_config "cage -s -- /usr/bin/hey-greeter"
fi

log_ok "Binaries deployed"

# =============================================================================
# Step 5: Set Permissions & Normalization
# =============================================================================

log_step "Step 5: Permissions & File Normalization"

chmod 755 "${AIROOTFS}/usr/bin/heydm" 2>/dev/null || true
chmod 755 "${AIROOTFS}/usr/bin/hey-greeter" 2>/dev/null || true
chmod 755 "${AIROOTFS}/usr/local/bin/hey-install"
chmod 755 "${AIROOTFS}/root/customize_airootfs.sh"
chmod 440 "${AIROOTFS}/etc/sudoers.d/00-heyos" 2>/dev/null || true

# NORMALIZATION OPTIMIZATION: Only normalize relevant scripts and configs
log "Normalizing line endings..."
find "$SCRIPT_DIR" -maxdepth 1 \( -name '*.sh' -o -name 'packages.*' \) -exec dos2unix -q {} +
find "$AIROOTFS" -type f \( -name '*.conf' -o -name '*.sh' -o -name 'hey-install' \) -exec dos2unix -q {} +

log_ok "Permissions and normalization complete"

# =============================================================================
# Step 5.5: Offline Caching (MD5 STAMPED)
# =============================================================================

log_step "Step 5.5: Caching offline installer packages"

PKG_CACHE_DIR="${SCRIPT_DIR}/pkg-cache"
ISO_PKG_DIR="${AIROOTFS}/opt/heyos-packages"
PKG_STAMP="${PKG_CACHE_DIR}/.pkg_list_stamp"

mkdir -p "${PKG_CACHE_DIR}"
mkdir -p "${ISO_PKG_DIR}"

# Extract package list from installer
INSTALL_PKGS=$(awk '/local PACKAGES=\(/{flag=1; next} /\)/{flag=0} flag' "${AIROOTFS}/usr/local/bin/hey-install" | tr -d '\r\\' | tr '\n' ' ')
INSTALL_PKGS="${INSTALL_PKGS} btrfs-progs"

# CACHING OPTIMIZATION: Only download if package list changed or cache empty
CURRENT_PKG_HASH=$(echo "$INSTALL_PKGS" | md5sum | cut -d' ' -f1)
if [[ ! -f "$PKG_STAMP" ]] || [[ "$CURRENT_PKG_HASH" != "$(cat "$PKG_STAMP")" ]] || $CLEAN; then
    log "Package list changed or cache empty — downloading..."
    EMPTY_DB="${BUILD_TMP}/empty_pacman_db"
    mkdir -p "${EMPTY_DB}/local"
    pacman -Syw --cachedir "${PKG_CACHE_DIR}" --dbpath "${EMPTY_DB}" --noconfirm ${INSTALL_PKGS} &>/dev/null || true
    echo "$CURRENT_PKG_HASH" > "$PKG_STAMP"
    log_ok "Packages downloaded to host cache."
else
    log_warn "Package list unchanged — using host cache."
fi

# Always sync from host cache to ISO dir (fast)
cp "${PKG_CACHE_DIR}/"*.pkg.tar.* "${ISO_PKG_DIR}/" 2>/dev/null || true
log_ok "Offline packages ready on ISO."

# =============================================================================
# Step 6: Build ISO
# =============================================================================

log_step "Step 6: Building ISO with mkarchiso"

WORK_DIR="${SCRIPT_DIR}/work"
mkdir -p "$WORK_DIR"
mkdir -p "$OUTPUT_DIR"

# Reuse work dir for incremental builds (mkarchiso skips installed packages)
# Auto-wipe if the package list has changed since last build
PACKAGES_FILE="${SCRIPT_DIR}/packages.x86_64"
PACKAGES_STAMP="${WORK_DIR}/.packages_stamp"
NEED_FULL_REINSTALL=false

if [[ -d "$WORK_DIR" ]]; then
    if [[ ! -f "$PACKAGES_STAMP" ]] || ! diff -q "$PACKAGES_FILE" "$PACKAGES_STAMP" &>/dev/null; then
        log "Package list changed — full re-installation required."
        NEED_FULL_REINSTALL=true
        cp "$PACKAGES_FILE" "$PACKAGES_STAMP"
    fi
fi

# SURGICAL CLEANUP:
# We delete everything except x86_64/ (where packages live).
# If the package list DID NOT change, we also keep the 'base._make_packages' marker.
log "Cleaning work directory markers..."
find "$WORK_DIR" -maxdepth 1 ! -name "x86_64" ! -name "work" ! -name "." -exec rm -rf {} +

if $NEED_FULL_REINSTALL; then
    rm -f "$WORK_DIR/x86_64/airootfs.extracted" 2>/dev/null || true
else
    # Keep the markers that tell mkarchiso packages are already installed
    touch "${WORK_DIR}/base._make_packages"
    touch "${WORK_DIR}/base._make_pacman_conf"
fi

log "Triggering fresh ISO generation..."
rm -f "${OUTPUT_DIR}"/*.iso

mkarchiso -v -w "$WORK_DIR" -o "$OUTPUT_DIR" "$SCRIPT_DIR" 2>&1 | tee -a "$BUILD_LOG"

# Find the generated ISO (case-insensitive search)
# We look for anything .iso in the output dir
ISO_FILE=$(find "$OUTPUT_DIR" -maxdepth 1 -iname "*.iso" -type f | head -1)

# FINAL RELOCATION: Move ISO to Windows workspace if applicable
if [[ -f "${ISO_FILE:-}" ]]; then
    # If we have a tracked WINDOWS_SRC, use it. 
    # Otherwise, check if our current SCRIPT_DIR is already a mount.
    FINAL_DEST=""
    if [[ -n "${WINDOWS_SRC:-}" ]]; then
        FINAL_DEST="${WINDOWS_SRC}/out"
    elif [[ "$SCRIPT_DIR" == /mnt/* ]]; then
        FINAL_DEST="${SCRIPT_DIR}/out"
    fi

    if [[ -n "$FINAL_DEST" ]]; then
        mkdir -p "$FINAL_DEST"
        echo -e "${BLUE}[MOVE]${NC} Moving ISO to Windows workspace: ${FINAL_DEST}" | tee -a "$BUILD_LOG"
        
        # Use rsync for cross-filesystem moves
        if rsync -ah --progress --remove-source-files "$ISO_FILE" "$FINAL_DEST/"; then
            NEW_ISO_PATH="${FINAL_DEST}/$(basename "$ISO_FILE")"
            log_ok "ISO delivered to Windows: $NEW_ISO_PATH"
            ISO_FILE="$NEW_ISO_PATH"
        else
            log_err "Failed to deliver ISO to Windows workspace. It remains at: $ISO_FILE"
        fi
    else
        log_info "ISO is located at: $ISO_FILE"
    fi
else
    log_err "No ISO file was generated. Check the logs above for errors."
fi

ELAPSED=$(( SECONDS - BUILD_START ))
log_step "Build Complete in $(( ELAPSED / 60 ))m $(( ELAPSED % 60 ))s"
