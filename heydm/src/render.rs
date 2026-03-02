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

use crate::state::HeyDM;

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
        for window in state.window_manager.windows().iter() {
            if let Some(surface) = window.wl_surface() {
                let location = Point::from((
                    window.current_position.x as i32,
                    window.current_position.y as i32,
                ));
                let elements = render_elements_from_surface_tree(
                    renderer,
                    &surface,
                    location,
                    Scale::from(window.scale as f64),
                    window.opacity as f64,
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
        pixmap.fill(Color::from_rgba8(20, 18, 25, 255));

        let mut paint = Paint::default();
        paint.anti_alias = true;

        // Fluid Background Shapes
        paint.set_color_rgba8(35, 30, 45, 255);
        let mut pb = PathBuilder::new();
        pb.move_to(0.0, 0.0);
        pb.cubic_to(300.0, 100.0, 500.0, -100.0, 800.0, 200.0);
        pb.cubic_to(1000.0, 500.0, 600.0, 800.0, 200.0, 600.0);
        pb.close();
        if let Some(path) = pb.finish() {
            pixmap.fill_path(&path, &paint, FillRule::Winding, SkiaTransform::identity(), None);
        }

        // Window Decorations (Shadows and Borders)
        let focused_idx = state.window_manager.windows().len().checked_sub(1);
        for (idx, window) in state.window_manager.windows().iter().enumerate() {
            let is_focused = Some(idx) == focused_idx;
            
            let x = window.current_position.x as f32;
            let y = window.current_position.y as f32;
            let w = window.current_size.w as f32;
            let h = window.current_size.h as f32;
            let radius = 12.0 * window.scale; // Scale radius with the window

            // 1. Draw Shadow (Simple rounded rect with transparency)
            let shadow_offset = 6.0 * window.scale;
            let shadow_blur = 12.0 * window.scale;
            let shadow_opacity = (100.0 * window.opacity) as u8;
            paint.set_color_rgba8(0, 0, 0, shadow_opacity);
            
            if let Some(shadow_rect) = Rect::from_xywh(
                x - shadow_blur/2.0 + shadow_offset, 
                y - shadow_blur/2.0 + shadow_offset, 
                w + shadow_blur, 
                h + shadow_blur
            ) {
                let mut shadow_pb = PathBuilder::new();
                Self::draw_rounded_rect(&mut shadow_pb, shadow_rect, radius + 4.0);
                if let Some(path) = shadow_pb.finish() {
                    pixmap.fill_path(&path, &paint, FillRule::Winding, SkiaTransform::identity(), None);
                }
            }

            // 2. Draw Rounded Border
            let b = 2.0;
            let border_opacity = (255.0 * window.opacity) as u8;
            let border_color = if is_focused { 
                Color::from_rgba8(216, 180, 204, border_opacity) // heyOS Pink
            } else { 
                Color::from_rgba8(38, 35, 45, border_opacity)    // Deep Gray
            };
            paint.set_color(border_color);
            
            if let Some(border_rect) = Rect::from_xywh(x - b, y - b, w + 2.0 * b, h + 2.0 * b) {
                let mut border_pb = PathBuilder::new();
                Self::draw_rounded_rect(&mut border_pb, border_rect, radius + b);
                if let Some(path) = border_pb.finish() {
                    pixmap.fill_path(&path, &paint, FillRule::Winding, SkiaTransform::identity(), None);
                }
            }
        }

        // Top Bar Pills
        paint.set_color_rgba8(25, 22, 30, 200);
        let pill_y = 10.0;
        let pill_h = 32.0;
        pixmap.fill_rect(Rect::from_xywh(10.0, pill_y, 240.0, pill_h).unwrap(), &paint, SkiaTransform::identity(), None);
        pixmap.fill_rect(Rect::from_xywh(260.0, pill_y, 180.0, pill_h).unwrap(), &paint, SkiaTransform::identity(), None);
        let center_x = output_size.w as f32 / 2.0;
        pixmap.fill_rect(Rect::from_xywh(center_x - 150.0, pill_y, 300.0, pill_h).unwrap(), &paint, SkiaTransform::identity(), None);
        pixmap.fill_rect(Rect::from_xywh(output_size.w as f32 - 450.0, pill_y, 180.0, pill_h).unwrap(), &paint, SkiaTransform::identity(), None);
        pixmap.fill_rect(Rect::from_xywh(output_size.w as f32 - 260.0, pill_y, 250.0, pill_h).unwrap(), &paint, SkiaTransform::identity(), None);

        // Sidebar
        pixmap.fill_rect(Rect::from_xywh(10.0, 50.0, 400.0, output_size.h as f32 - 60.0).unwrap(), &paint, SkiaTransform::identity(), None);

        // Settings Window
        let sw_w = 700.0;
        let sw_h = 450.0;
        let sw_x = (output_size.w as f32 - sw_w) / 2.0 + 100.0;
        let sw_y = (output_size.h as f32 - sw_h) / 2.0 - 50.0;
        pixmap.fill_rect(Rect::from_xywh(sw_x, sw_y, sw_w, sw_h).unwrap(), &paint, SkiaTransform::identity(), None);

        // Blob Clock
        paint.set_color_rgba8(120, 100, 125, 220);
        let cw_x = output_size.w as f32 / 2.0 + 150.0;
        let cw_y = output_size.h as f32 / 2.0;
        let mut pb_clock = PathBuilder::new();
        pb_clock.move_to(cw_x - 90.0, cw_y);
        pb_clock.cubic_to(cw_x - 90.0, cw_y - 110.0, cw_x + 90.0, cw_y - 110.0, cw_x + 90.0, cw_y);
        pb_clock.cubic_to(cw_x + 90.0, cw_y + 110.0, cw_x - 90.0, cw_y + 110.0, cw_x - 90.0, cw_y);
        if let Some(path) = pb_clock.finish() {
            pixmap.fill_path(&path, &paint, FillRule::Winding, SkiaTransform::identity(), None);
        }

        pixmap
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

    pub fn render_frame<R, F>(
        state: &HeyDM,
        renderer: &mut R,
        frame: &mut F,
        render_elements: &[WaylandSurfaceRenderElement<R>],
        ui_texture: &R::TextureId,
        output_size: Size<i32, Physical>,
    ) -> Result<(), Box<dyn std::error::Error>> 
    where 
        R: SmithayRenderer + ImportAll + ImportMemWl + ImportDmaWl + 'static,
        F: Frame<Error = R::Error, TextureId = R::TextureId>,
        R::TextureId: Clone + 'static,
    {
        // 1. Draw UI Texture
        frame.clear([0.0, 0.0, 0.0, 1.0], &[Rectangle::new((0, 0).into(), output_size)])?;
        
        Frame::draw_texture(
            frame,
            ui_texture,
            Rectangle::new((0, 0).into(), output_size),
            Rectangle::new((0, 0).into(), output_size),
            &[],
            1.0,
        )?;

        // 2. Real Wayland windows and surface elements are drawn here.
        // We've moved the border/shadow logic to draw_ui (tiny-skia) for high-fidelity effects.

        // Draw Wayland surface elements
        if !render_elements.is_empty() {
            let damage = vec![Rectangle::new((0, 0).into(), output_size)];
            let _ = draw_render_elements(renderer, frame, render_elements, &damage);
        }

        // 3. Cursor
        let (cx, cy) = state.window_manager.cursor_position();
        frame.clear([1.0, 1.0, 1.0, 1.0].into(), &[Rectangle::new((cx as i32 - 2, cy as i32 - 2).into(), (4, 4).into())])?;

        Ok(())
    }
}
