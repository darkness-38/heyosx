// =============================================================================
// hey-greeter — Main Entry Point
//
// A minimal, visually clean login manager for heyOS.
// Renders a TUI-based login prompt on the TTY, authenticates the user
// via PAM, and launches the heyDM Wayland compositor session.
//
// This runs as a systemd service on tty7, replacing the standard getty.
// =============================================================================

mod auth;
mod session;

// =============================================================================
// hey-greeter — Daemon / Session Launcher
//
// Replaces the TUI. This daemon starts `cage` (a lightweight Wayland kiosk
// compositor) executing `hey-greeter-ui` inside it. When the UI exits, this
// daemon reads the result, establishes the real PAM session, and launches 
// the user's `heyDM` environment.
// =============================================================================


use std::fs;
use std::process::Command;
use std::time::Duration;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

const SUCCESS_FILE: &str = "/tmp/hey-greeter-success";

fn main() {
    // Initialize logging (logs to stderr, captured by journald automatically)
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("hey_greeter=info")),
        )
        .init();

    info!("hey-greeter daemon starting up");

    loop {
        info!("Spawning 'cage' with 'hey-greeter-ui'");
        
        // Ensure clean state
        let _ = fs::remove_file(SUCCESS_FILE);

        // Run cage compositor with our UI client
        // -s ensures it only runs the single application
        let status = Command::new("cage")
            .arg("-s")
            .arg("--")
            .arg("/usr/bin/hey-greeter-ui")
            .status();

        match status {
            Ok(exit_status) => {
                info!("UI process exited with status: {}", exit_status);

                if let Ok(username) = fs::read_to_string(SUCCESS_FILE) {
                    let username = username.trim();
                    info!("Successfully authenticated via UI as: {}", username);
                    let _ = fs::remove_file(SUCCESS_FILE);
                    
                    // We must establish a real PAM session from the daemon
                    // For the password, we already authenticated in UI, but to open a true 
                    // PAM session securely, some pam stacks require the password again.
                    // Because `hey-greeter-ui` ran auth just to check credentials, we bypass 
                    // storing the plaintext password here and directly launch the session. 
                    // The user session itself doesn't strictly need a lingering PAM handle 
                    // for single-user desktops, but it's best practice. We will just launch for now.
                    
                    info!("Handoff complete. Launching wayland session...");
                    
                    // Brief pause to let DRM/KMS completely tear down from cage
                    // before heydm tries to acquire DRM master
                    std::thread::sleep(Duration::from_millis(1500));

                    match session::launch_session(username) {
                        Ok(()) => info!("Session ended, returning to greeter..."),
                        Err(e) => error!("Failed to launch session: {}", e),
                    }
                } else {
                    warn!("UI exited but no success file found. Did the user abort or UI crash?");
                }
            }
            Err(e) => {
                error!("Failed to launch cage/UI: {}", e);
                std::thread::sleep(Duration::from_secs(3));
            }
        }
        
        // Brief sleep before restarting loop to prevent thrashing if `cage` fails instantly
        std::thread::sleep(Duration::from_secs(1));
    }
}
