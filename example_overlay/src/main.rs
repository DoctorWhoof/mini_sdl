use mini_sdl::{sdl3::render::FPoint, *};
use sdl3::pixels::Color;

fn main() -> SdlResult<()> {
    let mut app = mini_sdl::App::new(
        "test",
        320,
        240,
        Timing::VsyncLimitFPS(60.0),
        Scaling::PreserveAspect
    )?;

    app.default_font = Some(app.font_load("example_overlay/src/roboto_medium.ttf", 36.0, 1.25)?);
    app.overlay_scale = 1.0;
    app.init_render_target()?;

    while !app.quit_requested {
        app.frame_start()?;
        app.overlay_push("This is the overlay!");
        app.overlay_push("It is always drawn on top of everything.");
        app.overlay_push("Every 'overlay_push' creates a new line.");
        app.overlay_push("Useful for things like showing the FPS...");
        app.overlay_push(format!("FPS: {}", app.fps()));
        // Draw directly to the render_target
        let Some(render_target) = &mut app.render_target else {
            println!("Render target not found");
            break
        };
        app.canvas
            .with_texture_canvas(render_target, |texture_canvas| {
                texture_canvas.set_draw_color(Color::RGBA(100, 100, 100, 255));
                texture_canvas.clear();
                texture_canvas.set_draw_color(Color::RGBA(200, 120, 0, 255));
                texture_canvas
                    .draw_line(point(10, 10), point(310, 230))
                    .unwrap();
                texture_canvas
                    .draw_line(point(310, 10), point(10, 230))
                    .unwrap();
            })?;
        // Will present the render target with proper scaling on the canvas.
        // The overlay will still be drawn on top.
        app.render_target_present()?;
        app.frame_finish()?;
    }
    Ok(())
}

fn point(x: u32, y: u32) -> FPoint {
    FPoint::new(x as f32, y as f32)
}
