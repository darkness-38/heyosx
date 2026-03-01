use greetd_ipc::codec::SyncCodec;
use greetd_ipc::{Request, Response, AuthMessageType};
use std::os::unix::net::UnixStream;
use slint::{SharedString, VecModel};
use std::rc::Rc;
use tracing::{info, error};

slint::include_modules!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let app = AppWindow::new()?;

    // Populate dummy users for now (in a real scenario we'd query /etc/passwd)
    let users: Vec<SharedString> = vec!["hamza".into(), "root".into()];
    let sessions: Vec<SharedString> = vec!["heydm".into(), "plasma".into(), "weston".into()];

    app.set_users(Rc::new(VecModel::from(users)).into());
    app.set_sessions(Rc::new(VecModel::from(sessions)).into());

    let app_weak = app.as_weak();
    app.on_login(move |user, password, session| {
        info!("Attempting login for user: {}", user);
        
        if let Ok(socket_path) = std::env::var("GREETD_SOCK") {
            match UnixStream::connect(socket_path) {
                Ok(mut stream) => {
                    let req = Request::CreateSession { username: user.to_string() };
                    if let Err(e) = req.write_to(&mut stream) {
                        error!("IPC send failed: {}", e);
                        return;
                    }
                    
                    match Response::read_from(&mut stream) {
                        Ok(Response::AuthMessage { auth_message_type, .. }) => {
                            if matches!(auth_message_type, AuthMessageType::Visible | AuthMessageType::Secret) {
                                let req = Request::PostAuthMessageResponse { response: Some(password.to_string()) };
                                if let Err(e) = req.write_to(&mut stream) {
                                    error!("IPC send failed: {}", e);
                                    return;
                                }
                                
                                match Response::read_from(&mut stream) {
                                    Ok(Response::Success) => {
                                        info!("Authentication successful! Starting session...");
                                        
                                        // Execute the session choice
                                        let cmd = vec![session.to_string()];
                                        let env = vec![];
                                        let req = Request::StartSession { cmd, env };
                                        
                                        if let Err(e) = req.write_to(&mut stream) {
                                            error!("Failed to send StartSession req: {}", e);
                                        } else {
                                            match Response::read_from(&mut stream) {
                                                Ok(Response::Success) => info!("Session started!"),
                                                Ok(Response::Error { description, .. }) => error!("Failed to start session: {}", description),
                                                _ => error!("Unexpected response to StartSession"),
                                            }
                                        }
                                    },
                                    Ok(Response::Error { description, .. }) => error!("Auth failed: {}", description),
                                    _ => error!("Unexpected response to password"),
                                }
                            }
                        },
                        Ok(Response::Error { description, .. }) => error!("CreateSession failed: {}", description),
                        _ => error!("Unexpected response to CreateSession"),
                    }
                },
                Err(e) => error!("Failed to connect to greetd: {}", e),
            }
        } else {
            error!("GREETD_SOCK not found. Cannot authenticate.");
        }
    });

    app.run()?;
    Ok(())
}
