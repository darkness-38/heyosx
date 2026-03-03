// =============================================================================
// heyDM — Renderer
//
// Renders the desktop: background, windows, panel, launcher, cursor.
// Uses a GlesFrame obtained from the winit/DRM backend's render surface.
// =============================================================================

use smithay::backend::renderer::{Frame, Renderer as SmithayRenderer, ImportAll, ImportMemWl, ImportDmaWl};
use smithay::utils::{Physical, Rectangle, Size, Point, Scale};
use smithay::backend::renderer::element::surface::{
    render_elements_from_surface_tree, WaylandSurfaceRenderElement,
};
use smithay::backend::renderer::element::Kind;
use smithay::backend::renderer::utils::draw_render_elements;
use tiny_skia::{Pixmap, Paint, FillRule, PathBuilder, Color, Rect, Transform as SkiaTransform};
use tracing::error;
use fontdue::{Font, FontSettings};
use std::sync::OnceLock;

use crate::state::HeyDM;

static FONT: OnceLock<Font> = OnceLock::new();

pub struct Renderer;

impl Renderer {
    /// Gather Wayland surface elements before starting the frame to avoid double borrow
    pub fn gather_elements<R>(
        state: &HeyDM,
        renderer: &mut R,
    ) -> Vec<WaylandSurfaceRenderElement<R>>
    where 
        R: SmithayRenderer + ImportAll + ImportMemWl + ImportDmaWl + 'static,
        R::TextureId: Clone + 'static,
    {
        let mut render_elements = Vec::new();
        let output_w = state.output_size.w as f64;

        for window in state.window_manager.windows().iter() {
            let ws_offset_x = (window.workspace as f64 - state.window_manager.current_workspace_offset) * output_w;
            let x = window.current_position.x + ws_offset_x;
            let w = window.current_size.w;

            // Skip if completely off-screen
            if x + w < 0.0 || x > output_w {
                continue;
            }

            if let Some(surface) = window.wl_surface() {
                let location = Point::from((
                    x as i32,
                    window.current_position.y as i32,
                ));
                let elements = render_elements_from_surface_tree(
                    renderer,
                    &surface,
                    location,
                    Scale::from(window.scale as f64),
                    window.opacity as f32,
                    Kind::Unspecified,
                );
                render_elements.extend(elements);
            }
        }
        render_elements
    }

    /// Draw the 1:1 UI to a Pixmap
    pub fn draw_ui(state: &HeyDM) -> Pixmap {
        let output_size = state.output_size;
        let mut pixmap = Pixmap::new(output_size.w as u32, output_size.h as u32).unwrap();
        pixmap.fill(Color::from_rgba8(15, 12, 20, 255)); // Deep Indigo Base

        let mut paint = Paint::default();
        paint.anti_alias = true;

        // 1. Living Background (Animated Shapes)
        static START_TIME: OnceLock<std::time::Instant> = OnceLock::new();
        let time = START_TIME.get_or_init(std::time::Instant::now).elapsed().as_secs_f64();
        
        let blob_colors = [
            Color::from_rgba8(216, 180, 204, 25), // heyOS Pink (Low Opacity)
            Color::from_rgba8(189, 147, 249, 20), // Purple
            Color::from_rgba8(72, 60, 120, 15),   // Deep Indigo
        ];

        for (i, color) in blob_colors.iter().enumerate() {
            let t = time * (0.12 + i as f64 * 0.04);
            let cx = (output_size.w as f32 * (0.3 + 0.2 * i as f32)) + ((t + i as f64).sin() * 150.0) as f32;
            let cy = (output_size.h as f32 * (0.5 + 0.1 * i as f32)) + ((t * 0.7).cos() * 120.0) as f32;
            let r = 400.0 + ((t * 0.5).sin() * 50.0) as f32;
            
            paint.set_color(*color);
            let mut pb = PathBuilder::new();
            pb.move_to(cx - r, cy);
            pb.cubic_to(cx - r, cy - r * 1.1, cx + r, cy - r * 0.9, cx + r, cy);
            pb.cubic_to(cx + r, cy + r * 1.1, cx - r, cy + r * 0.9, cx - r, cy);
            pb.close();
            
            if let Some(path) = pb.finish() {
                pixmap.fill_path(&path, &paint, FillRule::Winding, SkiaTransform::identity(), None);
            }
        }

        // Initialize Font
        let font = FONT.get_or_init(|| {
            let font_data = std::fs::read("/usr/share/fonts/TTF/DejaVuSans.ttf")
                .or_else(|_| std::fs::read("/usr/share/fonts/noto/NotoSans-Regular.ttf"))
                .expect("Failed to load font");
            Font::from_bytes(font_data, FontSettings::default()).expect("Failed to parse font")
        });

        // Window Decorations (Shadows and Borders)
        let output_w = output_size.w as f64;
        let ws_offset_base = state.window_manager.current_workspace_offset;
        let focused_ptr = state.window_manager.focused_window().map(|w| w as *const _);

        for window in state.window_manager.windows().iter() {
            let ws_offset_x = (window.workspace as f64 - ws_offset_base) * output_w;
            
            let x = window.current_position.x + ws_offset_x;
            let y = window.current_position.y;
            let w = window.current_size.w;
            let h = window.current_size.h;

            // Skip if completely off-screen
            if x + w < 0.0 || x > output_w {
                continue;
            }

            let is_focused = focused_ptr.map_or(false, |ptr| std::ptr::eq(ptr, *window as *const _));
            
            let x_f = x as f32;
            let y_f = y as f32;
            let w_f = w as f32;
            let h_f = h as f32;
            let radius = 12.0 * window.scale;

            // Soft Shadows (Layered)
            for i in 1..4 {
                let offset = (i as f32 * 3.0) * window.scale;
                let blur = (i as f32 * 6.0) * window.scale;
                paint.set_color_rgba8(0, 0, 0, (40 / i) as u8);
                
                if let Some(sr) = Rect::from_xywh(x_f - blur/2.0 + offset/2.0, y_f - blur/2.0 + offset/2.0, w_f + blur, h_f + blur) {
                    let mut spb = PathBuilder::new();
                    Self::draw_rounded_rect(&mut spb, sr, radius + 2.0);
                    if let Some(path) = spb.finish() {
                        pixmap.fill_path(&path, &paint, FillRule::Winding, SkiaTransform::identity(), None);
                    }
                }
            }

            // Borders
            let b = if is_focused { 2.5 } else { 1.5 };
            let border_opacity = (255.0 * window.opacity) as u8;
            let color = if is_focused { 
                Color::from_rgba8(216, 180, 204, border_opacity) // heyOS Pink
            } else { 
                Color::from_rgba8(45, 42, 55, (180.0 * window.opacity) as u8) 
            };
            paint.set_color(color);
            
            if let Some(br) = Rect::from_xywh(x_f - b, y_f - b, w_f + 2.0 * b, h_f + 2.0 * b) {
                let mut bpb = PathBuilder::new();
                Self::draw_rounded_rect(&mut bpb, br, radius + b);
                if let Some(path) = bpb.finish() {
                    pixmap.fill_path(&path, &paint, FillRule::Winding, SkiaTransform::identity(), None);
                }
            }
        }

        // 3. Draw Animated Focus Indicator
        if state.window_manager.focus_opacity > 0.01 {
            if let Some(focused) = state.window_manager.focused_window() {
                let ws_offset_x = (focused.workspace as f64 - ws_offset_base) * output_w;
                
                let x = state.window_manager.current_focus_pos.x + ws_offset_x;
                let y = state.window_manager.current_focus_pos.y;
                let w = state.window_manager.current_focus_size.w;
                let h = state.window_manager.current_focus_size.h;

                let x_f = x as f32;
                let y_f = y as f32;
                let w_f = w as f32;
                let h_f = h as f32;
                let radius = 14.0;
                let b = 3.0;
                
                let opacity = (255.0 * state.window_manager.focus_opacity) as u8;
                
                // --- Premium Magnetic Bloom Effect ---
                // Deep Glow
                paint.set_color_rgba8(189, 147, 249, opacity / 6);
                for i in 1..4 {
                    let spread = i as f32 * 6.0;
                    if let Some(glow_rect) = Rect::from_xywh(x_f - spread, y_f - spread, w_f + spread * 2.0, h_f + spread * 2.0) {
                        let mut glow_pb = PathBuilder::new();
                        Self::draw_rounded_rect(&mut glow_pb, glow_rect, radius + spread);
                        if let Some(path) = glow_pb.finish() {
                            pixmap.fill_path(&path, &paint, FillRule::Winding, SkiaTransform::identity(), None);
                        }
                    }
                }

                // Inner Vibrant Glow
                paint.set_color_rgba8(216, 180, 204, opacity / 2);
                if let Some(inner_glow) = Rect::from_xywh(x_f - 2.0, y_f - 2.0, w_f + 4.0, h_f + 4.0) {
                    let mut ig_pb = PathBuilder::new();
                    Self::draw_rounded_rect(&mut ig_pb, inner_glow, radius + 1.0);
                    if let Some(path) = ig_pb.finish() {
                        pixmap.fill_path(&path, &paint, FillRule::Winding, SkiaTransform::identity(), None);
                    }
                }

                // Primary Sharp Border
                paint.set_color_rgba8(216, 180, 204, opacity);
                if let Some(focus_rect) = Rect::from_xywh(x_f - b, y_f - b, w_f + 2.0 * b, h_f + 2.0 * b) {
                    let mut focus_pb = PathBuilder::new();
                    Self::draw_rounded_rect(&mut focus_pb, focus_rect, radius + b);
                    if let Some(path) = focus_pb.finish() {
                        pixmap.fill_path(&path, &paint, FillRule::Winding, SkiaTransform::identity(), None);
                    }
                }
            }
        }

        // --- Glassmorphism UI Elements ---
        let bar_color = Color::from_rgba8(25, 22, 30, 200);
        let text_color = Color::from_rgba8(235, 235, 235, 255);
        let active_color = Color::from_rgba8(216, 180, 204, 255);

        // 1. Status Bar Shell
        let bar_y = 10.0;
        let bar_h = 40.0;
        let bar_margin = 15.0;
        let bar_width = output_size.w as f32 - (bar_margin * 2.0);
        
        paint.set_color(bar_color);
        if let Some(bar_rect) = Rect::from_xywh(bar_margin, bar_y, bar_width, bar_h) {
            let mut bar_pb = PathBuilder::new();
            Self::draw_rounded_rect(&mut bar_pb, bar_rect, 12.0);
            if let Some(path) = bar_pb.finish() {
                pixmap.fill_path(&path, &paint, FillRule::Winding, SkiaTransform::identity(), None);
            }
        }

        // 2. Workspace Indicators (Inside Bar)
        for i in 0..9 {
            let ix = bar_margin + 20.0 + (i as f32 * 28.0);
            let is_active = i == state.window_manager.current_workspace;
            paint.set_color(if is_active { active_color } else { Color::from_rgba8(65, 60, 85, 180) });
            
            let dot_size = if is_active { 10.0 } else { 6.0 };
            let dot_y = bar_y + (bar_h / 2.0) - (dot_size / 2.0);
            
            if let Some(r_rect) = Rect::from_xywh(ix, dot_y, dot_size, dot_size) {
                let mut r_pb = PathBuilder::new();
                Self::draw_rounded_rect(&mut r_pb, r_rect, dot_size / 2.0);
                if let Some(path) = r_pb.finish() {
                    pixmap.fill_path(&path, &paint, FillRule::Winding, SkiaTransform::identity(), None);
                }
            }
        }

        // 3. Center Title
        let title = state.window_manager.focused_window()
            .and_then(|w| {
                smithay::wayland::compositor::with_states(w.wl_surface().as_ref().unwrap(), |states| {
                    states.data_map
                        .get::<smithay::wayland::shell::xdg::XdgToplevelSurfaceData>()
                        .and_then(|data| data.lock().unwrap().title.clone())
                })
            })
            .unwrap_or_else(|| "heyOS".to_string());
        
        let center_x = output_size.w as f32 / 2.0;
        Self::draw_text(&mut pixmap, font, &title, center_x - 60.0, bar_y + 26.0, 16.0, text_color);

        // 4. System Info (Right)
        let right_offset = output_size.w as f32 - bar_margin - 20.0;
        
        // Clock
        let clock = state.panel.clock_text();
        Self::draw_text(&mut pixmap, font, clock, right_offset - 120.0, bar_y + 26.0, 14.0, text_color);

        // Icons
        let icon_x = right_offset - 180.0;
        let icon_y = bar_y + 12.0;
        
        Self::draw_battery_icon(&mut pixmap, icon_x, icon_y, state.panel.battery_percent(), state.panel.is_charging());
        Self::draw_wifi_icon(&mut pixmap, icon_x - 30.0, icon_y, state.panel.network_status());

        // Sidebar Shell
        paint.set_color(Color::from_rgba8(25, 22, 30, 150));
        if let Some(side_rect) = Rect::from_xywh(15.0, 65.0, 280.0, output_size.h as f32 - 80.0) {
            let mut side_pb = PathBuilder::new();
            Self::draw_rounded_rect(&mut side_pb, side_rect, 16.0);
            if let Some(path) = side_pb.finish() {
                pixmap.fill_path(&path, &paint, FillRule::Winding, SkiaTransform::identity(), None);
            }
        }

        pixmap
    }

    fn draw_wifi_icon(pixmap: &mut Pixmap, x: f32, y: f32, status: &crate::panel::NetworkStatus) {
        let mut paint = Paint::default();
        paint.anti_alias = true;
        
        match status {
            crate::panel::NetworkStatus::Disconnected => paint.set_color_rgba8(255, 100, 100, 200),
            _ => paint.set_color_rgba8(235, 235, 235, 255),
        }

        let size = 16.0;
        for i in 0..3 {
            let r = size - (i as f32 * 5.0);
            let mut pb = PathBuilder::new();
            pb.move_to(x + size/2.0 - r/2.0, y + size - (i as f32 * 3.0));
            pb.cubic_to(
                x + size/2.0 - r/2.0, y + size - r,
                x + size/2.0 + r/2.0, y + size - r,
                x + size/2.0 + r/2.0, y + size - (i as f32 * 3.0)
            );
            
            if let Some(path) = pb.finish() {
                let mut stroke = smithay::backend::renderer::element::Kind::Unspecified; // Dummy
                // We use stroke-like filling for minimalist arcs
                pixmap.fill_path(&path, &paint, FillRule::Winding, SkiaTransform::identity(), None);
            }
        }
        
        // Dot at bottom
        if let Some(dot) = Rect::from_xywh(x + size/2.0 - 1.5, y + size - 2.0, 3.0, 3.0) {
            pixmap.fill_rect(dot, &paint, SkiaTransform::identity(), None);
        }
    }

    fn draw_battery_icon(pixmap: &mut Pixmap, x: f32, y: f32, percent: i32, charging: bool) {
        let mut paint = Paint::default();
        paint.anti_alias = true;
        
        let color = if charging {
            Color::from_rgba8(150, 255, 150, 255)
        } else if percent < 20 {
            Color::from_rgba8(255, 100, 100, 255)
        } else {
            Color::from_rgba8(235, 235, 235, 255)
        };
        paint.set_color(color);

        // Body
        if let Some(body) = Rect::from_xywh(x, y + 2.0, 22.0, 12.0) {
            let mut pb = PathBuilder::new();
            Self::draw_rounded_rect(&mut pb, body, 3.0);
            if let Some(path) = pb.finish() {
                let mut b_paint = Paint::default();
                b_paint.anti_alias = true;
                let mut b_color = color;
                b_color.set_alpha(0.3);
                b_paint.set_color(b_color);
                pixmap.fill_path(&path, &b_paint, FillRule::Winding, SkiaTransform::identity(), None);
            }
        }

        // Tip
        if let Some(tip) = Rect::from_xywh(x + 22.0, y + 6.0, 2.0, 4.0) {
            pixmap.fill_rect(tip, &paint, SkiaTransform::identity(), None);
        }

        // Level
        if percent > 0 {
            let level_w = (percent as f32 / 100.0) * 18.0;
            if let Some(level) = Rect::from_xywh(x + 2.0, y + 4.0, level_w, 8.0) {
                let mut pb = PathBuilder::new();
                Self::draw_rounded_rect(&mut pb, level, 1.5);
                if let Some(path) = pb.finish() {
                    pixmap.fill_path(&path, &paint, FillRule::Winding, SkiaTransform::identity(), None);
                }
            }
        }

        // Charging Bolt (simplified)
        if charging {
            paint.set_color_rgba8(255, 255, 0, 255);
            let mut pb = PathBuilder::new();
            pb.move_to(x + 11.0, y);
            pb.line_to(x + 7.0, y + 8.0);
            pb.line_to(x + 10.0, y + 8.0);
            pb.line_to(x + 9.0, y + 16.0);
            pb.line_to(x + 13.0, y + 8.0);
            pb.line_to(x + 10.0, y + 8.0);
            pb.close();
            if let Some(path) = pb.finish() {
                pixmap.fill_path(&path, &paint, FillRule::Winding, SkiaTransform::identity(), None);
            }
        }
    }

    /// Helper to draw text using fontdue
    fn draw_text(pixmap: &mut Pixmap, font: &Font, text: &str, x: f32, y: f32, size: f32, color: Color) {
        let mut curr_x = x;
        for c in text.chars() {
            let (metrics, bitmap) = font.rasterize(c, size);
            
            for row in 0..metrics.height {
                for col in 0..metrics.width {
                    let alpha = bitmap[row * metrics.width + col];
                    if alpha > 0 {
                        let mut c_paint = Paint::default();
                        let mut final_color = color;
                        final_color.set_alpha(color.alpha() * (alpha as f32 / 255.0));
                        c_paint.set_color(final_color);
                        
                        let px = curr_x + metrics.xmin as f32 + col as f32;
                        let py = y - metrics.height as f32 - metrics.ymin as f32 + row as f32;
                        
                        if let Some(p_rect) = Rect::from_xywh(px, py, 1.0, 1.0) {
                            pixmap.fill_rect(p_rect, &c_paint, SkiaTransform::identity(), None);
                        }
                    }
                }
            }
            curr_x += metrics.advance_width;
        }
    }

    /// Helper to draw a rounded rectangle path
    fn draw_rounded_rect(pb: &mut PathBuilder, rect: Rect, radius: f32) {
        let left = rect.left();
        let top = rect.top();
        let right = rect.right();
        let bottom = rect.bottom();
        
        pb.move_to(left + radius, top);
        pb.line_to(right - radius, top);
        pb.quad_to(right, top, right, top + radius);
        pb.line_to(right, bottom - radius);
        pb.quad_to(right, bottom, right - radius, bottom);
        pb.line_to(left + radius, bottom);
        pb.quad_to(left, bottom, left, bottom - radius);
        pb.line_to(left, top + radius);
        pb.quad_to(left, top, left + radius, top);
        pb.close();
    }

    pub fn render_frame<R>(
        state: &HeyDM,
        frame: &mut R::Frame<'_, '_>,
        render_elements: &[WaylandSurfaceRenderElement<R>],
        ui_texture: &R::TextureId,
        output_size: Size<i32, Physical>,
    ) -> Result<(), Box<dyn std::error::Error>> 
    where 
        R: SmithayRenderer + ImportAll + ImportMemWl + ImportDmaWl + 'static,
        R::TextureId: Clone + 'static,
    {
        // 1. Draw UI Texture
        frame.clear([0.0, 0.0, 0.0, 1.0].into(), &[Rectangle::new((0, 0).into(), output_size)])?;

        let ui_rect = Rectangle::new((0, 0).into(), output_size);
        let src_rect = Rectangle::new((0.0, 0.0).into(), (output_size.w as f64, output_size.h as f64).into());

        if let Err(e) = frame.render_texture_from_to(
            ui_texture,
            src_rect,
            ui_rect,
            &[],
            &[],
            smithay::utils::Transform::Normal,
            1.0,
        ) {
            error!("Failed to render UI texture: {}", e);
        }

        // 2. Draw Wayland surface elements
        if !render_elements.is_empty() {
            let damage = vec![Rectangle::new((0, 0).into(), output_size)];
            let _ = draw_render_elements(frame, 1.0, render_elements, &damage);
        }

        // 3. Cursor
        let (cx, cy) = state.window_manager.cursor_position();
        let cursor_rect = Rectangle::new((cx as i32 - 3, cy as i32 - 3).into(), (6, 6).into());
        frame.clear([1.0, 1.0, 1.0, 1.0].into(), &[cursor_rect])?;

        Ok(())
    }
}
