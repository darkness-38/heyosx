// =============================================================================
// heyDM — Renderer
//
// Renders the desktop: background, windows, panel, launcher, cursor.
// Uses a GlesFrame obtained from the winit/DRM backend's render surface.
// =============================================================================

use smithay::backend::renderer::glow::GlowFrame;
use smithay::backend::renderer::Frame;
use smithay::output::Output;
use smithay::utils::{Physical, Rectangle, Size};

use crate::state::HeyDM;

/// Color constants for the heyOS desktop theme (RGBA f32)
pub mod colors {
    pub const BG_TOP: [f32; 4]             = [0.06, 0.07, 0.15, 1.0];
    pub const PANEL_BG: [f32; 4]           = [0.05, 0.05, 0.12, 0.90];
    pub const LAUNCHER_BG: [f32; 4]        = [0.04, 0.04, 0.10, 0.92];
    pub const LAUNCHER_HIGHLIGHT: [f32; 4] = [0.20, 0.40, 0.80, 0.60];
    pub const BORDER_FOCUSED: [f32; 4]     = [0.30, 0.55, 0.95, 1.0];
    pub const BORDER_UNFOCUSED: [f32; 4]   = [0.20, 0.22, 0.30, 0.60];
}

pub const PANEL_HEIGHT: i32 = 32;
pub const BORDER_WIDTH: i32 = 2;

/// Build a Rectangle from (x, y, w, h) — Rectangle::new uses loc + size
fn rect(x: i32, y: i32, w: i32, h: i32) -> Rectangle<i32, Physical> {
    Rectangle::new((x, y).into(), (w, h).into())
}

pub struct Renderer;

impl Renderer {
    /// Render a full frame into the given GlesFrame.
    /// The winit backend calls `backend.bind()` → `renderer.render()` externally,
    /// we receive the frame here and draw everything into it.
    pub fn render_frame(
        state: &HeyDM,
        frame: &mut GlowFrame<'_, '_>,
        _output: &Output,
        output_size: Size<i32, Physical>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // ---- 1. Background ----
        frame.clear(
            colors::BG_TOP.into(),
            &[rect(0, 0, output_size.w, output_size.h)],
        )?;

        // ---- 2. Windows ----
        let focused_idx = state.window_manager.windows().len().checked_sub(1);
        for (idx, window) in state.window_manager.windows().iter().enumerate() {
            let geom = window.geometry();
            let border_color = if Some(idx) == focused_idx {
                colors::BORDER_FOCUSED.into()
            } else {
                colors::BORDER_UNFOCUSED.into()
            };
            frame.clear(border_color, &[rect(
                geom.loc.x - BORDER_WIDTH, geom.loc.y - BORDER_WIDTH,
                geom.size.w + 2 * BORDER_WIDTH, BORDER_WIDTH,
            )])?;
            frame.clear(border_color, &[rect(
                geom.loc.x - BORDER_WIDTH, geom.loc.y + geom.size.h,
                geom.size.w + 2 * BORDER_WIDTH, BORDER_WIDTH,
            )])?;
            frame.clear(border_color, &[rect(
                geom.loc.x - BORDER_WIDTH, geom.loc.y,
                BORDER_WIDTH, geom.size.h,
            )])?;
            frame.clear(border_color, &[rect(
                geom.loc.x + geom.size.w, geom.loc.y,
                BORDER_WIDTH, geom.size.h,
            )])?;
        }

        // ---- 3. Panel ----
        frame.clear(
            colors::PANEL_BG.into(),
            &[rect(0, 0, output_size.w, PANEL_HEIGHT)],
        )?;

        // ---- 4. Launcher ----
        if state.launcher.is_visible() {
            frame.clear(
                [0.0_f32, 0.0, 0.0, 0.5].into(),
                &[rect(0, 0, output_size.w, output_size.h)],
            )?;

            let lw = 600.min(output_size.w - 100);
            let lh = 400.min(output_size.h - 200);
            let lx = (output_size.w - lw) / 2;
            let ly = (output_size.h - lh) / 2;

            frame.clear(colors::LAUNCHER_BG.into(), &[rect(lx, ly, lw, lh)])?;
            frame.clear(
                [0.08_f32, 0.08, 0.18, 0.95].into(),
                &[rect(lx + 10, ly + 10, lw - 20, 40)],
            )?;

            if let Some(selected) = state.launcher.selected_index() {
                let item_h = 36_i32;
                let item_y = ly + 50 + (selected as i32 * item_h);
                if item_y + item_h < ly + lh {
                    frame.clear(
                        colors::LAUNCHER_HIGHLIGHT.into(),
                        &[rect(lx + 10, item_y, lw - 20, item_h)],
                    )?;
                }
            }
        }

        // ---- 5. Cursor ----
        let (cx, cy) = state.window_manager.cursor_position();
        let cs = 8_i32;
        frame.clear(
            [1.0_f32, 1.0, 1.0, 0.9].into(),
            &[rect(cx as i32 - cs / 2, cy as i32 - cs / 2, cs, cs)],
        )?;

        Ok(())
    }
}
