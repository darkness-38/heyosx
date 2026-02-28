// =============================================================================
// heyDM — Window Manager
//
// Manages all toplevel windows: tracking, positioning, focusing, moving,
// resizing, tiling, and fullscreen. Maintains a stack-ordered list of
// windows and a cursor position.
// =============================================================================

use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Logical, Physical, Point, Rectangle, Size};
use smithay::wayland::shell::xdg::ToplevelSurface;

use tracing::{debug, info};

/// Represents a single toplevel window in the compositor
#[derive(Debug, Clone)]
pub struct WindowElement {
    /// The XDG toplevel surface
    toplevel: ToplevelSurface,
    /// Position of the window in output coordinates
    position: Point<i32, Logical>,
    /// Size of the window
    size: Size<i32, Logical>,
    /// Whether the window is fullscreen
    fullscreen: bool,
    /// Saved geometry before fullscreen (for restore)
    saved_geometry: Option<Rectangle<i32, Logical>>,
}

impl WindowElement {
    /// Create a new window element from an XDG toplevel surface
    pub fn new(toplevel: ToplevelSurface) -> Self {
        Self {
            toplevel,
            position: Point::from((100, 100)),
            size: Size::from((800, 600)),
            fullscreen: false,
            saved_geometry: None,
        }
    }

    /// Get the XDG toplevel surface
    pub fn toplevel(&self) -> &ToplevelSurface {
        &self.toplevel
    }

    /// Get the window's bounding rectangle
    pub fn geometry(&self) -> Rectangle<i32, Logical> {
        Rectangle::new(self.position, self.size)
    }

    /// Set the window position
    pub fn set_position(&mut self, pos: Point<i32, Logical>) {
        self.position = pos;
    }

    /// Set the window size
    pub fn set_size(&mut self, size: Size<i32, Logical>) {
        self.size = size;
    }

    /// Check if a point is inside this window
    pub fn contains_point(&self, point: (f64, f64)) -> bool {
        let rect = self.geometry();
        point.0 >= rect.loc.x as f64
            && point.0 <= (rect.loc.x + rect.size.w) as f64
            && point.1 >= rect.loc.y as f64
            && point.1 <= (rect.loc.y + rect.size.h) as f64
    }

    /// Get the WlSurface associated with this window (clones the Arc-backed handle)
    pub fn wl_surface(&self) -> Option<WlSurface> {
        Some(self.toplevel.wl_surface().clone())
    }
}

/// The window manager tracks all windows and manages focus, layout, etc.
pub struct WindowManager {
    /// All managed windows, in stack order (last = topmost)
    windows: Vec<WindowElement>,
    /// Index of the currently focused window (None if no windows)
    focused: Option<usize>,
    /// Current cursor position
    cursor_pos: (f64, f64),
    /// Active grab state (for moving/resizing)
    grab: Option<GrabState>,
    /// Panel height (reserved space at top)
    panel_height: i32,
}

/// State for an active pointer grab (move or resize)
#[derive(Debug, Clone)]
struct GrabState {
    /// Index of the window being grabbed
    window_index: usize,
    /// Type of grab
    kind: GrabKind,
    /// Initial cursor position when the grab started
    initial_cursor: (f64, f64),
    /// Initial window position when the grab started
    initial_window_pos: Point<i32, Logical>,
    /// Initial window size when the grab started
    initial_window_size: Size<i32, Logical>,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
enum GrabKind {
    Move,
    Resize,
}

#[allow(dead_code)]
impl WindowManager {
    /// Create a new empty window manager
    pub fn new() -> Self {
        Self {
            windows: Vec::new(),
            focused: None,
            cursor_pos: (0.0, 0.0),
            grab: None,
            panel_height: 32,
        }
    }

    /// Add a new window to the manager
    pub fn add_window(
        &mut self,
        mut window: WindowElement,
        output_size: &Size<i32, Physical>,
    ) {
        // Center the window on screen, below the panel
        let x = (output_size.w - window.size.w) / 2;
        let y = self.panel_height + (output_size.h - self.panel_height - window.size.h) / 2;
        window.set_position(Point::from((x.max(0), y.max(self.panel_height))));

        self.windows.push(window);
        self.focused = Some(self.windows.len() - 1);

        info!(
            "Window added (total: {}), focused: {:?}",
            self.windows.len(),
            self.focused
        );
    }

    /// Remove a window by its toplevel surface
    pub fn remove_window(&mut self, surface: &ToplevelSurface) {
        if let Some(idx) = self
            .windows
            .iter()
            .position(|w| &w.toplevel == surface)
        {
            self.windows.remove(idx);

            // Update focus
            if self.windows.is_empty() {
                self.focused = None;
            } else if let Some(focused) = self.focused {
                if focused >= self.windows.len() {
                    self.focused = Some(self.windows.len() - 1);
                } else if focused > idx {
                    self.focused = Some(focused - 1);
                }
            }

            info!(
                "Window removed (total: {}), focused: {:?}",
                self.windows.len(),
                self.focused
            );
        }
    }

    /// Handle a surface commit (update window geometry)
    pub fn handle_commit(&mut self, _surface: &WlSurface) {
        // Update internal geometry tracking based on committed state
        // In a full implementation, this would read the surface's committed size
    }

    /// Get all windows in stack order
    pub fn windows(&self) -> &[WindowElement] {
        &self.windows
    }

    /// Get the currently focused window
    pub fn focused_window(&self) -> Option<&WindowElement> {
        self.focused.map(|idx| &self.windows[idx])
    }

    /// Close the currently focused window
    pub fn close_focused(&mut self) {
        if let Some(idx) = self.focused {
            if idx < self.windows.len() {
                // Send a close request to the toplevel
                self.windows[idx].toplevel.send_close();
            }
        }
    }

    /// Toggle fullscreen for the focused window
    pub fn toggle_fullscreen(&mut self, output_size: &Size<i32, Physical>) {
        if let Some(idx) = self.focused {
            if idx < self.windows.len() {
                let window = &mut self.windows[idx];
                if window.fullscreen {
                    // Restore from fullscreen
                    if let Some(saved) = window.saved_geometry.take() {
                        window.set_position(saved.loc);
                        window.set_size(saved.size);
                    }
                    window.fullscreen = false;
                    info!("Window exited fullscreen");
                } else {
                    // Save current geometry and go fullscreen
                    window.saved_geometry = Some(window.geometry());
                    window.set_position(Point::from((0, 0)));
                    window.set_size(Size::from((output_size.w, output_size.h)));
                    window.fullscreen = true;
                    info!("Window entered fullscreen");
                }
            }
        }
    }

    /// Tile the focused window to the left half of the screen
    pub fn tile_left(&mut self, output_size: &Size<i32, Physical>) {
        if let Some(idx) = self.focused {
            if idx < self.windows.len() {
                let window = &mut self.windows[idx];
                window.set_position(Point::from((0, self.panel_height)));
                window.set_size(Size::from((
                    output_size.w / 2,
                    output_size.h - self.panel_height,
                )));
                window.fullscreen = false;
                info!("Window tiled to left half");
            }
        }
    }

    /// Tile the focused window to the right half of the screen
    pub fn tile_right(&mut self, output_size: &Size<i32, Physical>) {
        if let Some(idx) = self.focused {
            if idx < self.windows.len() {
                let window = &mut self.windows[idx];
                window.set_position(Point::from((
                    output_size.w / 2,
                    self.panel_height,
                )));
                window.set_size(Size::from((
                    output_size.w / 2,
                    output_size.h - self.panel_height,
                )));
                window.fullscreen = false;
                info!("Window tiled to right half");
            }
        }
    }

    /// Cycle focus to the next window
    pub fn cycle_focus(&mut self) {
        if self.windows.len() <= 1 {
            return;
        }

        self.focused = Some(match self.focused {
            Some(idx) => (idx + 1) % self.windows.len(),
            None => 0,
        });

        // Raise the focused window to the top of the stack
        if let Some(idx) = self.focused {
            let window = self.windows.remove(idx);
            self.windows.push(window);
            self.focused = Some(self.windows.len() - 1);
        }

        debug!("Focus cycled to window {:?}", self.focused);
    }

    /// Focus the window at the given screen position
    pub fn focus_at(&mut self, pos: (f64, f64)) {
        // Search from top of stack (last) to bottom (first)
        let found = self
            .windows
            .iter()
            .enumerate()
            .rev()
            .find(|(_, w)| w.contains_point(pos))
            .map(|(idx, _)| idx);

        if let Some(idx) = found {
            self.focused = Some(idx);

            // Raise to top of stack
            let window = self.windows.remove(idx);
            self.windows.push(window);
            self.focused = Some(self.windows.len() - 1);
        }
    }

    /// Find the Wayland surface under the given screen position (returns owned WlSurface)
    pub fn surface_under(&self, pos: (f64, f64)) -> Option<(WlSurface, (f64, f64))> {
        for window in self.windows.iter().rev() {
            if window.contains_point(pos) {
                if let Some(surface) = window.wl_surface() {
                    let relative_pos = (
                        pos.0 - window.position.x as f64,
                        pos.1 - window.position.y as f64,
                    );
                    return Some((surface, relative_pos));
                }
            }
        }
        None
    }

    // ---- Cursor management ----

    /// Get current cursor position
    pub fn cursor_position(&self) -> (f64, f64) {
        self.cursor_pos
    }

    /// Set absolute cursor position
    pub fn set_cursor_position(&mut self, x: f64, y: f64) {
        self.cursor_pos = (x, y);
    }

    /// Update cursor position by a relative delta, clamped to output bounds
    pub fn update_cursor_relative(
        &mut self,
        dx: f64,
        dy: f64,
        output_size: Size<i32, Physical>,
    ) -> (f64, f64) {
        self.cursor_pos.0 = (self.cursor_pos.0 + dx).clamp(0.0, output_size.w as f64);
        self.cursor_pos.1 = (self.cursor_pos.1 + dy).clamp(0.0, output_size.h as f64);
        self.cursor_pos
    }

    // ---- Pointer grab (move/resize) ----

    /// Handle pointer motion during an active grab
    pub fn handle_pointer_motion(&mut self, pos: (f64, f64)) -> bool {
        let grab = match &self.grab {
            Some(g) => g.clone(),
            None => return false,
        };

        let dx = pos.0 - grab.initial_cursor.0;
        let dy = pos.1 - grab.initial_cursor.1;

        match grab.kind {
            GrabKind::Move => {
                if grab.window_index < self.windows.len() {
                    let new_x = grab.initial_window_pos.x + dx as i32;
                    let new_y = grab.initial_window_pos.y + dy as i32;
                    self.windows[grab.window_index]
                        .set_position(Point::from((new_x, new_y.max(self.panel_height))));
                }
            }
            GrabKind::Resize => {
                if grab.window_index < self.windows.len() {
                    let new_w = (grab.initial_window_size.w + dx as i32).max(200);
                    let new_h = (grab.initial_window_size.h + dy as i32).max(150);
                    self.windows[grab.window_index].set_size(Size::from((new_w, new_h)));
                }
            }
        }

        true
    }

    /// Start a move grab on the focused window
    pub fn begin_move(&mut self) {
        if let Some(idx) = self.focused {
            if idx < self.windows.len() {
                self.grab = Some(GrabState {
                    window_index: idx,
                    kind: GrabKind::Move,
                    initial_cursor: self.cursor_pos,
                    initial_window_pos: self.windows[idx].position,
                    initial_window_size: self.windows[idx].size,
                });
                debug!("Move grab started on window {idx}");
            }
        }
    }

    /// Start a resize grab on the focused window
    pub fn begin_resize(&mut self) {
        if let Some(idx) = self.focused {
            if idx < self.windows.len() {
                self.grab = Some(GrabState {
                    window_index: idx,
                    kind: GrabKind::Resize,
                    initial_cursor: self.cursor_pos,
                    initial_window_pos: self.windows[idx].position,
                    initial_window_size: self.windows[idx].size,
                });
                debug!("Resize grab started on window {idx}");
            }
        }
    }

    /// End any active grab
    pub fn end_grab(&mut self) {
        if self.grab.is_some() {
            debug!("Grab ended");
            self.grab = None;
        }
    }
}
