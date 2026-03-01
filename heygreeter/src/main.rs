use greetd_ipc::codec::SyncCodec;
use greetd_ipc::{Request, Response, AuthMessageType};
use std::os::unix::net::UnixStream;
use slint::{SharedString, VecModel};
use std::rc::Rc;
use tracing::{info, error};
use std::path::PathBuf;

slint::include_modules!();

/// Detect "real" users (UID >= 1000) from /etc/passwd
fn detect_users() -> Vec<String> {
    use std::io::{BufRead, BufReader};
    let mut users = Vec::new();
    
    if let Ok(file) = std::fs::File::open("/etc/passwd") {
        let reader = BufReader::new(file);
        for line in reader.lines().flatten() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 3 {
                if let Ok(uid) = parts[2].parse::<u32>() {
                    // Filter for normal users (Arch standard is 1000+)
                    // Also include 'hey' live user specifically if detected
                    if uid >= 1000 || parts[0] == "hey" {
                        users.push(parts[0].to_string());
                    }
                }
            }
        }
    }
    
    if users.is_empty() {
        users.push("hey".to_string());
    }
    users
}

/// Parse a .desktop file to find the Exec command
fn get_session_command(session_name: &str) -> Vec<String> {
    let session_dirs = ["/usr/share/wayland-sessions", "/usr/share/xsessions"];
    for dir in session_dirs {
        let path = PathBuf::from(dir).join(format!("{}.desktop", session_name));
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                if line.starts_with("Exec=") {
                    let exec = line.trim_start_matches("Exec=").trim();
                    if let Some(cmd) = shlex::split(exec) {
                        return cmd;
                    }
                }
            }
        }
    }
    // Fallback
    vec![session_name.to_string()]
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let app = AppWindow::new()?;

    let users = detect_users();
    let user_models: Vec<SharedString> = users.into_iter().map(SharedString::from).collect();
    
    let mut sessions: Vec<SharedString> = Vec::new();
    let session_dirs = ["/usr/share/wayland-sessions", "/usr/share/xsessions"];
    for dir in session_dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.path().file_stem().and_then(|s| s.to_str()) {
                    let name_str = name.to_string();
                    if !sessions.iter().any(|s| s.as_str() == name_str) {
                        sessions.push(name_str.into());
                    }
                }
            }
        }
    }

    if sessions.is_empty() {
        sessions.push("heydm".into());
    }

    app.set_users(Rc::new(VecModel::from(user_models)).into());
    app.set_sessions(Rc::new(VecModel::from(sessions)).into());

    // Update clock every second
    let clock_handle = app.as_weak();
    let timer = slint::Timer::default();
    timer.start(slint::TimerMode::Repeated, std::time::Duration::from_secs(1), move || {
        if let Some(app) = clock_handle.upgrade() {
            let now = chrono::Local::now();
            app.set_current_time(now.format("%H:%M").to_string().into());
            app.set_current_date(now.format("%A, %B %e").to_string().into());
        }
    });

    let app_handle = app.as_weak();
    app.on_login(move |user, password, session| {
        let Some(app) = app_handle.upgrade() else { return; };
        app.set_error_message("".into());
        info!("Attempting login for user: {}", user);
        
        let socket_path = match std::env::var("GREETD_SOCK") {
            Ok(path) => path,
            Err(_) => {
                error!("GREETD_SOCK not found");
                app.set_error_message("System error: greetd not found".into());
                return;
            }
        };

        match UnixStream::connect(socket_path) {
            Ok(mut stream) => {
                let req = Request::CreateSession { username: user.to_string() };
                if let Err(e) = req.write_to(&mut stream) {
                    app.set_error_message(format!("IPC Error: {}", e).into());
                    return;
                }
                
                match Response::read_from(&mut stream) {
                    Ok(Response::AuthMessage { auth_message_type, .. }) => {
                        if matches!(auth_message_type, AuthMessageType::Visible | AuthMessageType::Secret) {
                            let req = Request::PostAuthMessageResponse { response: Some(password.to_string()) };
                            if let Err(e) = req.write_to(&mut stream) {
                                app.set_error_message(format!("Auth communication failed: {}", e).into());
                                return;
                            }
                            
                            match Response::read_from(&mut stream) {
                                Ok(Response::Success) => {
                                    info!("Authentication successful! Starting session...");
                                    
                                    let cmd = get_session_command(session.as_str());
                                    info!("Executing session command: {:?}", cmd);
                                    let req = Request::StartSession { cmd, env: vec![] };
                                    
                                    if let Err(e) = req.write_to(&mut stream) {
                                        app.set_error_message(format!("Failed to start session: {}", e).into());
                                    } else {
                                        match Response::read_from(&mut stream) {
                                            Ok(Response::Success) => {
                                                info!("Session started! Exiting greeter...");
                                                std::process::exit(0);
                                            },
                                            Ok(Response::Error { description, .. }) => {
                                                app.set_error_message(description.into());
                                            },
                                            _ => app.set_error_message("Unexpected session response".into()),
                                        }
                                    }
                                },
                                Ok(Response::Error { description, .. }) => {
                                    app.set_error_message(description.into());
                                },
                                _ => app.set_error_message("Unexpected auth response".into()),
                            }
                        }
                    },
                    Ok(Response::Error { description, .. }) => app.set_error_message(description.into()),
                    _ => app.set_error_message("Unexpected greetd response".into()),
                }
            },
            Err(e) => app.set_error_message(format!("Failed to connect to login manager: {}", e).into()),
        }
    });

    app.run()?;
    Ok(())
}
