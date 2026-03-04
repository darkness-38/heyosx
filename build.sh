#!/usr/bin/env bash
# =============================================================================
# heyOS — Master Build Script (ULTRA OPTIMIZED + INCREMENTAL)
# =============================================================================

set -euo pipefail

# ---- Configuration ----
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WINDOWS_SRC="${WINDOWS_SRC:-}"
BUILD_LOG="${WINDOWS_SRC:-$SCRIPT_DIR}/build_log.txt"
OUTPUT_DIR="${SCRIPT_DIR}/out"
AIROOTFS="${SCRIPT_DIR}/airootfs"
NATIVE_BUILD_DIR="/var/lib/heyos-build"

# Performance Tuning
TOTAL_JOBS=$(nproc 2>/dev/null || echo 4)
export CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

# ---- Parse flags ----
CLEAN=false
GREETER_ONLY=false
VERBOSE=false
for arg in "$@"; do
    case "$arg" in
        --clean) CLEAN=true ;;
        --greeter-only) GREETER_ONLY=true ;;
        -v|--verbose) VERBOSE=true ;;
    esac
done

# ---- Colors ----
GREEN='\033[0;32m'; YELLOW='\033[1;33m'; RED='\033[0;31m'; BLUE='\033[0;34m'; CYAN='\033[0;36m'; PINK='\033[1;35m'; BOLD='\033[1m'; NC='\033[0m'

log()      { echo -e "${BLUE}[BUILD]${NC} $*" | tee -a "$BUILD_LOG"; }
log_ok()   { echo -e "${GREEN}[OK]${NC}    $*" | tee -a "$BUILD_LOG"; }
log_warn() { echo -e "${YELLOW}[SKIP]${NC}  $*" | tee -a "$BUILD_LOG"; }
log_err()  { echo -e "${RED}[ERROR]${NC} $*" | tee -a "$BUILD_LOG"; }
log_step() { echo -e "\n${CYAN}${BOLD}══════ $* ══════${NC}\n" | tee -a "$BUILD_LOG"; }

BUILD_START=$SECONDS

# =============================================================================
# Initialization & Relocation (WSL Optimization)
# =============================================================================

if [[ "$SCRIPT_DIR" == /mnt/* ]]; then
    log "Detected Windows mount — syncing to native Linux filesystem..."
    mkdir -p "$NATIVE_BUILD_DIR"
    
    # RELOCATION OPTIMIZATION:
    # Use --exclude to ensure rsync ignores these directories entirely.
    # It will not copy them from the source, and importantly, it will NOT 
    # try to delete them in the destination if they already exist.
    rsync -a --delete \
        --exclude='/work/' \
        --exclude='/out/' \
        --exclude='/pkg-cache/' \
        --exclude='/heygreeter/target/' \
        --exclude=".git/" \
        "$SCRIPT_DIR/" "$NATIVE_BUILD_DIR/"
        
    cd "$NATIVE_BUILD_DIR"
    export WINDOWS_SRC="$SCRIPT_DIR"
    exec bash "$NATIVE_BUILD_DIR/build.sh" "$@"
fi

if $CLEAN; then
    log "Wiping all build caches..."
    rm -rf "${SCRIPT_DIR}/work" "${SCRIPT_DIR}/pkg-cache" "${AIROOTFS}/opt/heyos-packages"
    # Surgical cargo clean
    if [[ -d "${SCRIPT_DIR}/heygreeter" ]]; then
        cd "${SCRIPT_DIR}/heygreeter" && cargo clean && cd ..
    fi
fi

mkdir -p "$OUTPUT_DIR"
rm -f "${OUTPUT_DIR}"/*.iso

# =============================================================================
# Step 1: Environment Setup
# =============================================================================

log_step "Step 1: Environment Setup"

MIRROR_LIST="/etc/pacman.d/mirrorlist"
if [[ -f "$MIRROR_LIST" ]] && [[ $(find "$MIRROR_LIST" -mmin -720 2>/dev/null) ]]; then
    log_warn "Mirrorlist is fresh, skipping reflector."
else
    log "Optimizing mirrors..."
    reflector --latest 10 --protocol https --sort rate --save "$MIRROR_LIST" &>/dev/null || true
fi

# =============================================================================
# Step 2: Parallel Tasks (Incremental Rust Build + Package Caching)
# =============================================================================

log_step "Step 2: Parallel Execution"

# TASK A: Incremental Greeter Build
build_hey_greeter() {
    log "[RUST] Building hey-greeter..."
    cd "${SCRIPT_DIR}/heygreeter"
    
    # Use mold linker if available
    local rust_flags=""
    if command -v mold &>/dev/null; then rust_flags="-C link-arg=-fuse-ld=mold"; fi
    
    RUSTFLAGS="$rust_flags" cargo build --release -j "$TOTAL_JOBS" --quiet
    cp "target/release/hey-greeter" "${AIROOTFS}/usr/bin/hey-greeter"
    cd "${SCRIPT_DIR}"
}

# TASK B: Cache Offline Packages & Create Repo
cache_packages() {
    local pkg_cache_dir="${SCRIPT_DIR}/pkg-cache"
    local iso_pkg_dir="${AIROOTFS}/opt/heyos-packages"
    local target_pkg_file="${SCRIPT_DIR}/packages.target"
    
    mkdir -p "$pkg_cache_dir" "$iso_pkg_dir"

    # Read packages, ignoring comments and empty lines, and STRIP Windows carriage returns
    local install_pkgs=$(grep -v '^#' "$target_pkg_file" | tr -d '\r' | xargs)
    
    local current_hash=$(echo "$install_pkgs" | md5sum | cut -d' ' -f1)
    local stamp="${pkg_cache_dir}/.pkg_stamp"
    
    if [[ ! -f "$stamp" ]] || [[ "$current_hash" != "$(cat "$stamp")" ]]; then
        log "[CACHE] Downloading target packages and dependencies..."
        
        # ISO ISOLATION FIX:
        # We use a temporary DB path AND a minimal config to ensure we resolve 
        # dependencies for the TARGET, not based on the HOST system's state.
        # This prevents host-side conflicts (like libxml2 or systemd versions).
        local tmp_db="/tmp/heyos-build-db"
        local tmp_conf="/tmp/heyos-pacman.conf"
        rm -rf "$tmp_db" && mkdir -p "$tmp_db/local"
        
        cat << EOF > "$tmp_conf"
[options]
Architecture = auto
SigLevel = Optional TrustAll
LocalFileSigLevel = Optional
[core]
Include = /etc/pacman.d/mirrorlist
[extra]
Include = /etc/pacman.d/mirrorlist
EOF

        if ! pacman -Syw --cachedir "$pkg_cache_dir" --dbpath "$tmp_db" --config "$tmp_conf" --noconfirm $install_pkgs 2>&1 | tee -a "$BUILD_LOG"; then
            log_err "Failed to download packages. Check network or package names."
            return 1
        fi
        
        log "[REPO] Generating local repository index..."
        # Use a subshell with explicit error reporting
        (
            cd "$pkg_cache_dir"
            shopt -s nullglob
            # Fix: Only include the actual package archives, NOT the .sig files
            # repo-add handles signatures automatically if they exist next to the package
            local pkg_files=(*.pkg.tar.zst *.pkg.tar.xz *.pkg.tar.gz)
            
            if [[ ${#pkg_files[@]} -eq 0 ]]; then
                log_err "No package files found in $pkg_cache_dir"
                exit 1
            fi

            # Remove old DB to ensure a clean index
            rm -f "heyos_offline.db.tar.gz" "heyos_offline.db"
            
            # repo-add will pick up .sig files automatically if they match the package name
            if ! repo-add "heyos_offline.db.tar.gz" "${pkg_files[@]}" 2>&1 | tee -a "$BUILD_LOG"; then
                log_err "repo-add failed. Check the logs for corrupted packages."
                exit 1
            fi
        ) || return 1
        
        echo "$current_hash" > "$stamp"
    fi
    
    # Sync everything to ISO
    log "[SYNC] Integrating packages into ISO..."
    rsync -a --delete "$pkg_cache_dir/" "$iso_pkg_dir/" --exclude=".pkg_stamp"
    
    # Copy packages.target to ISO for installer reference
    mkdir -p "${AIROOTFS}/usr/share/heyos"
    cp "$target_pkg_file" "${AIROOTFS}/usr/share/heyos/packages.target"
}

# Run in parallel
build_hey_greeter &
PID_RUST=$!
cache_packages &
PID_CACHE=$!

wait $PID_RUST || { log_err "Rust build failed"; exit 1; }
wait $PID_CACHE || { log_err "Package caching failed"; exit 1; }

log_ok "Parallel tasks completed."

if $GREETER_ONLY; then
    log_info "Greeter updated. Skipping ISO build."
    exit 0
fi

# =============================================================================
# Step 3: ISO Generation
# =============================================================================

log_step "Step 3: ISO Generation"

log "Normalizing scripts..."
find "$AIROOTFS" -type f \( -name '*.sh' -o -name 'hey-*' \) -print0 | xargs -0 dos2unix -q 2>/dev/null || true

# Incremental mkarchiso
WORK_DIR="${SCRIPT_DIR}/work"
mkdir -p "$WORK_DIR"

PACKAGES_FILE="${SCRIPT_DIR}/packages.x86_64"
PACKAGES_STAMP="${WORK_DIR}/.packages_stamp"

if [[ -f "$PACKAGES_STAMP" ]] && diff -q "$PACKAGES_FILE" "$PACKAGES_STAMP" &>/dev/null; then
    log_warn "Package list unchanged, performing incremental build."
    touch "${WORK_DIR}/base._make_packages"
    touch "${WORK_DIR}/base._make_pacman_conf"
else
    cp "$PACKAGES_FILE" "$PACKAGES_STAMP"
    rm -f "$WORK_DIR/x86_64/airootfs.extracted" 2>/dev/null || true
fi

# Protect airootfs from full re-extraction if possible
find "$WORK_DIR" -maxdepth 1 ! -name "x86_64" ! -name "work" ! -name "." -exec rm -rf {} +

log "Running mkarchiso..."
mkarchiso -v -w "$WORK_DIR" -o "$OUTPUT_DIR" "$SCRIPT_DIR"

# Deliver ISO
ISO_FILE=$(find "$OUTPUT_DIR" -maxdepth 1 -iname "*.iso" -type f | head -1)
if [[ -f "${ISO_FILE:-}" ]] && [[ -n "${WINDOWS_SRC:-}" ]]; then
    log "Delivering ISO to Windows workspace..."
    mkdir -p "${WINDOWS_SRC}/out"
    rsync -ah --progress --remove-source-files "$ISO_FILE" "${WINDOWS_SRC}/out/"
fi

ELAPSED=$(( SECONDS - BUILD_START ))
log_step "Build Complete in $(( ELAPSED / 60 ))m $(( ELAPSED % 60 ))s"
