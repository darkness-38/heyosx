// =============================================================================
// heyDM — Status Panel
//
// The top bar of the heyOS desktop, displaying:
//   - Left:   "heyOS" branding / launcher trigger button
//   - Center: Window title of focused application
//   - Right:  Network status, battery level, clock
//
// Uses fontdue for software text rasterization into pixel buffers that are
// uploaded as GPU textures for drawing.
// =============================================================================

use chrono::Local;
use std::fs;
use std::path::Path;
use tracing::debug;

/// Height of the status panel in pixels
#[allow(dead_code)]
pub const PANEL_HEIGHT: i32 = 32;

/// Status panel state and data
pub struct StatusPanel {
    /// Cached clock string (updated once per second)
    clock_text: String,
    /// Last update timestamp (seconds)
    last_update_sec: i64,
    /// Battery percentage (0-100, or -1 if no battery)
    battery_percent: i32,
    /// Whether the battery is charging
    battery_charging: bool,
    /// Network connection status
    network_status: NetworkStatus,
    /// Network SSID or interface name
    network_name: String,
}

/// Network connection state
#[derive(Debug, Clone, PartialEq)]
pub enum NetworkStatus {
    /// Not connected to any network
    Disconnected,
    /// Connected via WiFi
    WiFi,
    /// Connected via Ethernet
    Ethernet,
    /// Connection status unknown
    Unknown,
}

#[allow(dead_code)]
impl StatusPanel {
    /// Create a new status panel with initial state
    pub fn new() -> Self {
        let mut panel = Self {
            clock_text: String::new(),
            last_update_sec: 0,
            battery_percent: -1,
            battery_charging: false,
            network_status: NetworkStatus::Unknown,
            network_name: String::new(),
        };
        panel.update();
        panel
    }

    /// Update all panel data (called each frame, but internally rate-limited)
    pub fn update(&mut self) {
        let now = Local::now();
        let current_sec = now.timestamp();

        // Only update once per second (no need to read sysfs 60 times/sec)
        if current_sec == self.last_update_sec {
            return;
        }
        self.last_update_sec = current_sec;

        // ---- Update clock ----
        self.clock_text = now.format("%a %b %d  %H:%M").to_string();

        // ---- Update battery ----
        self.update_battery();

        // ---- Update network ----
        self.update_network();
    }

    /// Read battery status from /sys/class/power_supply/
    fn update_battery(&mut self) {
        let bat_path = Path::new("/sys/class/power_supply/BAT0");

        if !bat_path.exists() {
            // Try BAT1 as fallback
            let bat1_path = Path::new("/sys/class/power_supply/BAT1");
            if !bat1_path.exists() {
                self.battery_percent = -1; // No battery found (desktop/VM)
                return;
            }
            self.read_battery_sysfs(bat1_path);
            return;
        }

        self.read_battery_sysfs(bat_path);
    }

    /// Parse battery info from a sysfs power_supply path
    fn read_battery_sysfs(&mut self, path: &Path) {
        // Read capacity (percentage)
        if let Ok(capacity_str) = fs::read_to_string(path.join("capacity")) {
            if let Ok(capacity) = capacity_str.trim().parse::<i32>() {
                self.battery_percent = capacity.clamp(0, 100);
            }
        }

        // Read charging status
        if let Ok(status_str) = fs::read_to_string(path.join("status")) {
            let status = status_str.trim().to_lowercase();
            self.battery_charging = status == "charging" || status == "full";
        }
    }

    /// Read network status from NetworkManager via /sys or simple checks
    fn update_network(&mut self) {
        // Check for wired connection first
        let net_path = Path::new("/sys/class/net");

        if let Ok(entries) = fs::read_dir(net_path) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();

                // Skip loopback
                if name == "lo" {
                    continue;
                }

                // Check if interface is up by reading operstate
                let operstate_path = entry.path().join("operstate");
                if let Ok(state) = fs::read_to_string(&operstate_path) {
                    if state.trim() == "up" {
                        if name.starts_with("wl") || name.starts_with("wlan") {
                            self.network_status = NetworkStatus::WiFi;
                            self.network_name = name;
                            return;
                        } else if name.starts_with("en")
                            || name.starts_with("eth")
                            || name.starts_with("ens")
                        {
                            self.network_status = NetworkStatus::Ethernet;
                            self.network_name = name;
                            return;
                        }
                    }
                }
            }
        }

        self.network_status = NetworkStatus::Disconnected;
        self.network_name.clear();
    }

    // ---- Public accessors for the renderer ----

    /// Get the formatted clock string
    pub fn clock_text(&self) -> &str {
        &self.clock_text
    }

    /// Get battery percentage (-1 if no battery)
    pub fn battery_percent(&self) -> i32 {
        self.battery_percent
    }

    /// Whether the battery is charging
    pub fn is_charging(&self) -> bool {
        self.battery_charging
    }

    /// Get a display string for battery status
    pub fn battery_text(&self) -> String {
        if self.battery_percent < 0 {
            "AC".to_string()
        } else {
            let icon = if self.battery_charging {
                "⚡"
            } else if self.battery_percent > 80 {
                "█"
            } else if self.battery_percent > 60 {
                "▓"
            } else if self.battery_percent > 40 {
                "▒"
            } else if self.battery_percent > 20 {
                "░"
            } else {
                "!"
            };
            format!("{icon} {:.0}%", self.battery_percent)
        }
    }

    /// Get network status
    pub fn network_status(&self) -> &NetworkStatus {
        &self.network_status
    }

    /// Get a display string for network status
    pub fn network_text(&self) -> String {
        match &self.network_status {
            NetworkStatus::WiFi => format!("WiFi: {}", self.network_name),
            NetworkStatus::Ethernet => format!("Eth: {}", self.network_name),
            NetworkStatus::Disconnected => "Disconnected".to_string(),
            NetworkStatus::Unknown => "Network: ?".to_string(),
        }
    }

    /// Handle a click on the panel area
    /// Returns true if the click was consumed
    pub fn handle_click(&mut self, x: f64, _y: f64) -> bool {
        // Left side (first 100px) — "heyOS" button / launcher trigger
        if x < 100.0 {
            debug!("Panel: heyOS button clicked");
            return true; // The caller should toggle the launcher
        }

        false
    }
}
