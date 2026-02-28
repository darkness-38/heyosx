// =============================================================================
// hey-greeter — Session Launcher
//
// After successful PAM authentication, this module:
//   1. Resolves the user's UID/GID from /etc/passwd
//   2. Sets up the environment for the Wayland session
//   3. Drops root privileges to the authenticated user
//   4. Executes heyDM as the user's desktop session
//
// The greeter runs as root (via systemd), so we MUST drop privileges
// before launching the compositor for security.
// =============================================================================

use std::env;
use std::ffi::CString;
use std::os::unix::process::CommandExt;
use std::path::Path;

use nix::unistd::{self, Gid, Uid};
use tracing::{error, info};

/// Path to the heyDM compositor binary
const HEYDM_PATH: &str = "/usr/bin/heydm";

/// Launch a Wayland session for the authenticated user.
///
/// This function:
///   1. Looks up the user's UID, GID, and home directory
///   2. Sets XDG and Wayland environment variables
///   3. Forks a child process
///   4. In the child: drops to user privileges and execs heyDM
///   5. In the parent: waits for the child (session) to exit
pub fn launch_session(username: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("Preparing Wayland session for user: {username}");

    // ---- Resolve user info from /etc/passwd ----
    let user_info = resolve_user(username)?;

    info!(
        "User resolved: uid={}, gid={}, home={}",
        user_info.uid, user_info.gid, user_info.home
    );

    // ---- Verify heyDM exists ----
    if !Path::new(HEYDM_PATH).exists() {
        return Err(format!("heyDM binary not found at {HEYDM_PATH}").into());
    }

    // ---- Fork the session ----
    match unsafe { nix::unistd::fork() } {
        Ok(nix::unistd::ForkResult::Child) => {
            // === CHILD PROCESS — becomes the user session ===

            // Set up environment variables
            setup_session_env(&user_info);

            // Create XDG_RUNTIME_DIR if it doesn't exist
            let xdg_runtime = format!("/run/user/{}", user_info.uid);
            if !Path::new(&xdg_runtime).exists() {
                let _ = std::fs::create_dir_all(&xdg_runtime);
                let _ = nix::unistd::chown(
                    Path::new(&xdg_runtime),
                    Some(Uid::from_raw(user_info.uid)),
                    Some(Gid::from_raw(user_info.gid)),
                );
                let _ = std::fs::set_permissions(
                    &xdg_runtime,
                    std::os::unix::fs::PermissionsExt::from_mode(0o700),
                );
            }

            // Start a new session
            let _ = unistd::setsid();

            // Drop privileges: set GID first, then UID
            if let Err(e) = unistd::setgid(Gid::from_raw(user_info.gid)) {
                error!("Failed to setgid({}): {e}", user_info.gid);
                std::process::exit(1);
            }

            // Initialize supplementary groups from /etc/group.
            // This preserves groups like wheel, video, audio, etc.
            let c_username = CString::new(username).unwrap_or_default();
            if let Err(e) = unistd::initgroups(&c_username, Gid::from_raw(user_info.gid)) {
                error!("Failed to initgroups: {e}");
                // Non-fatal: continue with just the primary group
            }

            if let Err(e) = unistd::setuid(Uid::from_raw(user_info.uid)) {
                error!("Failed to setuid({}): {e}", user_info.uid);
                std::process::exit(1);
            }

            // Change to user's home directory
            if let Err(e) = env::set_current_dir(&user_info.home) {
                error!("Failed to cd to {}: {e}", user_info.home);
                // Non-fatal: continue from /
            }

            info!("Privileges dropped. Launching heyDM as user '{username}'");

            // Exec heyDM — this replaces the current process
            let err = std::process::Command::new(HEYDM_PATH)
                .env("USER", username)
                .env("LOGNAME", username)
                .env("HOME", &user_info.home)
                .env("SHELL", &user_info.shell)
                .env("XDG_SESSION_TYPE", "wayland")
                .env("XDG_RUNTIME_DIR", &xdg_runtime)
                .exec();

            // If exec() returns, it failed
            error!("Failed to exec heyDM: {err}");
            std::process::exit(1);
        }
        Ok(nix::unistd::ForkResult::Parent { child }) => {
            // === PARENT PROCESS — waits for session to end ===
            info!("Session forked as PID {child}, waiting for exit...");

            // Wait for the child process (the entire desktop session)
            match nix::sys::wait::waitpid(child, None) {
                Ok(status) => {
                    info!("Session for '{username}' ended with status: {status:?}");
                }
                Err(e) => {
                    error!("Error waiting for session: {e}");
                }
            }

            Ok(())
        }
        Err(e) => {
            Err(format!("fork() failed: {e}").into())
        }
    }
}

/// User information resolved from /etc/passwd
struct UserInfo {
    uid: u32,
    gid: u32,
    home: String,
    shell: String,
}

/// Resolve a username to UID, GID, home, and shell by reading /etc/passwd
fn resolve_user(username: &str) -> Result<UserInfo, Box<dyn std::error::Error>> {
    let passwd_content = std::fs::read_to_string("/etc/passwd")?;

    for line in passwd_content.lines() {
        let fields: Vec<&str> = line.split(':').collect();
        if fields.len() >= 7 && fields[0] == username {
            let uid = fields[2].parse::<u32>()?;
            let gid = fields[3].parse::<u32>()?;
            let home = fields[5].to_string();
            let shell = fields[6].to_string();

            return Ok(UserInfo {
                uid,
                gid,
                home,
                shell,
            });
        }
    }

    Err(format!("User '{username}' not found in /etc/passwd").into())
}

/// Set up the environment variables for the Wayland session.
///
/// Note: env::set_var / env::remove_var are deprecated since Rust 1.80 due to
/// thread-safety concerns. This is safe here because we call this in a forked
/// child process which is single-threaded.
#[allow(deprecated)]
fn setup_session_env(user: &UserInfo) {
    let xdg_runtime = format!("/run/user/{}", user.uid);

    env::set_var("HOME", &user.home);
    env::set_var("SHELL", &user.shell);
    env::set_var("XDG_RUNTIME_DIR", &xdg_runtime);
    env::set_var("XDG_SESSION_TYPE", "wayland");
    env::set_var("XDG_CONFIG_HOME", format!("{}/.config", user.home));
    env::set_var("XDG_DATA_HOME", format!("{}/.local/share", user.home));
    env::set_var("XDG_CACHE_HOME", format!("{}/.cache", user.home));
    env::set_var("XDG_STATE_HOME", format!("{}/.local/state", user.home));

    // Clear potentially dangerous inherited variables
    env::remove_var("LD_PRELOAD");
    env::remove_var("LD_LIBRARY_PATH");

    // Set PATH to a safe default
    env::set_var(
        "PATH",
        "/usr/local/bin:/usr/bin:/bin:/usr/local/sbin:/usr/sbin:/sbin",
    );

    // Locale
    if env::var("LANG").is_err() {
        env::set_var("LANG", "en_US.UTF-8");
    }
}
