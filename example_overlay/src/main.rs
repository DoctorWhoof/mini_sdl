use mini_sdl::*;
use sdl2::pixels::Color;

fn main() -> Result<(), String> {
    let mut app = mini_sdl::App::new(
        "test",
        320,
        240,
        Timing::VsyncLimitFPS(60.0),
        Scaling::PreserveAspect,
    )?;

    app.default_font = Some(
        app.font_load("example_font/classic-display/classic-display.ttf", 16)?
    );
    app.overlay_scale = 4.0;

    while !app.quit_requested {
        app.frame_start()?;
        app.overlay_push("This is the overlay!");
        app.overlay_push("It is always drawn on top of everything.");
        app.overlay_push("Every 'overlay_push' creates a new line.");
        app.overlay_push("Useful for things like showing the FPS...");
        app.overlay_push(format!("FPS: {}", app.fps()));
        // Draw directly to the render_target
        app.canvas
            .with_texture_canvas(&mut app.render_target, |texture_canvas| {
                texture_canvas.set_draw_color(Color::RGBA(100, 100, 100, 255));
                texture_canvas.clear();
                texture_canvas.set_draw_color(Color::RGBA(200, 120, 0, 255));
                texture_canvas.draw_line((10, 10), (310, 230)).unwrap();
                texture_canvas.draw_line((310, 10), (10, 230)).unwrap();
            })
            .map_err(|e| e.to_string())?;
        // Will present the render target with proper scaling on the canvas.
        // The overlay will still be drawn on top.
        app.render_target_present()?;
        app.frame_finish()?;
    }
    Ok(())
}
