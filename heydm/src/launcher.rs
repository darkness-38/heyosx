// =============================================================================
// heyDM — Application Launcher
//
// A dynamic application launcher that:
//   1. Scans XDG .desktop files from standard paths
//   2. Presents a searchable list of installed applications
//   3. Launches the selected application
//
// Toggled with Super+D and rendered as a centered overlay by the renderer.
// =============================================================================

use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Represents a launchable application parsed from a .desktop file
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AppEntry {
    /// Display name of the application (e.g., "Firefox Web Browser")
    pub name: String,
    /// Generic name / subtitle (e.g., "Web Browser")
    pub generic_name: String,
    /// The Exec= command to launch the application
    pub exec: String,
    /// Optional icon name
    pub icon: String,
    /// Categories for filtering
    pub categories: Vec<String>,
    /// Source .desktop file path
    pub desktop_file: PathBuf,
}

/// The application launcher overlay
pub struct AppLauncher {
    /// All discovered applications
    apps: Vec<AppEntry>,
    /// Current search query
    search_query: String,
    /// Filtered results based on search query
    filtered: Vec<usize>, // indices into `apps`
    /// Currently selected item index in `filtered`
    selected: usize,
    /// Whether the launcher is currently visible
    visible: bool,
}

#[allow(dead_code)]
impl AppLauncher {
    /// Create a new launcher, scanning for .desktop files
    pub fn new() -> Self {
        let mut launcher = Self {
            apps: Vec::new(),
            search_query: String::new(),
            filtered: Vec::new(),
            selected: 0,
            visible: false,
        };

        launcher.scan_desktop_files();
        launcher.update_filter();

        info!("Application launcher initialized: {} apps found", launcher.apps.len());
        launcher
    }

    /// Scan standard XDG directories for .desktop files
    fn scan_desktop_files(&mut self) {
        let search_dirs = [
            "/usr/share/applications",
            "/usr/local/share/applications",
            "/var/lib/flatpak/exports/share/applications",
        ];

        // Also check user-specific directory
        let home = std::env::var("HOME").unwrap_or_default();
        let user_dir = format!("{home}/.local/share/applications");

        let mut all_dirs: Vec<&str> = search_dirs.to_vec();
        if !home.is_empty() {
            all_dirs.push(&user_dir);
        }

        for dir in all_dirs {
            let dir_path = Path::new(dir);
            if !dir_path.exists() {
                continue;
            }

            debug!("Scanning .desktop files in: {dir}");
            self.scan_directory(dir_path);
        }

        // Sort applications alphabetically by name
        self.apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    }

    /// Scan a single directory for .desktop files
    fn scan_directory(&mut self, dir: &Path) {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
                continue;
            }

            if let Some(app) = self.parse_desktop_file(&path) {
                self.apps.push(app);
            }
        }
    }

    /// Parse a single .desktop file into an AppEntry
    fn parse_desktop_file(&self, path: &Path) -> Option<AppEntry> {
        let content = fs::read_to_string(path).ok()?;

        let mut name = String::new();
        let mut generic_name = String::new();
        let mut exec = String::new();
        let mut icon = String::new();
        let mut categories = Vec::new();
        let mut no_display = false;
        let mut hidden = false;
        let mut in_desktop_entry = false;

        for line in content.lines() {
            let line = line.trim();

            // Track section headers
            if line.starts_with('[') {
                in_desktop_entry = line == "[Desktop Entry]";
                continue;
            }

            if !in_desktop_entry {
                continue;
            }

            // Parse key=value pairs
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();

                match key {
                    "Name" if name.is_empty() => name = value.to_string(),
                    "GenericName" if generic_name.is_empty() => {
                        generic_name = value.to_string()
                    }
                    "Exec" if exec.is_empty() => {
                        // Remove field codes like %f, %u, %U, etc.
                        exec = value
                            .replace("%f", "")
                            .replace("%F", "")
                            .replace("%u", "")
                            .replace("%U", "")
                            .replace("%d", "")
                            .replace("%D", "")
                            .replace("%n", "")
                            .replace("%N", "")
                            .replace("%k", "")
                            .replace("%v", "")
                            .trim()
                            .to_string();
                    }
                    "Icon" if icon.is_empty() => icon = value.to_string(),
                    "Categories" => {
                        categories = value
                            .split(';')
                            .map(|c| c.trim().to_string())
                            .filter(|c| !c.is_empty())
                            .collect();
                    }
                    "NoDisplay" => no_display = value.eq_ignore_ascii_case("true"),
                    "Hidden" => hidden = value.eq_ignore_ascii_case("true"),
                    _ => {}
                }
            }
        }

        // Skip hidden or NoDisplay entries
        if no_display || hidden {
            return None;
        }

        // Must have both a name and an exec command
        if name.is_empty() || exec.is_empty() {
            return None;
        }

        Some(AppEntry {
            name,
            generic_name,
            exec,
            icon,
            categories,
            desktop_file: path.to_path_buf(),
        })
    }

    // ---- State management ----

    /// Toggle the launcher visibility
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        if self.visible {
            // Reset state when opening
            self.search_query.clear();
            self.selected = 0;
            self.update_filter();
            info!("Launcher opened");
        } else {
            info!("Launcher closed");
        }
    }

    /// Show the launcher
    pub fn show(&mut self) {
        self.visible = true;
        self.search_query.clear();
        self.selected = 0;
        self.update_filter();
    }

    /// Hide the launcher
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Whether the launcher is currently visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Get the index of the currently selected item
    pub fn selected_index(&self) -> Option<usize> {
        if self.filtered.is_empty() {
            None
        } else {
            Some(self.selected)
        }
    }

    // ---- Search filtering ----

    /// Update the filtered list based on the current search query
    fn update_filter(&mut self) {
        let query = self.search_query.to_lowercase();

        if query.is_empty() {
            // Show all apps
            self.filtered = (0..self.apps.len()).collect();
        } else {
            // Filter by name, generic name, and categories
            self.filtered = self
                .apps
                .iter()
                .enumerate()
                .filter(|(_, app)| {
                    app.name.to_lowercase().contains(&query)
                        || app.generic_name.to_lowercase().contains(&query)
                        || app
                            .categories
                            .iter()
                            .any(|c| c.to_lowercase().contains(&query))
                })
                .map(|(idx, _)| idx)
                .collect();
        }

        // Clamp selection
        if self.selected >= self.filtered.len() && !self.filtered.is_empty() {
            self.selected = self.filtered.len() - 1;
        }
    }

    /// Add a character to the search query
    pub fn type_char(&mut self, ch: char) {
        self.search_query.push(ch);
        self.selected = 0;
        self.update_filter();
    }

    /// Remove the last character from the search query
    pub fn backspace(&mut self) {
        self.search_query.pop();
        self.selected = 0;
        self.update_filter();
    }

    /// Move selection up
    pub fn select_prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        if self.selected + 1 < self.filtered.len() {
            self.selected += 1;
        }
    }

    /// Get the exec command of the currently selected app
    pub fn get_selected_exec(&self) -> Option<&str> {
        let idx = *self.filtered.get(self.selected)?;
        Some(&self.apps[idx].exec)
    }

    /// Get the display entries (name + generic name) for the currently visible items
    pub fn visible_entries(&self) -> Vec<(&str, &str)> {
        self.filtered
            .iter()
            .map(|&idx| {
                (
                    self.apps[idx].name.as_str(),
                    self.apps[idx].generic_name.as_str(),
                )
            })
            .collect()
    }

    /// Get search query
    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    // ---- Click handling ----

    /// Handle a click on the launcher overlay
    /// Returns Some(exec_command) if an app was selected, None otherwise
    pub fn handle_click(&self, x: f64, y: f64) -> Option<String> {
        if !self.visible {
            return None;
        }

        // Calculate launcher geometry (must match renderer)
        let output_w = 1920; // These would ideally be passed in
        let output_h = 1080;

        let launcher_w = 800.min(output_w - 100);
        let launcher_h = 600.min(output_h - 200);
        let launcher_x = (output_w - launcher_w) / 2;
        let launcher_y = (output_h - launcher_h) / 2;

        if x < launcher_x as f64 || x > (launcher_x + launcher_w) as f64 || y < launcher_y as f64 || y > (launcher_y + launcher_h) as f64 {
            return None;
        }

        let search_bar_h = 50;
        let items_start_y = launcher_y + 20 + search_bar_h + 20; // 90
        
        if y < items_start_y as f64 {
            return None; // clicked search bar
        }
        
        let cols = 4;
        let item_w = (launcher_w - 60) / cols;
        let item_h = 100;
        
        let col = ((x - (launcher_x as f64 + 30.0)) / item_w as f64) as i32;
        let row = ((y - items_start_y as f64) / item_h as f64) as i32;
        
        if col < 0 || col >= cols || row < 0 || row >= 3 {
            return None; // outside grid
        }
        
        let clicked_idx = (row * cols + col) as usize;

        if let Some(&app_idx) = self.filtered.get(clicked_idx) {
            let exec = self.apps[app_idx].exec.clone();
            info!("Launcher: selected '{}' → {}", self.apps[app_idx].name, exec);
            Some(exec)
        } else {
            None
        }
    }
}
