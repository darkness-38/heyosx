// =============================================================================
// hey-greeter-ui — graphical login screen for heyOS
//
// An egui (eframe) based Wayland client. Runs fullscreen inside the minimal
// `cage` kiosk compositor. It collects the username and password, performs
// PAM authentication using the shared `auth` module, and drops a small 
// success file for the parent daemon if authentication passes.
// =============================================================================

mod auth;

use eframe::egui;
use std::fs;
use std::process;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

const SUCCESS_FILE: &str = "/tmp/hey-greeter-success";

fn main() -> Result<(), eframe::Error> {
    // Setup logging for the UI process
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("hey_greeter_ui=info")),
        )
        .init();

    info!("Starting graphical login UI");

    // Remove any stale success file
    let _ = fs::remove_file(SUCCESS_FILE);

    let options = eframe::NativeOptions {
        // Run full screen, no window decorations
        viewport: egui::ViewportBuilder::default()
            .with_fullscreen(true)
            .with_decorations(false),
        ..Default::default()
    };

    eframe::run_native(
        "heyOS Greeter",
        options,
        Box::new(|_cc| Box::<GreeterApp>::default()),
    )
}

struct GreeterApp {
    username: String,
    password: String,
    auth_error: Option<String>,
    pending_auth: bool,
}

impl Default for GreeterApp {
    fn default() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            auth_error: None,
            pending_auth: false,
        }
    }
}

impl eframe::App for GreeterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Define a beautiful dark theme
        let mut style = (*ctx.style()).clone();
        style.visuals = egui::Visuals::dark();
        // Customize colors (heyOS blue tint)
        style.visuals.window_fill = egui::Color32::from_rgb(20, 24, 32);
        style.visuals.panel_fill = egui::Color32::from_rgb(15, 18, 25);
        ctx.set_style(style);

        egui::CentralPanel::default().show(ctx, |ui| {
            // Center everything horizontally and vertically
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() / 3.0);

                // --- Banner / Logo ---
                ui.heading(
                    egui::RichText::new("heyOS")
                        .size(48.0)
                        .color(egui::Color32::from_rgb(100, 160, 255))
                        .strong(),
                );
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("Welcome back")
                        .size(18.0)
                        .color(egui::Color32::LIGHT_GRAY),
                );
                
                ui.add_space(32.0);

                // --- Login Form ---
                // We use a fixed-width container to group the inputs beautifully
                let form_width = 280.0;
                
                ui.allocate_ui(egui::vec2(form_width, 200.0), |ui| {
                    ui.spacing_mut().item_spacing.y = 12.0;

                    // Username Input
                    ui.label("Username");
                    let username_resp = ui.add(
                        egui::TextEdit::singleline(&mut self.username)
                            .desired_width(f32::INFINITY)
                            .margin(egui::vec2(8.0, 8.0)),
                    );

                    // Password Input
                    ui.label("Password");
                    let password_resp = ui.add(
                        egui::TextEdit::singleline(&mut self.password)
                            .password(true)
                            .desired_width(f32::INFINITY)
                            .margin(egui::vec2(8.0, 8.0)),
                    );

                    ui.add_space(8.0);

                    // Submit Button
                    ui.scope(|ui| {
                        // Make button slightly taller
                        ui.spacing_mut().button_padding = egui::vec2(0.0, 10.0);
                        let btn = egui::Button::new(
                            egui::RichText::new(if self.pending_auth { "Authenticating..." } else { "Log In" })
                                .size(16.0)
                                .color(egui::Color32::WHITE)
                        )
                        .fill(egui::Color32::from_rgb(65, 120, 220));

                        let submit_clicked = ui.add_sized([f32::INFINITY, 40.0], btn).clicked();

                        // Handle Enter key submission
                        let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));

                        if (submit_clicked || enter_pressed) && !self.pending_auth {
                            self.pending_auth = true;
                            self.auth_error = None;
                        }
                    });

                    // Error presentation
                    if let Some(err) = &self.auth_error {
                        ui.add_space(12.0);
                        ui.label(
                            egui::RichText::new(err)
                                .color(egui::Color32::from_rgb(255, 100, 100))
                                .size(14.0)
                        );
                    }

                    // Perform authentication if requested
                    if self.pending_auth {
                        self.pending_auth = false; // Reset state for next frame
                        match auth::authenticate(&self.username, &self.password) {
                            Ok(pam_session) => {
                                info!("UI auth successful for {}", self.username);
                                // PAM session drops immediately here. This is fine because
                                // the real session will be opened by `hey-greeter` parent daemon.
                                // We are just verifying credentials.
                                drop(pam_session);
                                
                                // Write success state for daemon reading
                                if let Err(e) = fs::write(SUCCESS_FILE, &self.username) {
                                    error!("Failed to write success file: {e}");
                                }

                                // Exit cleanly, signaling to `cage` to tear down
                                process::exit(0);
                            }
                            Err(e) => {
                                error!("UI auth failed: {e}");
                                self.auth_error = Some("Invalid username or password".to_string());
                                // Zero out password
                                auth::zeroize_string(&mut self.password);
                                // Refocus password field on fail
                                password_resp.request_focus();
                            }
                        }
                    } else if username_resp.gained_focus() {
                        // Minor UX affordance
                    } else if self.username.is_empty() {
                        username_resp.request_focus();
                    }
                });
            });
        });
    }
}
