// =============================================================================
// heyDM — Main entry point
// The custom Wayland compositor for heyOS
//
// Initializes the backend (udev/DRM for real hardware, winit for testing),
// sets up the event loop, and runs the compositor.
// =============================================================================

mod input;
mod launcher;
mod panel;
mod render;
mod state;
mod window;

use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use crate::state::HeyDM;

fn main() {
    // Initialize structured logging with RUST_LOG support
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("heydm=info,smithay=warn")),
        )
        .with_target(true)
        .with_thread_ids(false)
        .init();

    info!("╔═══════════════════════════════════════╗");
    info!("║         heyDM Compositor v0.1         ║");
    info!("║       Wayland Desktop for heyOS       ║");
    info!("╚═══════════════════════════════════════╝");

    // Determine which backend to use:
    //   - If WAYLAND_DISPLAY or DISPLAY is set, use winit (nested compositor for dev)
//   - Otherwise, use udev/DRM (direct hardware — production path)
    // NOTE: For heyOS v0.1, heydm is designed to run nested under 'cage' 
    // for DRM/udev management on bare metal. The internal udev path in 
    // state.rs is currently a placeholder for future direct-to-hardware support.
    let use_winit = std::env::var("WAYLAND_DISPLAY").is_ok()
        || std::env::var("DISPLAY").is_ok();

    if use_winit {
        info!("Detected existing display server — starting in nested (winit) mode");
    } else {
        info!("No display server detected — starting in direct (udev/DRM) mode");
    }

    match HeyDM::run(use_winit) {
        Ok(()) => info!("heyDM shut down cleanly."),
        Err(e) => {
            error!("heyDM encountered a fatal error: {e}");
            std::process::exit(1);
        }
    }
}
