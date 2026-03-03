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
    /// Target position of the window in output coordinates
    position: Point<i32, Logical>,
    /// Target size of the window
    size: Size<i32, Logical>,
    /// Current animated position
    pub current_position: Point<f64, Logical>,
    /// Current animated size
    pub current_size: Size<f64, Logical>,
    /// Current opacity (0.0 to 1.0)
    pub opacity: f32,
    /// Current scale (e.g., 1.0 is normal)
    pub scale: f32,
    /// Whether the window is fullscreen
    fullscreen: bool,
    /// Saved geometry before fullscreen (for restore)
    saved_geometry: Option<Rectangle<i32, Logical>>,
    /// Workspace index (0-8)
    pub workspace: usize,
}

impl WindowElement {
    /// Create a new window element from an XDG toplevel surface
    pub fn new(toplevel: ToplevelSurface) -> Self {
        let position = Point::from((100, 100));
        let size = Size::from((800, 600));
        Self {
            toplevel,
            position,
            size,
            current_position: Point::from((position.x as f64, position.y as f64)),
            current_size: Size::from((size.w as f64, size.h as f64)),
            opacity: 0.0, // Start invisible for fade-in
            scale: 0.9,   // Start slightly smaller for scale-in
            fullscreen: false,
            saved_geometry: None,
            workspace: 0,
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

    /// Check if a point is inside this window (using current animated geometry)
    pub fn contains_point(&self, point: (f64, f64)) -> bool {
        let x = self.current_position.x;
        let y = self.current_position.y;
        let w = self.current_size.w;
        let h = self.current_size.h;
        
        point.0 >= x && point.0 <= x + w && point.1 >= y && point.1 <= y + h
    }

    /// Get the WlSurface associated with this window (clones the Arc-backed handle)
    pub fn wl_surface(&self) -> Option<WlSurface> {
        Some(self.toplevel.wl_surface().clone())
    }
}

/// Available window layouts
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Layout {
    Floating,
    Tiling,
}

/// The window manager tracks all windows and manages focus, layout, etc.
pub struct WindowManager {
    /// All managed windows, in stack order (last = topmost)
    windows: Vec<WindowElement>,
    /// Index of the currently focused window (None if no windows)
    focused: Option<usize>,
    /// Current workspace (0-8)
    pub current_workspace: usize,
    /// Current animated workspace offset (for sliding transitions)
    pub current_workspace_offset: f64,
    /// Current animated focus position
    pub current_focus_pos: Point<f64, Logical>,
    /// Current animated focus size
    pub current_focus_size: Size<f64, Logical>,
    /// Current focus indicator opacity
    pub focus_opacity: f32,
    /// Current cursor position
    cursor_pos: (f64, f64),
    /// Active grab state (for moving/resizing)
    grab: Option<GrabState>,
    /// Panel height (reserved space at top)
    panel_height: i32,
    /// Current layout mode
    pub layout: Layout,
    /// Number of windows in the master area (for tiling)
    pub master_count: usize,
    /// Ratio of the screen taken by the master area (0.0 to 1.0)
    pub master_ratio: f32,
    /// Gap size between windows in pixels
    pub gaps: i32,
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
            current_workspace: 0,
            current_workspace_offset: 0.0,
            current_focus_pos: Point::from((0.0, 0.0)),
            current_focus_size: Size::from((0.0, 0.0)),
            focus_opacity: 0.0,
            cursor_pos: (0.0, 0.0),
            grab: None,
            panel_height: 40,
            layout: Layout::Tiling,
            master_count: 1,
            master_ratio: 0.5,
            gaps: 10,
        }
    }

    /// Recompute window layout based on current mode
    pub fn recompute_layout(&mut self, output_size: &Size<i32, Physical>) {
        if self.layout == Layout::Floating {
            // Floating layout - windows maintain their own geometry
            return;
        }

        // Get list of windows that should be tiled (exclude fullscreen and only on current workspace)
        let tiled_windows: Vec<usize> = self.windows
            .iter()
            .enumerate()
            .filter(|(_, w)| !w.fullscreen && w.workspace == self.current_workspace)
            .map(|(i, _)| i)
            .collect();

        if tiled_windows.is_empty() {
            return;
        }

        let n = tiled_windows.len();
        
        // Useable area (subtract panel and apply outer gaps)
        let area_x = self.gaps;
        let area_y = self.panel_height + self.gaps;
        let area_w = output_size.w - 2 * self.gaps;
        let area_h = (output_size.h - self.panel_height - 2 * self.gaps).max(0);

        if n <= self.master_count {
            // All windows in master area (full width)
            let total_gaps = self.gaps * (n as i32 - 1);
            let win_h = (area_h - total_gaps) / n as i32;
            
            for (i, &win_idx) in tiled_windows.iter().enumerate() {
                let y = area_y + i as i32 * (win_h + self.gaps);
                self.windows[win_idx].set_position(Point::from((area_x, y)));
                self.windows[win_idx].set_size(Size::from((area_w, win_h)));
            }
        } else {
            // Split into Master and Stack
            let m = self.master_count;
            let s = n - m;

            let master_area_w = ((area_w - self.gaps) as f32 * self.master_ratio) as i32;
            let stack_area_w = area_w - master_area_w - self.gaps;
            let stack_x = area_x + master_area_w + self.gaps;

            // Master area
            let m_total_gaps = self.gaps * (m as i32 - 1);
            let m_win_h = (area_h - m_total_gaps) / m as i32;
            
            for (i, &win_idx) in tiled_windows.iter().take(m).enumerate() {
                let y = area_y + i as i32 * (m_win_h + self.gaps);
                self.windows[win_idx].set_position(Point::from((area_x, y)));
                self.windows[win_idx].set_size(Size::from((master_area_w, m_win_h)));
            }

            // Stack area
            let s_total_gaps = self.gaps * (s as i32 - 1);
            let s_win_h = (area_h - s_total_gaps) / s as i32;
            
            for (i, &win_idx) in tiled_windows.iter().skip(m).enumerate() {
                let y = area_y + i as i32 * (s_win_h + self.gaps);
                self.windows[win_idx].set_position(Point::from((stack_x, y)));
                self.windows[win_idx].set_size(Size::from((stack_area_w, s_win_h)));
            }
        }

        debug!("Tiling layout recomputed for {} windows", n);
    }

    pub fn inc_master_count(&mut self, output_size: &Size<i32, Physical>) {
        self.master_count += 1;
        info!("Master count incremented to {}", self.master_count);
        self.recompute_layout(output_size);
    }

    pub fn dec_master_count(&mut self, output_size: &Size<i32, Physical>) {
        if self.master_count > 1 {
            self.master_count -= 1;
            info!("Master count decremented to {}", self.master_count);
            self.recompute_layout(output_size);
        }
    }

    pub fn inc_master_ratio(&mut self, output_size: &Size<i32, Physical>) {
        self.master_ratio = (self.master_ratio + 0.05).min(0.9);
        info!("Master ratio incremented to {:.2}", self.master_ratio);
        self.recompute_layout(output_size);
    }

    pub fn dec_master_ratio(&mut self, output_size: &Size<i32, Physical>) {
        self.master_ratio = (self.master_ratio - 0.05).max(0.1);
        info!("Master ratio decremented to {:.2}", self.master_ratio);
        self.recompute_layout(output_size);
    }

    pub fn toggle_layout(&mut self, output_size: &Size<i32, Physical>) {
        self.layout = match self.layout {
            Layout::Floating => Layout::Tiling,
            Layout::Tiling => Layout::Floating,
        };
        info!("Layout toggled to {:?}", self.layout);
        self.recompute_layout(output_size);
    }

    /// Add a new window to the manager
    pub fn add_window(
        &mut self,
        mut window: WindowElement,
        output_size: &Size<i32, Physical>,
    ) {
        window.workspace = self.current_workspace;
        self.windows.push(window);
        self.focused = Some(self.windows.len() - 1);

        info!(
            "Window added to workspace {} (total: {}), focused: {:?}",
            self.current_workspace,
            self.windows.len(),
            self.focused
        );

        self.recompute_layout(output_size);
    }

    /// Switch to a different workspace
    pub fn switch_workspace(&mut self, new_workspace: usize, output_size: &Size<i32, Physical>) {
        if new_workspace == self.current_workspace {
            return;
        }

        info!("Switching to workspace {}", new_workspace);
        self.current_workspace = new_workspace;

        // Update focus to the topmost window on the new workspace
        self.focused = self.windows
            .iter()
            .enumerate()
            .rev()
            .find(|(_, w)| w.workspace == self.current_workspace)
            .map(|(idx, _)| idx);

        self.recompute_layout(output_size);
    }

    /// Move the focused window to another workspace
    pub fn move_focused_to_workspace(&mut self, target_workspace: usize, output_size: &Size<i32, Physical>) {
        if let Some(idx) = self.focused {
            info!("Moving focused window to workspace {}", target_workspace);
            self.windows[idx].workspace = target_workspace;

            // Focus the next available window on the current workspace
            self.focused = self.windows
                .iter()
                .enumerate()
                .rev()
                .find(|(_, w)| w.workspace == self.current_workspace)
                .map(|(idx, _)| idx);

            self.recompute_layout(output_size);
        }
    }

    /// Remove a window by its toplevel surface
    pub fn remove_window(&mut self, surface: &ToplevelSurface, output_size: &Size<i32, Physical>) {
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
                    // Focus the topmost window on the current workspace
                    self.focused = self.windows
                        .iter()
                        .enumerate()
                        .rev()
                        .find(|(_, w)| w.workspace == self.current_workspace)
                        .map(|(idx, _)| idx);
                } else if focused > idx {
                    self.focused = Some(focused - 1);
                }
            }

            info!(
                "Window removed (total: {}), focused: {:?}",
                self.windows.len(),
                self.focused
            );

            self.recompute_layout(output_size);
        }
    }

    /// Handle a surface commit (update window geometry)
    pub fn handle_commit(&mut self, _surface: &WlSurface) {
        // Update internal geometry tracking based on committed state
        // In a full implementation, this would read the surface's committed size
    }

    /// Get all windows visible on the current workspace (in stack order)
    pub fn windows(&self) -> Vec<&WindowElement> {
        // If we are animating between workspaces, return all windows so we can draw the ones sliding out/in
        if (self.current_workspace_offset - self.current_workspace as f64).abs() > 0.001 {
            self.windows.iter().collect()
        } else {
            self.windows.iter().filter(|w| w.workspace == self.current_workspace).collect()
        }
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
                
                self.recompute_layout(output_size);
            }
        }
    }

    /// Cycle focus to the next window on the current workspace
    pub fn cycle_focus(&mut self) {
        let visible_indices: Vec<usize> = self.windows
            .iter()
            .enumerate()
            .filter(|(_, w)| w.workspace == self.current_workspace)
            .map(|(i, _)| i)
            .collect();

        if visible_indices.len() <= 1 {
            return;
        }

        let current_visible_idx = visible_indices.iter().position(|&idx| Some(idx) == self.focused);
        
        let next_idx = match current_visible_idx {
            Some(i) => visible_indices[(i + 1) % visible_indices.len()],
            None => visible_indices[0],
        };

        self.focused = Some(next_idx);

        // Raise the focused window to the top of the stack
        let window = self.windows.remove(next_idx);
        self.windows.push(window);
        self.focused = Some(self.windows.len() - 1);

        debug!("Focus cycled to window {:?}", self.focused);
    }

    /// Focus the window at the given screen position (only on current workspace)
    pub fn focus_at(&mut self, pos: (f64, f64)) {
        // Search from top of stack (last) to bottom (first)
        let found = self
            .windows
            .iter()
            .enumerate()
            .rev()
            .find(|(_, w)| w.workspace == self.current_workspace && w.contains_point(pos))
            .map(|(idx, _)| idx);

        if let Some(idx) = found {
            self.focused = Some(idx);

            // Raise to top of stack
            let window = self.windows.remove(idx);
            self.windows.push(window);
            self.focused = Some(self.windows.len() - 1);
        }
    }

    /// Find the Wayland surface under the given screen position (only on current workspace)
    pub fn surface_under(&self, pos: (f64, f64)) -> Option<(WlSurface, (f64, f64))> {
        for window in self.windows.iter().rev() {
            if window.workspace == self.current_workspace && window.contains_point(pos) {
                if let Some(surface) = window.wl_surface() {
                    let relative_pos = (
                        pos.0 - window.current_position.x,
                        pos.1 - window.current_position.y,
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

    /// Update animations for all windows
    pub fn update_animations(&mut self, dt: std::time::Duration) {
        let dt_secs = dt.as_secs_f64();
        // Easing factor (lower is smoother, higher is faster)
        let factor = 12.0;
        let focus_factor = 10.0; // Slightly slower for "magnetic" feel

        // Interpolate workspace offset
        self.current_workspace_offset += (self.current_workspace as f64 - self.current_workspace_offset) * factor * dt_secs;

        // Update focus indicator
        if let Some(idx) = self.focused {
            let window = &self.windows[idx];
            let target_pos = window.position;
            let target_size = window.size;

            // Interpolate focus box geometry towards target window geometry
            self.current_focus_pos.x += (target_pos.x as f64 - self.current_focus_pos.x) * focus_factor * dt_secs;
            self.current_focus_pos.y += (target_pos.y as f64 - self.current_focus_pos.y) * focus_factor * dt_secs;
            self.current_focus_size.w += (target_size.w as f64 - self.current_focus_size.w) * focus_factor * dt_secs;
            self.current_focus_size.h += (target_size.h as f64 - self.current_focus_size.h) * focus_factor * dt_secs;

            // Fade in
            self.focus_opacity += (1.0 - self.focus_opacity) * focus_factor as f32 * dt_secs as f32;
        } else {
            // Fade out
            self.focus_opacity += (0.0 - self.focus_opacity) * focus_factor as f32 * dt_secs as f32;
        }

        for window in &mut self.windows {
            // Update position
            let target_pos = window.position;
            window.current_position.x += (target_pos.x as f64 - window.current_position.x) * factor * dt_secs;
            window.current_position.y += (target_pos.y as f64 - window.current_position.y) * factor * dt_secs;

            // Update size
            let target_size = window.size;
            window.current_size.w += (target_size.w as f64 - window.current_size.w) * factor * dt_secs;
            window.current_size.h += (target_size.h as f64 - window.current_size.h) * factor * dt_secs;

            // Update opacity (target is always 1.0 when active)
            window.opacity += (1.0 - window.opacity) * factor as f32 * dt_secs as f32;

            // Update scale (target is always 1.0)
            window.scale += (1.0 - window.scale) * factor as f32 * dt_secs as f32;
        }
    }
}
