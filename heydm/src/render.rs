// =============================================================================
// heyDM — Renderer
//
// Renders the desktop: background, windows, panel, launcher, cursor.
// Uses a GlesFrame obtained from the winit/DRM backend's render surface.
// =============================================================================

use smithay::backend::renderer::Frame;
use smithay::output::Output;
use smithay::utils::{Physical, Rectangle, Size};

use crate::state::HeyDM;

/// Color constants for the heyOS desktop theme (End-4 inspired)
pub mod colors {
    pub const BG_DARK: [f32; 4]            = [0.04, 0.04, 0.06, 1.0];
    pub const PANEL_BG: [f32; 4]           = [0.08, 0.08, 0.12, 0.95];
    pub const ACCENT_CRIMSON: [f32; 4]     = [0.83, 0.23, 0.28, 1.0];
    pub const ACCENT_CYAN: [f32; 4]        = [0.29, 0.70, 0.83, 1.0];
    pub const LAUNCHER_BG: [f32; 4]        = [0.06, 0.06, 0.09, 0.98];
    pub const BORDER_FOCUSED: [f32; 4]     = [0.83, 0.23, 0.28, 1.0]; // Crimson
    pub const BORDER_UNFOCUSED: [f32; 4]   = [0.15, 0.15, 0.20, 0.60];
}

pub const PANEL_HEIGHT: i32 = 44;
pub const PANEL_MARGIN: i32 = 10;
pub const BORDER_WIDTH: i32 = 3;

/// Build a Rectangle from (x, y, w, h)
fn rect(x: i32, y: i32, w: i32, h: i32) -> Rectangle<i32, Physical> {
    Rectangle::new((x, y).into(), (w, h).into())
}

pub struct Renderer;

impl Renderer {
    /// Render a full frame into the given frame.
    pub fn render_frame<F: Frame>(
        state: &HeyDM,
        frame: &mut F,
        _output: &Output,
        output_size: Size<i32, Physical>,
    ) -> Result<(), Box<dyn std::error::Error>> 
    where F::Error: 'static
    {
        // ---- 1. Background ----
        frame.clear(
            colors::BG_DARK.into(),
            &[rect(0, 0, output_size.w, output_size.h)],
        )?;

        // ---- 2. Windows ----
        let focused_idx = state.window_manager.windows().len().checked_sub(1);
        for (idx, window) in state.window_manager.windows().iter().enumerate() {
            let geom = window.geometry();
            let is_focused = Some(idx) == focused_idx;
            let border_color = if is_focused {
                colors::BORDER_FOCUSED.into()
            } else {
                colors::BORDER_UNFOCUSED.into()
            };

            // Draw thick borders
            let b = BORDER_WIDTH;
            frame.clear(border_color, &[
                rect(geom.loc.x - b, geom.loc.y - b, geom.size.w + 2 * b, b), // Top
                rect(geom.loc.x - b, geom.loc.y + geom.size.h, geom.size.w + 2 * b, b), // Bottom
                rect(geom.loc.x - b, geom.loc.y, b, geom.size.h), // Left
                rect(geom.loc.x + geom.size.w, geom.loc.y, b, geom.size.h), // Right
            ])?;
        }

        // ---- 3. Island Panel (Floating) ----
        let panel_w = output_size.w - (PANEL_MARGIN * 2);
        let panel_x = PANEL_MARGIN;
        let panel_y = PANEL_MARGIN;

        // Main Panel Bar
        frame.clear(
            colors::PANEL_BG.into(),
            &[rect(panel_x, panel_y, panel_w, PANEL_HEIGHT)],
        )?;

        // Decorative Accent Line (Bottom of panel)
        frame.clear(
            colors::ACCENT_CRIMSON.into(),
            &[rect(panel_x + 20, panel_y + PANEL_HEIGHT - 2, 60, 2)],
        )?;

        // ---- 4. Launcher (Grid Style) ----
        if state.launcher.is_visible() {
            // Dark overlay
            frame.clear(
                [0.0_f32, 0.0, 0.0, 0.7].into(),
                &[rect(0, 0, output_size.w, output_size.h)],
            )?;

            let lw = 800.min(output_size.w - 100).max(0);
            let lh = 600.min(output_size.h - 200).max(0);
            let lx = (output_size.w - lw) / 2;
            let ly = (output_size.h - lh) / 2;

            // Launcher Box
            frame.clear(colors::LAUNCHER_BG.into(), &[rect(lx, ly, lw, lh)])?;
            
            // Search Bar Area
            frame.clear(
                [0.12_f32, 0.12, 0.18, 1.0].into(),
                &[rect(lx + 20, ly + 20, lw - 40, 50)],
            )?;

            // Grid Items
            let cols = 4;
            let item_w = (lw - 60) / cols;
            let item_h = 100;
            
            let visible_apps = state.launcher.visible_entries();
            let count = visible_apps.len().min(12);
            
            for i in 0..count { // Draw dynamically based on available apps
                let row = i as i32 / cols;
                let col = i as i32 % cols;
                let ix = lx + 30 + (col * item_w);
                let iy = ly + 90 + (row * item_h);
                
                let is_selected = state.launcher.selected_index() == Some(i as usize);
                let item_bg = if is_selected {
                    let mut c = colors::ACCENT_CRIMSON;
                    c[3] = 0.2;
                    c.into()
                } else {
                    [1.0_f32, 1.0, 1.0, 0.03].into()
                };

                frame.clear(item_bg, &[rect(ix + 5, iy + 5, item_w - 10, item_h - 10)])?;
                
                // Icon Placeholder
                frame.clear(
                    if is_selected { colors::ACCENT_CRIMSON.into() } else { colors::ACCENT_CYAN.into() },
                    &[rect(ix + (item_w / 2) - 15, iy + 20, 30, 30)]
                )?;
            }
        }

        // ---- 5. Cursor (Glow) ----
        let (cx, cy) = state.window_manager.cursor_position();
        frame.clear(
            colors::ACCENT_CYAN.into(),
            &[rect(cx as i32 - 4, cy as i32 - 4, 8, 8)],
        )?;

        Ok(())
    }
}
